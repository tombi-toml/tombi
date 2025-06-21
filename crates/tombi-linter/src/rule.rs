mod key_empty;
mod dotted_keys_out_of_order;
pub use key_empty::KeyEmptyRule;
pub use dotted_keys_out_of_order::DottedKeysOutOfOrderRule;

pub trait Rule<N: tombi_ast::AstNode> {
    fn check(node: &N, l: &mut crate::Linter);
}
