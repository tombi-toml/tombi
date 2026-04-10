use crate::app::CommonArgs;
use std::time::Duration;

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

    runtime.block_on(tombi_lsp::serve(
        tombi_lsp::Args {},
        args.common.offline,
        args.common.no_cache,
    ));
    runtime.shutdown_timeout(Duration::from_secs(1));

    Ok(())
}
