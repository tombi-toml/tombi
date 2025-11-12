use indexmap::{
    map::{Entry, MutableKeys},
    IndexMap,
};
use itertools::Itertools;
use tombi_ast::{AstChildren, AstNode, TombiValueCommentDirective};
use tombi_toml_version::TomlVersion;

use crate::{
    Array, DocumentTreeAndErrors, IntoDocumentTreeAndErrors, Key, Value, ValueImpl, ValueType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    /// A root table.
    Root,

    /// A table.
    ///
    /// ```toml
    /// [key1.key2.key3]
    ///            ^^^^
    /// ```
    Table,

    /// A table of parent keys.
    ///
    /// ```toml
    /// [key1.key2.key3]
    ///  ^^^^^^^^^
    /// ```
    ParentTable,

    /// An inline table.
    ///
    /// ```toml
    /// key1 = { key2 = "value" }
    ///        ^^^^^^^^^^^^^^^^^^
    ///
    /// ```
    InlineTable { has_comment: bool },

    /// A table of parent keys.
    ///
    /// ```toml
    /// key1.key2.key3 = "value"
    /// ^^^^^^^^^
    /// ```
    ParentKey,

    /// A key-value.
    ///
    /// ```toml
    /// key1.key2.key3 = "value"
    ///           ^^^^^^^^^^^^^^
    /// ```
    KeyValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    kind: TableKind,
    range: tombi_text::Range,
    symbol_range: tombi_text::Range,
    key_values: IndexMap<Key, Value>,
    pub(crate) comment_directives: Option<Vec<TombiValueCommentDirective>>,
    pub(crate) inner_comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl Table {
    pub(crate) fn new_empty() -> Self {
        Self {
            kind: TableKind::Table,
            key_values: Default::default(),
            range: tombi_text::Range::default(),
            symbol_range: tombi_text::Range::default(),
            comment_directives: None,
            inner_comment_directives: None,
        }
    }

    pub(crate) fn new_root(node: &tombi_ast::Root) -> Self {
        Self {
            kind: TableKind::Root,
            key_values: Default::default(),
            range: node.syntax().range(),
            symbol_range: node.syntax().range(),
            comment_directives: None,
            inner_comment_directives: None,
        }
    }

    pub(crate) fn new_table(node: &tombi_ast::Table) -> Self {
        Self {
            kind: TableKind::Table,
            key_values: Default::default(),
            range: node.syntax().range(),
            symbol_range: tombi_text::Range::new(
                node.bracket_start()
                    .map(|bracket| bracket.range().start)
                    .unwrap_or_else(|| node.range().start),
                node.range().end,
            ),
            comment_directives: None,
            inner_comment_directives: None,
        }
    }

    pub(crate) fn new_array_of_table(node: &tombi_ast::ArrayOfTable) -> Self {
        Self {
            kind: TableKind::Table,
            key_values: Default::default(),
            range: node.syntax().range(),
            symbol_range: tombi_text::Range::new(
                node.double_bracket_start()
                    .map(|bracket| bracket.range().start)
                    .unwrap_or_else(|| node.range().start),
                node.range().end,
            ),
            comment_directives: None,
            inner_comment_directives: None,
        }
    }

    pub(crate) fn new_inline_table(node: &tombi_ast::InlineTable) -> Self {
        let has_comment = !node.inner_begin_dangling_comments().is_empty()
            || node
                .inner_end_dangling_comments()
                .into_iter()
                .flatten()
                .next()
                .is_some()
            || node.has_inner_comments();

        let symbol_range = tombi_text::Range::new(
            node.brace_start()
                .map_or_else(|| node.range().start, |brace| brace.range().start),
            node.brace_end()
                .map_or_else(|| node.range().end, |brace| brace.range().end),
        );

        Self {
            kind: TableKind::InlineTable { has_comment },
            key_values: Default::default(),
            range: node.syntax().range(),
            symbol_range,
            comment_directives: None,
            inner_comment_directives: None,
        }
    }

    pub(crate) fn new_key_value(node: &tombi_ast::KeyValue) -> Self {
        Self {
            kind: TableKind::KeyValue,
            key_values: Default::default(),
            range: node.syntax().range(),
            symbol_range: node.syntax().range(),
            comment_directives: None,
            inner_comment_directives: None,
        }
    }

    pub(crate) fn new_parent_table(&self) -> Self {
        Self {
            kind: TableKind::ParentTable,
            key_values: Default::default(),
            range: self.range,
            symbol_range: self.symbol_range,
            comment_directives: self.comment_directives.clone(),
            inner_comment_directives: None,
        }
    }

    pub(crate) fn new_parent_key(&self, parent_key: &Key) -> Self {
        Self {
            kind: TableKind::ParentKey,
            key_values: Default::default(),
            range: tombi_text::Range::new(parent_key.range().start, self.range.end),
            symbol_range: tombi_text::Range::new(parent_key.range().start, self.symbol_range.end),
            comment_directives: parent_key.comment_directives.clone(),
            inner_comment_directives: None,
        }
    }

    #[inline]
    pub fn comment_directives(&self) -> Option<&[TombiValueCommentDirective]> {
        self.comment_directives.as_deref()
    }

    #[inline]
    pub fn inner_comment_directives(&self) -> Option<&[TombiValueCommentDirective]> {
        self.inner_comment_directives.as_deref()
    }

    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.key_values.contains_key(key)
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.key_values.keys()
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.key_values.values()
    }

    #[inline]
    pub fn key_values(&self) -> &IndexMap<Key, Value> {
        &self.key_values
    }

    pub fn merge(&mut self, other: Self) -> Result<(), Vec<crate::Error>> {
        use TableKind::*;

        let mut errors = vec![];

        let mut is_conflict = false;
        match (self.kind, other.kind) {
            (KeyValue, KeyValue) => {
                for (self_key, self_value) in self.key_values() {
                    if let Some(other_value) = other.key_values.get(self_key) {
                        if match (self_value, other_value) {
                            (Value::Table(table1), _) => {
                                matches!(table1.kind(), TableKind::InlineTable { .. })
                            }
                            (_, Value::Table(table2)) => {
                                matches!(table2.kind(), TableKind::InlineTable { .. })
                            }
                            _ => false,
                        } {
                            is_conflict = true;
                            break;
                        }
                    }
                }
            }
            (Table | InlineTable { .. } | KeyValue, Table | InlineTable { .. })
            | (InlineTable { .. }, ParentTable | ParentKey | KeyValue)
            | (ParentTable, ParentKey) => {
                is_conflict = true;
            }
            (ParentTable, Table | InlineTable { .. }) => {
                self.kind = other.kind;
            }
            (ParentKey, Table | InlineTable { .. }) => {
                self.kind = other.kind;
                is_conflict = true;
            }
            _ => {}
        }

        if is_conflict {
            errors.push(crate::Error::ConflictTable {
                range1: self.symbol_range,
                range2: other.symbol_range,
            });
            return Err(errors);
        }

        self.range += other.range;
        self.symbol_range += other.symbol_range;

        // Merge the key_values of the two tables recursively
        for (key, value2) in other.key_values {
            match self.key_values.entry(key.clone()) {
                Entry::Occupied(mut entry) => {
                    let value1 = entry.get_mut();
                    match (value1, value2) {
                        (Value::Table(table1), Value::Table(table2)) => {
                            if let Err(errs) = table1.merge(table2) {
                                errors.extend(errs);
                            };
                        }
                        (Value::Array(array1), Value::Array(array2)) => {
                            if let Err(errs) = array1.merge(array2) {
                                errors.extend(errs);
                            }
                        }
                        _ => {
                            let range = key.range();
                            errors.push(crate::Error::DuplicateKey {
                                key: key.value,
                                range,
                            });
                        }
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(value2);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub(crate) fn insert(mut self, key: Key, value: Value) -> Result<Self, Vec<crate::Error>> {
        let mut errors = Vec::new();

        match self.key_values.entry(key) {
            Entry::Occupied(mut entry) => {
                let existing_value = entry.get_mut();
                match (existing_value, value) {
                    (Value::Table(table1), Value::Table(table2)) => {
                        if let Err(errs) = table1.merge(table2) {
                            errors.extend(errs);
                        }
                    }
                    (Value::Array(array1), Value::Array(array2)) => {
                        if let Err(errs) = array1.merge(array2) {
                            errors.extend(errs);
                        }
                    }
                    _ => {
                        errors.push(crate::Error::DuplicateKey {
                            key: entry.key().value.to_string(),
                            range: entry.key().range(),
                        });
                    }
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }

        if errors.is_empty() {
            Ok(self)
        } else {
            Err(errors)
        }
    }

    pub fn entry(&mut self, key: Key) -> Entry<'_, Key, Value> {
        self.key_values.entry(key)
    }

    pub fn get<K>(&self, key: &K) -> Option<&Value>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values.get(key)
    }

    pub fn get_mut<K>(&mut self, key: &K) -> Option<&mut Value>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values.get_mut(key)
    }

    pub fn get_key_value<K>(&self, key: &K) -> Option<(&Key, &Value)>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values.get_key_value(key)
    }

    pub fn get_key_value_mut<K>(&mut self, key: &K) -> Option<(&Key, &mut Value)>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values
            .get_full_mut(key)
            .map(|(_, key, value)| (key, value))
    }

    pub fn get_full<K>(&self, key: &K) -> Option<(usize, &Key, &Value)>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values.get_full(key)
    }

    pub fn get_full_mut<K>(&mut self, key: &K) -> Option<(usize, &Key, &mut Value)>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values.get_full_mut(key)
    }

    pub fn get_index_of<K>(&self, key: &K) -> Option<usize>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values.get_index_of(key)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Key, &mut Value)> {
        self.key_values.iter_mut()
    }

    pub fn iter_mut2(&mut self) -> impl Iterator<Item = (&mut Key, &mut Value)> {
        self.key_values.iter_mut2()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.key_values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.key_values.is_empty()
    }

    #[inline]
    pub fn kind(&self) -> TableKind {
        self.kind
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.range
    }

    #[inline]
    pub fn symbol_range(&self) -> tombi_text::Range {
        self.symbol_range
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ {} }}",
            self.key_values
                .iter()
                .filter_map(|(k, v)| if let crate::Value::Incomplete { .. } = &v {
                    None
                } else {
                    Some(format!("{} = {}", k, v))
                })
                .join(", ")
        )
    }
}

