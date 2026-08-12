use crate::app::CommonArgs;
use tower::ServiceExt as _;

/// Run TOML Language Server.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    common: CommonArgs,
}

pub fn run(args: impl Into<Args>) -> Result<(), crate::Error> {
    let args: Args = args.into();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move {
        log::info!(
            "Tombi Language Server version \"{}\" will start.",
            env!("CARGO_PKG_VERSION")
        );

        let (service, socket) = tombi_lsp::lsp_service(args.common.offline, args.common.no_cache);
        serve(tokio::io::stdin(), tokio::io::stdout(), socket, service).await;

        log::info!("Tombi LSP Server did shut down.");
    });

    runtime.shutdown_timeout(std::time::Duration::from_secs(1));

    Ok(())
}

async fn serve<I, O>(
    input: I,
    output: O,
    socket: tower_lsp::ClientSocket,
    service: tower_lsp::LspService<tombi_lsp::Backend>,
) where
    I: tokio::io::AsyncRead + Unpin,
    O: tokio::io::AsyncWrite,
{
    let (exit_tx, exit_rx) = tokio::sync::oneshot::channel();
    let mut exit_tx = Some(exit_tx);
    let service = service.map_request(move |request: tower_lsp::jsonrpc::Request| {
        if request.method() == "exit"
            && let Some(exit_tx) = exit_tx.take()
        {
            let _ = exit_tx.send(());
        }
        request
    });

    tokio::select! {
        _ = tower_lsp::Server::new(input, output, socket).serve(service) => {}
        _ = exit_rx => {}
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::io::AsyncWriteExt as _;

    #[tokio::test]
    async fn exit_notification_stops_server_while_input_remains_open() {
        let (mut client_input, server_input) = tokio::io::duplex(1024);
        let (server_output, _client_output) = tokio::io::duplex(1024);
        let (service, socket) = tombi_lsp::lsp_service(true, true);

        client_input
            .write_all(b"Content-Length: 33\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"exit\"}")
            .await
            .unwrap();

        tokio::time::timeout(
            Duration::from_secs(1),
            super::serve(server_input, server_output, socket, service),
        )
        .await
        .expect("the server must not wait for the client to close stdin");
    }
}
