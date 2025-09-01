mod array_of_table;
mod key;
mod key_value;
mod root;
mod table;
mod value;

pub trait Lint {
    async fn lint(&self, l: &mut crate::Linter<'_>);
}
