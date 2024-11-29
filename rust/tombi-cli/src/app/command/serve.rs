pub use lsp::Args;

pub fn run(args: impl Into<Args>) -> Result<(), crate::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(lsp::serve(args))?;

    Ok(())
}