impl From<Table> for IndexMap<Key, Value> {
    fn from(table: Table) -> IndexMap<Key, Value> {
        table.key_values
    }
}

impl ValueImpl for Table {
    fn value_type(&self) -> ValueType {
        ValueType::Table
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Table> for tombi_ast::Table {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Table> {
        let mut table = Table::new_table(&self);
        let mut errors = vec![];
        let key_values = self.key_values().collect_vec();

        {
            let mut comment_directives = vec![];
            let mut inner_comment_directives = vec![];

            for comment in self.header_leading_comments() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }

                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    comment_directives.push(comment_directive);
                }
            }

            if let Some(comment) = self.header_trailing_comment() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    comment_directives.push(comment_directive);
                }
            }

            if key_values.is_empty() {
                for comments in self.key_values_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }
            } else {
                for comments in self.key_values_begin_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }

                for comments in self.key_values_end_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }
            }

            if !comment_directives.is_empty() {
                table.comment_directives = Some(comment_directives);
            }
            if !inner_comment_directives.is_empty() {
                table.inner_comment_directives = Some(inner_comment_directives);
            }
        }

        let empty_table = table.clone();

        let Some(header_keys) = self.header() else {
            errors.push(crate::Error::IncompleteNode {
                range: self.range(),
            });
            return DocumentTreeAndErrors {
                tree: empty_table,
                errors,
            };
        };

        let (mut header_keys, errs) = header_keys
            .into_document_tree_and_errors(toml_version)
            .into();
        if !errs.is_empty() {
            errors.extend(errs);
        }

        for key_value in key_values {
            let (other, errs) = key_value.into_document_tree_and_errors(toml_version).into();
            if !errs.is_empty() {
                errors.extend(errs);
            }
            if let Err(errs) = table.merge(other) {
                errors.extend(errs)
            }
        }

        let array_of_table_keys = get_array_of_tables_keys(
            self.parent_array_of_tables_keys(toml_version),
            toml_version,
            &mut errors,
        );

        let mut is_array_of_table = false;
        while let Some(mut key) = header_keys.pop() {
            key.comment_directives = table.comment_directives.clone();
            if is_array_of_table {
                if let Err(errs) =
                    insert_array_of_tables(&mut table, key, Array::new_parent_array_of_tables)
                {
                    errors.extend(errs);
                };
            } else if let Err(errs) = insert_table(&mut table, key) {
                errors.extend(errs);
            };

            is_array_of_table = array_of_table_keys.contains(&header_keys);
        }

        DocumentTreeAndErrors {
            tree: table,
            errors,
        }
    }
}

