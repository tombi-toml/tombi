use indexmap::{map::Entry, IndexMap};
use itertools::Itertools;
use tombi_ast::{AstChildren, AstNode};
use tombi_toml_version::TomlVersion;

use crate::{
    support::comment::try_new_comment, Array, DocumentTreeAndErrors, IntoDocumentTreeAndErrors,
    Key, Value, ValueImpl, ValueType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    Root,
    Table,
    ParentTable,
    InlineTable,
    ParentKey,
    KeyValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    kind: TableKind,
    range: tombi_text::Range,
    symbol_range: tombi_text::Range,
    key_values: IndexMap<Key, Value>,
}

impl Table {
    pub(crate) fn new_empty() -> Self {
        Self {
            kind: TableKind::Table,
            key_values: Default::default(),
            range: tombi_text::Range::default(),
            symbol_range: tombi_text::Range::default(),
        }
    }

    pub(crate) fn new_root(node: &tombi_ast::Root) -> Self {
        Self {
            kind: TableKind::Root,
            key_values: Default::default(),
            range: node.syntax().range(),
            symbol_range: node.syntax().range(),
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
        }
    }

    pub(crate) fn new_inline_table(node: &tombi_ast::InlineTable) -> Self {
        Self {
            kind: TableKind::InlineTable,
            key_values: Default::default(),
            range: node.syntax().range(),
            symbol_range: tombi_text::Range::new(
                node.brace_start()
                    .map(|brace| brace.range().start)
                    .unwrap_or_else(|| node.range().start),
                node.range().end,
            ),
        }
    }

    pub(crate) fn new_key_value(node: &tombi_ast::KeyValue) -> Self {
        Self {
            kind: TableKind::KeyValue,
            key_values: Default::default(),
            range: node.syntax().range(),
            symbol_range: node.syntax().range(),
        }
    }

    pub(crate) fn new_parent_table(&self) -> Self {
        Self {
            kind: TableKind::ParentTable,
            key_values: Default::default(),
            range: self.range,
            symbol_range: self.symbol_range,
        }
    }

