use crate::app::CommonArgs;

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
        tower_lsp::Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
            .serve(service)
            .await;

        log::info!("Tombi LSP Server did shut down.");
    });

    runtime.shutdown_timeout(std::time::Duration::from_secs(1));

    Ok(())
}
