mod array_of_table;
mod key_value;
mod root;
mod table;
mod value;

pub trait Lint {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()>;
}
