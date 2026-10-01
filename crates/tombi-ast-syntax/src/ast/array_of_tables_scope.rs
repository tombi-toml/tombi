use tombi_toml_version::TomlVersion;

/// Incrementally tracks the `[[...]]` headers that precede the current table.
///
/// `parent_array_of_tables_keys` walks the previous siblings of a table every
/// time it is called, which is `O(n)` per table and `O(n^2)` for a document.
/// Callers that visit the top-level tables in source order can instead record
/// every `[[...]]` header once with [`ArrayOfTablesScope::enter`] and look the
/// state up with [`ArrayOfTablesScope::get`].
///
/// The scope reproduces the semantics of `parent_array_of_tables_keys`: only
/// the run of preceding `[[...]]` headers sharing the first key with the
/// current header is visible. A `[[...]]` header with a different first key
/// starts a new run and hides everything recorded before it.
#[derive(Debug, Clone)]
pub struct ArrayOfTablesScope<T> {
    first_key: Option<String>,
    value: T,
}

impl<T: Default> Default for ArrayOfTablesScope<T> {
    fn default() -> Self {
        Self {
            first_key: None,
            value: T::default(),
        }
    }
}

impl<T: Default> ArrayOfTablesScope<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the state recorded for `header` if the current run shares its first key.
    pub fn get(&self, header: Option<&crate::Keys>, toml_version: TomlVersion) -> Option<&T> {
        let first_key = self.first_key.as_deref()?;
        let header_first_key = header?.keys().next()?;
        let header_first_key = header_first_key.try_to_content(toml_version).ok()?;
        (header_first_key == first_key).then_some(&self.value)
    }

    /// Registers the header of an `[[...]]` table and returns the state of its run.
    ///
    /// Returns `None` when the header is missing or its first key is unusable.
    /// A header whose first key differs from the current run starts a new run.
    pub fn enter(&mut self, header: Option<&crate::Keys>) -> Option<&mut T> {
        let header = header?;
        let first_key = header.keys().next().and_then(|key| {
            key.try_to_content(TomlVersion::latest())
                .ok()
                .map(std::borrow::Cow::into_owned)
        });

        if first_key.is_none() || first_key != self.first_key {
            self.value = T::default();
            self.first_key = first_key;
        }

        self.first_key.as_ref().map(|_| &mut self.value)
    }
}
