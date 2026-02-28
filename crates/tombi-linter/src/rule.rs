mod dotted_keys_out_of_order;
mod inline_table_toml_version;
mod key_empty;
mod missing_comma;
mod tables_out_of_order;
pub use dotted_keys_out_of_order::DottedKeysOutOfOrderRule;
pub use inline_table_toml_version::InlineTableTomlVersionRule;
pub use key_empty::KeyEmptyRule;
pub use missing_comma::MissingCommaRule;
pub use tables_out_of_order::TablesOutOfOrderRule;

pub trait Rule<N> {
    async fn check(node: &N, l: &mut crate::Linter<'_>);
}
