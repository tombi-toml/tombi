mod array;
mod array_or_table;
mod builder;
mod comment;
mod inline_table;
mod key;
mod key_value;
mod root;
mod table;
mod token_type;
mod value;

pub use builder::SemanticTokensBuilder;
pub use token_type::{SUPPORTED_TOKEN_TYPES, TokenType};

pub trait AppendSemanticTokens {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder);
}
