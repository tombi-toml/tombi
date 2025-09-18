use crate::args::LspArgs;

pub fn run(args: LspArgs) -> Result<(), crate::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(tombi_lsp::serve(
            tombi_lsp::Args {},
            args.common.offline,
            args.common.no_cache,
        ));

    Ok(())
}
