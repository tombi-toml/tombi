mod dotted_keys_out_of_order;
mod key_empty;
mod tables_out_of_order;
pub use dotted_keys_out_of_order::DottedKeysOutOfOrderRule;
pub use key_empty::KeyEmptyRule;
pub use tables_out_of_order::TablesOutOfOrderRule;

pub trait Rule<N> {
    async fn check(node: &N, l: &mut crate::Linter<'_>);
}