impl IntoDocumentTreeAndErrors<Table> for tombi_ast::ArrayOfTable {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<Table> {
        let mut table = Table::new_array_of_table(&self);
        let mut errors = vec![];
        let key_values = self.key_values().collect_vec();

        {
            let mut comment_directives = vec![];
            let mut inner_comment_directives = vec![];

            for comment in self.header_leading_comments() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    comment_directives.push(comment_directive);
                }
            }

            if let Some(comment) = self.header_trailing_comment() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    comment_directives.push(comment_directive);
                }
            }

            if key_values.is_empty() {
                for comments in self.key_values_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }
            } else {
                for comments in self.key_values_begin_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }

                for comments in self.key_values_end_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }
            }

            if !comment_directives.is_empty() {
                table.comment_directives = Some(comment_directives);
            }
            if !inner_comment_directives.is_empty() {
                table.inner_comment_directives = Some(inner_comment_directives);
            }
        }

        let empty_table = table.clone();

        let Some(header_keys) = self.header() else {
            errors.push(crate::Error::IncompleteNode {
                range: self.range(),
            });

            return DocumentTreeAndErrors {
                tree: empty_table,
                errors,
            };
        };

        let (mut header_keys, errs) = header_keys
            .into_document_tree_and_errors(toml_version)
            .into();

        if !errs.is_empty() {
            errors.extend(errs);
        }

        for key_value in key_values {
            let (other, errs) = key_value.into_document_tree_and_errors(toml_version).into();
            if !errs.is_empty() {
                errors.extend(errs);
            }
            if let Err(errs) = table.merge(other) {
                errors.extend(errs)
            }
        }

        let array_of_table_keys = get_array_of_tables_keys(
            self.parrent_array_of_tables_keys(),
            toml_version,
            &mut errors,
        );

        if let Some(mut key) = header_keys.pop() {
            key.comment_directives = table.comment_directives.clone();
            if let Err(errs) = insert_array_of_tables(&mut table, key, Array::new_array_of_tables) {
                errors.extend(errs);
            }
        }

        let mut is_array_of_table = array_of_table_keys.contains(&header_keys);
        while let Some(key) = header_keys.pop() {
            if is_array_of_table {
                if let Err(errs) =
                    insert_array_of_tables(&mut table, key, Array::new_parent_array_of_tables)
                {
                    errors.extend(errs);
                };
            } else if let Err(errs) = insert_table(&mut table, key) {
                errors.extend(errs);
            };

            is_array_of_table = array_of_table_keys.contains(&header_keys);
        }

        DocumentTreeAndErrors {
            tree: table,
            errors,
        }
    }
}

