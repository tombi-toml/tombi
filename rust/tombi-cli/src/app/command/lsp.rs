pub use tombi_lsp::Args;

pub fn run(args: impl Into<Args>, offline: bool, no_cache: bool) -> Result<(), crate::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(tombi_lsp::serve(args, offline, no_cache));

    Ok(())
}