    pub(crate) fn new_parent_key(&self) -> Self {
        Self {
            kind: TableKind::ParentKey,
            key_values: Default::default(),
            range: self.range,
            symbol_range: self.symbol_range,
        }
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

        let is_conflict = match (self.kind, other.kind) {
            (KeyValue, KeyValue) => {
                match self
                    .key_values
                    .values()
                    .zip(other.key_values.values())
                    .next()
                {
                    Some((Value::Table(table1), _)) => table1.kind() == InlineTable,
                    Some((_, Value::Table(table2))) => table2.kind() == InlineTable,
                    Some(_) => false,
                    None => unreachable!("KeyValue must have one value."),
                }
            }
            (Table | InlineTable | KeyValue, Table | InlineTable)
            | (InlineTable, ParentTable | ParentKey | KeyValue)
            | (ParentTable, ParentKey) => true,
            (ParentTable, Table | InlineTable) => {
                self.kind = other.kind;
                false
            }
            (ParentKey, Table | InlineTable) => {
                self.kind = other.kind;
                true
            }
            _ => false,
        };

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
                            errors.push(crate::Error::DuplicateKey {
                                key: key.value().to_string(),
                                range: key.range(),
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
                            key: entry.key().value().to_string(),
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

    pub fn get_key_value<K>(&self, key: &K) -> Option<(&Key, &Value)>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values.get_key_value(key)
    }

    pub fn get_mut<K>(&mut self, key: &K) -> Option<&mut Value>
    where
        K: ?Sized + std::hash::Hash + indexmap::Equivalent<Key>,
    {
        self.key_values.get_mut(key)
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
        let mut errors = Vec::new();

        for comment in self.header_leading_comments() {
            if let Err(error) = try_new_comment(comment.as_ref()) {
                errors.push(error);
            }
        }

        if let Some(comment) = self.header_tailing_comment() {
            if let Err(error) = try_new_comment(comment.as_ref()) {
                errors.push(error);
            }
        }

        let Some(header_keys) = self.header() else {
            errors.push(crate::Error::IncompleteNode {
                range: self.range(),
            });
            return DocumentTreeAndErrors {
                tree: table,
                errors,
            };
        };

        let (mut header_keys, errs) = header_keys
            .into_document_tree_and_errors(toml_version)
            .into();
        if !errs.is_empty() {
            errors.extend(errs);
            return make_keys_table(header_keys, table, errors);
        }

        for key_value in self.key_values() {
            let (other, errs) = key_value.into_document_tree_and_errors(toml_version).into();
            if !errs.is_empty() {
                errors.extend(errs);
            }
            if let Err(errs) = table.merge(other) {
                errors.extend(errs)
            }
        }

        let array_of_table_keys =
            get_array_of_tables_keys(self.array_of_tables_keys(), toml_version, &mut errors);

        let mut is_array_of_table = false;
        while let Some(key) = header_keys.pop() {
            if is_array_of_table {
                if let Err(errs) =
                    insert_array_of_tables(&mut table, key, Array::new_parent_array_of_tables)
                {
                    errors.extend(errs);
                    return make_keys_table(header_keys, table, errors);
                };
            } else if let Err(errs) =
                insert_table(&mut table, key, |table| table.new_parent_table())
            {
                errors.extend(errs);
                return make_keys_table(header_keys, table, errors);
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
        let mut errors = Vec::new();

        for comment in self.header_leading_comments() {
            if let Err(error) = try_new_comment(comment.as_ref()) {
                errors.push(error);
            }
        }

        if let Some(comment) = self.header_tailing_comment() {
            if let Err(error) = try_new_comment(comment.as_ref()) {
                errors.push(error);
            }
        }

        let Some(header_keys) = self.header() else {
            errors.push(crate::Error::IncompleteNode {
                range: self.range(),
            });
            return DocumentTreeAndErrors {
                tree: table,
                errors,
            };
        };

        let (mut header_keys, errs) = header_keys
            .into_document_tree_and_errors(toml_version)
            .into();
        if !errs.is_empty() {
            errors.extend(errs);
            return make_keys_table(header_keys, table, errors);
        }

        for key_value in self.key_values() {
            let (other, errs) = key_value.into_document_tree_and_errors(toml_version).into();
            if !errs.is_empty() {
                errors.extend(errs);
            }
            if let Err(errs) = table.merge(other) {
                errors.extend(errs)
            }
        }

        let array_of_table_keys =
            get_array_of_tables_keys(self.array_of_tables_keys(), toml_version, &mut errors);

        if let Some(key) = header_keys.pop() {
            if let Err(errs) = insert_array_of_tables(&mut table, key, Array::new_array_of_tables) {
                errors.extend(errs);
                return make_keys_table(header_keys, table, errors);
            }
        }

        let mut is_array_of_table = array_of_table_keys.contains(&header_keys);
        while let Some(key) = header_keys.pop() {
            if is_array_of_table {
                if let Err(errs) =
                    insert_array_of_tables(&mut table, key, Array::new_parent_array_of_tables)
                {
                    errors.extend(errs);
                    return make_keys_table(header_keys, table, errors);
                };
            } else if let Err(errs) =
                insert_table(&mut table, key, |table| table.new_parent_table())
            {
                errors.extend(errs);
                return make_keys_table(header_keys, table, errors);
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
        let table = Table::new_key_value(&self);
        let mut errors = Vec::new();

        for comment in self.leading_comments() {
            if let Err(error) = try_new_comment(comment.as_ref()) {
                errors.push(error);
            }
        }

        let Some(keys) = self.keys() else {
            errors.push(crate::Error::IncompleteNode {
                range: self.range(),
            });
            return DocumentTreeAndErrors {
                tree: table,
                errors,
            };
        };

        let (mut keys, errs) = keys.into_document_tree_and_errors(toml_version).into();
        if !errs.is_empty() {
            errors.extend(errs);
            return make_keys_table(keys, table, errors);
        }

        let value = match self.value() {
            Some(value) => {
                let (value, errs) = value.into_document_tree_and_errors(toml_version).into();
                if !errs.is_empty() {
                    errors.extend(errs);
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

        let table = if let Some(key) = keys.pop() {
            match Table::new_key_value(&self).insert(key, value) {
                Ok(table) => table,
                Err(errs) => {
                    errors.extend(errs);
                    return make_keys_table(keys, table, errors);
                }
            }
        } else {
            return make_keys_table(keys, table, errors);
        };

        make_keys_table(keys, table, errors)
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::InlineTable {
    fn into_document_tree_and_errors(
        self,
        toml_version: TomlVersion,
    ) -> DocumentTreeAndErrors<crate::Value> {
        let mut table = Table::new_inline_table(&self);
        let mut errors = Vec::new();

        for comments in self.inner_begin_dangling_comments() {
            for comment in comments {
                if let Err(error) = try_new_comment(comment.as_ref()) {
                    errors.push(error);
                }
            }
        }

        table.kind = TableKind::Table;

        for (key_value, comma) in self.key_values_with_comma() {
            let (other, errs) = key_value.into_document_tree_and_errors(toml_version).into();

            if !errs.is_empty() {
                errors.extend(errs)
            }
            if let Err(errs) = table.merge(other) {
                errors.extend(errs)
            }

            if let Some(comma) = comma {
                for comment in comma.leading_comments() {
                    if let Err(error) = try_new_comment(comment.as_ref()) {
                        errors.push(error);
                    }
                }
                if let Some(comment) = comma.tailing_comment() {
                    if let Err(error) = try_new_comment(comment.as_ref()) {
                        errors.push(error);
                    }
                }
            }
        }

        table.kind = TableKind::InlineTable;

        for comments in self.inner_end_dangling_comments() {
            for comment in comments {
                if let Err(error) = try_new_comment(comment.as_ref()) {
                    errors.push(error);
                }
            }
        }

        DocumentTreeAndErrors {
            tree: crate::Value::Table(table),
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

fn insert_table(
    table: &mut Table,
    key: Key,
    new_table_fn: impl Fn(&Table) -> Table,
) -> Result<(), Vec<crate::Error>> {
    let new_table = new_table_fn(table);
    match new_table_fn(table).insert(key, Value::Table(std::mem::replace(table, new_table))) {
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
    match table.new_parent_table().insert(key, Value::Array(array)) {
        Ok(t) => {
            *table = t;
            Ok(())
        }
        Err(errors) => Err(errors),
    }
}

fn make_keys_table(
    keys: Vec<crate::Key>,
    mut table: crate::Table,
    mut errors: Vec<crate::Error>,
) -> DocumentTreeAndErrors<crate::Table> {
    for key in keys.into_iter().rev() {
        let dummy_table = table.clone();
        match table.new_parent_key().insert(
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