impl IntoDocumentTreeAndErrors<Table> for tombi_ast::TableOrArrayOfTable {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<Table> {
        match self {
            tombi_ast::TableOrArrayOfTable::Table(table) => {
                table.into_document_tree_and_errors(toml_version)
            }
            tombi_ast::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                array_of_table.into_document_tree_and_errors(toml_version)
            }
        }
    }
}

impl IntoDocumentTreeAndErrors<Table> for tombi_ast::KeyValue {
    fn into_document_tree_and_errors(
        self,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> DocumentTreeAndErrors<Table> {
        let mut table = Table::new_key_value(&self);
        let mut errors = Vec::new();

        let mut comment_directives = vec![];

        for comment in self.leading_comments() {
            if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                errors.push(error);
            }
            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                comment_directives.push(comment_directive);
            }
        }

        let empty_table = table.clone();

        let Some(keys) = self.keys() else {
            errors.push(crate::Error::IncompleteNode {
                range: self.range(),
            });
            return DocumentTreeAndErrors {
                tree: empty_table,
                errors,
            };
        };

        let (mut keys, errs) = keys.into_document_tree_and_errors(toml_version).into();
        if !errs.is_empty() {
            errors.extend(errs);
        }

        let value = match self.value() {
            Some(value) => {
                let (mut value, errs) = value.into_document_tree_and_errors(toml_version).into();
                if !errs.is_empty() {
                    errors.extend(errs);
                }

                if let Some(value_comment_directives) = value.comment_directives() {
                    comment_directives.extend(value_comment_directives.iter().cloned());
                }

                if !comment_directives.is_empty() {
                    table.comment_directives = Some(comment_directives.clone());
                    value.set_comment_directives(comment_directives.clone());
                }

                value
            }
            None => {
                errors.push(crate::Error::IncompleteNode {
                    range: table.range(),
                });
                Value::Incomplete {
                    range: tombi_text::Range::at(self.range().end),
                }
            }
        };

