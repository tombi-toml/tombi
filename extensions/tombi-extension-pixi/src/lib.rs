mod channels;
mod completion;
mod conda;
mod dependency;
mod hover;

pub use completion::completion;
pub use hover::hover;

pub(crate) const PIXI_TOML_FILENAME: &str = "pixi.toml";

pub(crate) fn is_pixi_toml(text_document_uri: &tombi_uri::Uri) -> bool {
    text_document_uri.path().ends_with(PIXI_TOML_FILENAME)
}
