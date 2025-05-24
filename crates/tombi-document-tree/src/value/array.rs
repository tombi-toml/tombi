use tombi_ast::AstNode;

use crate::{
    support::comment::try_new_comment, DocumentTreeAndErrors, IntoDocumentTreeAndErrors, Value,
    ValueImpl, ValueType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrayKind {
    #[default]
    /// An array of tables.
    ///
    /// ```toml
    /// [[array]]
    /// ```
    ArrayOfTable,

    /// An array of tables of parent keys.
    ///
    /// ```toml
    /// [[fruit]]
    /// [fruit.info]
    /// #^^^^^                 <- Here
    ///
    /// [[fruit]]
    /// [[fruit.variables]]
    /// # ^^^^^                <- Here
    ///
    /// [fruit.variables.info]
    /// #^^^^^ ^^^^^^^^^       <- Here
    /// ```
    ParentArrayOfTable,

    /// An array.
    ///
    /// ```toml
    /// key = [1, 2, 3]
    /// ```
    Array,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Array {
    kind: ArrayKind,
    range: tombi_text::Range,
    symbol_range: tombi_text::Range,
    values: Vec<Value>,
}

impl Array {
    pub(crate) fn new_array(node: &tombi_ast::Array) -> Self {
        Self {
            kind: ArrayKind::Array,
            values: vec![],
            range: node.range(),
            symbol_range: match (node.bracket_start(), node.bracket_end()) {
                (Some(start), Some(end)) => {
                    tombi_text::Range::new(start.range().start, end.range().end)
                }
                _ => node.range(),
            },
        }
    }

    pub(crate) fn new_array_of_tables(table: &crate::Table) -> Self {
        Self {
            kind: ArrayKind::ArrayOfTable,
            values: vec![],
            range: table.range(),
            symbol_range: table.symbol_range(),
        }
    }

    pub(crate) fn new_parent_array_of_tables(table: &crate::Table) -> Self {
        Self {
            kind: ArrayKind::ParentArrayOfTable,
            values: vec![],
            range: table.range(),
            symbol_range: table.symbol_range(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.values.get_mut(index)
    }

    pub fn first(&self) -> Option<&Value> {
        self.values.first()
    }

    pub fn last(&self) -> Option<&Value> {
        self.values.last()
    }

    pub fn push(&mut self, value: Value) {
        self.range += value.range();
        self.symbol_range += value.symbol_range();

        self.values.push(value);
    }

    pub fn extend(&mut self, values: Vec<Value>) {
        for value in values {
            self.push(value);
        }
    }

    pub fn merge(&mut self, mut other: Self) -> Result<(), Vec<crate::Error>> {
        use ArrayKind::*;

        let mut errors = Vec::new();

        match (self.kind(), other.kind()) {
            (ArrayOfTable | ParentArrayOfTable, ParentArrayOfTable) => {
                let Some(Value::Table(table2)) = other.values.pop() else {
                    unreachable!("Parent of array of tables must have one table.")
                };
                if let Some(Value::Table(table1)) = self.values.last_mut() {
                    if let Err(errs) = table1.merge(table2) {
                        errors.extend(errs);
                    }
                } else {
                    self.push(Value::Table(table2));
                }
            }
            (ArrayOfTable | ParentArrayOfTable, ArrayOfTable) | (Array, Array) => {
                self.extend(other.values);
            }
            (Array, _) | (_, Array) => {
                errors.push(crate::Error::ConflictArray {
                    range1: self.symbol_range,
                    range2: other.symbol_range,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn kind(&self) -> ArrayKind {
        self.kind
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut Vec<Value> {
        &mut self.values
    }

    pub fn range(&self) -> tombi_text::Range {
        self.range
    }

    pub fn symbol_range(&self) -> tombi_text::Range {
        self.symbol_range
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.values.iter()
    }
}

impl ValueImpl for Array {
    fn value_type(&self) -> ValueType {
        ValueType::Array
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl IntoDocumentTreeAndErrors<crate::Value> for tombi_ast::Array {
    fn into_document_tree_and_errors(
        self,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> crate::DocumentTreeAndErrors<crate::Value> {
        let mut array = Array::new_array(&self);

        let mut errors = Vec::new();

        for comments in self.inner_begin_dangling_comments() {
            for comment in comments {
                if let Err(error) = try_new_comment(comment.as_ref()) {
                    errors.push(error);
                }
            }
        }

        for (value_or_key, comma) in self.value_or_key_values_with_commata() {
            match value_or_key {
                tombi_ast::ValueOrKeyValue::Value(value) => {
                    let (value, errs) = value.into_document_tree_and_errors(toml_version).into();
                    if !errs.is_empty() {
                        errors.extend(errs);
                    }
                    array.push(value);
                }
                tombi_ast::ValueOrKeyValue::KeyValue(key_value) => {
                    let (table, errs) =
                        key_value.into_document_tree_and_errors(toml_version).into();
                    if !errs.is_empty() {
                        errors.extend(errs);
                    }
                    array.push(crate::Value::Table(table));
                }
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

        for comments in self.inner_end_dangling_comments() {
            for comment in comments {
                if let Err(error) = try_new_comment(comment.as_ref()) {
                    errors.push(error);
                }
            }
        }

        DocumentTreeAndErrors {
            tree: crate::Value::Array(array),
            errors,
        }
    }
}

impl IntoIterator for Array {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}