        let mut table = if let Some(mut key) = keys.pop() {
            table.range = key.range() + value.range();
            table.symbol_range = key.range() + value.symbol_range();
            if !comment_directives.is_empty() {
                key.comment_directives = Some(comment_directives.clone());
            }

            match table.insert(key, value) {
                Ok(t) => t,
                Err(errs) => {
                    errors.extend(errs);

                    return DocumentTreeAndErrors {
                        tree: empty_table,
                        errors,
                    };
                }
            }
        } else {
            return DocumentTreeAndErrors {
                tree: empty_table,
                errors,
            };
        };

        for mut key in keys.into_iter().rev() {
            let dummy_table = table.clone();
            if !comment_directives.is_empty() {
                key.comment_directives = Some(comment_directives.clone());
            }

            match table.new_parent_key(&key).insert(
                key,
                crate::Value::Table(std::mem::replace(&mut table, dummy_table)),
            ) {
                Ok(t) => table = t,
                Err(errs) => {
                    errors.extend(errs);
                }
            }
        }

        DocumentTreeAndErrors {
            tree: table,
            errors,
        }
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::InlineTable {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let mut table = Table::new_inline_table(&self);
        let table_kind = table.kind;
        let mut errors = vec![];
        let key_values = self.key_values().collect_vec();
        {
            let mut comment_directives = vec![];
            let mut inner_comment_directives = vec![];

            for comment in self.leading_comments() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    comment_directives.push(comment_directive);
                }
            }

            if key_values.is_empty() {
                for comments in self.inner_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }
            } else {
                for comments in self.inner_begin_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }

                for comments in self.inner_end_dangling_comments() {
                    for comment in comments {
                        if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                            errors.push(error);
                        }
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            inner_comment_directives.push(comment_directive);
                        }
                    }
                }
            }

            if let Some(comment) = self.trailing_comment() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    comment_directives.push(comment_directive);
                }
            }

            if !comment_directives.is_empty() {
                table.comment_directives = Some(comment_directives);
            }
            if !inner_comment_directives.is_empty() {
                table.inner_comment_directives = Some(inner_comment_directives);
            }
        }
        table.kind = TableKind::Table;

        for (key_value, comma) in self.key_values_with_comma() {
            let keys = key_value.keys().map(|k| k.keys());
            let (mut other, errs) = key_value.into_document_tree_and_errors(toml_version).into();

            if let Some(comma) = comma {
                let mut comment_directives = vec![];
                for comment in comma.leading_comments() {
                    if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                        errors.push(error);
                    }
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        comment_directives.push(comment_directive);
                    }
                }
                if let Some(comment) = comma.trailing_comment() {
                    if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                        errors.push(error);
                    }
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        comment_directives.push(comment_directive);
                    }
                }

                if !comment_directives.is_empty() {
                    if let Some(keys) = keys {
                        append_comment_directives(
                            &mut other,
                            keys.into_iter(),
                            &comment_directives,
                        );
                    }
                    other.comment_directives = Some(comment_directives);
                }
            }

            if !errs.is_empty() {
                errors.extend(errs)
            }
            if let Err(errs) = table.merge(other) {
                errors.extend(errs)
            }
        }

        table.kind = table_kind;

        DocumentTreeAndErrors {
            tree: crate::Value::Table(table),
            errors,
        }
    }
}

impl<T> IntoDocumentTreeAndErrors<crate::Table> for Vec<T>
where
    T: IntoDocumentTreeAndErrors<crate::Table>,
{
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Table> {
        let mut errors = Vec::new();
        let tables = self
            .into_iter()
            .map(|value| {
                let (table, errs) = value.into_document_tree_and_errors(toml_version).into();
                if !errs.is_empty() {
                    errors.extend(errs);
                }
                table
            })
            .collect_vec();

        let table = tables.into_iter().reduce(|mut acc, other| {
            if let Err(errs) = acc.merge(other) {
                errors.extend(errs);
            }
            acc
        });

        DocumentTreeAndErrors {
            tree: table.unwrap_or_else(Table::new_empty),
            errors,
        }
    }
}

impl IntoIterator for Table {
    type Item = (Key, Value);
    type IntoIter = indexmap::map::IntoIter<Key, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.key_values.into_iter()
    }
}

fn get_array_of_tables_keys(
    keys_iter: impl Iterator<Item = AstChildren<tombi_ast::Key>>,
    toml_version: TomlVersion,
    errors: &mut Vec<crate::Error>,
) -> Vec<Vec<Key>> {
    keys_iter
        .filter_map(|keys| {
            let mut new_keys = vec![];
            for key in keys {
                let (key, errs) = key.into_document_tree_and_errors(toml_version).into();
                if !errs.is_empty() {
                    errors.extend(errs);
                    return None;
                }
                if let Some(key) = key {
                    new_keys.push(key);
                }
            }
            Some(new_keys)
        })
        .unique()
        .collect_vec()
}

fn insert_table(table: &mut Table, key: Key) -> Result<(), Vec<crate::Error>> {
    let new_table = table.new_parent_table();
    match table
        .new_parent_table()
        .insert(key, Value::Table(std::mem::replace(table, new_table)))
    {
        Ok(t) => {
            *table = t;
            Ok(())
        }
        Err(errs) => Err(errs),
    }
}

fn insert_array_of_tables(
    table: &mut Table,
    key: Key,
    new_array_of_tables_fn: impl Fn(&Table) -> Array,
) -> Result<(), Vec<crate::Error>> {
    let mut array = new_array_of_tables_fn(table);
    let new_table = table.new_parent_table();
    array.push(Value::Table(std::mem::replace(table, new_table)));
    array.comment_directives = key.comment_directives.clone();
    match table.new_parent_table().insert(key, Value::Array(array)) {
        Ok(t) => {
            *table = t;
            Ok(())
        }
        Err(errors) => Err(errors),
    }
}

fn append_comment_directives(
    table: &mut Table,
    mut keys: impl Iterator<Item = tombi_ast::Key>,
    comment_directives: &Vec<TombiValueCommentDirective>,
) {
    // Get the next key in the path
    let Some(ast_key) = keys.next() else {
        // No more keys, append comment directives to the final table
        if let Some(table_comment_directives) = table.comment_directives.as_mut() {
            table_comment_directives.extend(comment_directives.iter().cloned());
        } else {
            table.comment_directives = Some(comment_directives.clone());
        }
        return;
    };

    // Since Table has only one key, we can directly access it without iteration
    let temp_key_values = std::mem::replace(&mut table.key_values, IndexMap::new());

    // Extract the single key-value pair
    let Some((mut key, value)) = temp_key_values.into_iter().next() else {
        return;
    };

    // Check if this is the key we're looking for
    if key == ast_key {
        // Update the key's comment directives
        if let Some(key_comment_directives) = key.comment_directives.as_mut() {
            key_comment_directives.extend(comment_directives.iter().cloned());
        } else {
            key.comment_directives = Some(comment_directives.clone());
        }

        if let Value::Table(mut nested_table) = value {
            // Recursively process the remaining keys
            append_comment_directives(&mut nested_table, keys, comment_directives);
            table.key_values.insert(key, Value::Table(nested_table));
        } else {
            // Put the value back if it's not a table
            table.key_values.insert(key, value);
        }
    } else {
        // Key doesn't match, put it back
        table.key_values.insert(key, value);
    }
}
