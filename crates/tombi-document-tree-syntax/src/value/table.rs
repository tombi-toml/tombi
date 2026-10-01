use itertools::Itertools;
use tombi_ast_syntax::{AstNode, TombiValueCommentDirective};

use crate::{
    Array, ArrayKind, DocumentTreeAndErrors, IntoDocumentTreeWithContext, Key, Value, ValueImpl,
    ValueType,
};

use tombi_document_tree::TableKind;

#[derive(Debug, Clone, PartialEq)]
pub struct Table<'t> {
    kind: TableKind,
    span: tombi_text::Span,
    symbol_span: tombi_text::Span,
    key_values: tombi_hashmap::IndexMap<Key<'t>, Value<'t>>,
    pub(crate) header_comment_directives: Option<Vec<TombiValueCommentDirective>>,
    pub(crate) body_comment_directives: Option<Vec<TombiValueCommentDirective>>,
    pub(crate) group_boundary_comment_directives: Option<Vec<TombiValueCommentDirective>>,
}

impl<'t> Table<'t> {
    pub(crate) fn new_empty() -> Self {
        Self {
            kind: TableKind::Table,
            key_values: Default::default(),
            span: tombi_text::Span::default(),
            symbol_span: tombi_text::Span::default(),
            header_comment_directives: None,
            body_comment_directives: None,
            group_boundary_comment_directives: None,
        }
    }

    pub(crate) fn new_root(node: &tombi_ast_syntax::Root<'t>) -> Self {
        Self {
            kind: TableKind::Root,
            key_values: Default::default(),
            span: node.syntax().span(),
            symbol_span: node.syntax().span(),
            header_comment_directives: None,
            body_comment_directives: None,
            group_boundary_comment_directives: None,
        }
    }

    pub(crate) fn new_table(node: &tombi_ast_syntax::Table<'t>) -> Self {
        Self {
            kind: TableKind::Table,
            key_values: Default::default(),
            span: node.syntax().span(),
            symbol_span: tombi_text::Span::new(
                node.bracket_start()
                    .map(|bracket| bracket.span().start)
                    .unwrap_or_else(|| node.span().start),
                node.span().end,
            ),
            header_comment_directives: None,
            body_comment_directives: None,
            group_boundary_comment_directives: None,
        }
    }

    pub(crate) fn new_array_of_table(node: &tombi_ast_syntax::ArrayOfTable<'t>) -> Self {
        Self {
            kind: TableKind::Table,
            key_values: Default::default(),
            span: node.syntax().span(),
            symbol_span: tombi_text::Span::new(
                node.double_bracket_start()
                    .map(|bracket| bracket.span().start)
                    .unwrap_or_else(|| node.span().start),
                node.span().end,
            ),
            header_comment_directives: None,
            body_comment_directives: None,
            group_boundary_comment_directives: None,
        }
    }

    pub(crate) fn new_inline_table(node: &tombi_ast_syntax::InlineTable<'t>) -> Self {
        let has_comment = node.brace_start_trailing_comment().is_some()
            || node.dangling_comment_groups().next().is_some()
            || node.has_inner_comments();

        let symbol_span = tombi_text::Span::new(
            node.brace_start()
                .map_or_else(|| node.span().start, |brace| brace.span().start),
            node.brace_end()
                .map_or_else(|| node.span().end, |brace| brace.span().end),
        );

        Self {
            kind: TableKind::InlineTable { has_comment },
            key_values: Default::default(),
            span: node.syntax().span(),
            symbol_span,
            header_comment_directives: None,
            body_comment_directives: None,
            group_boundary_comment_directives: None,
        }
    }

    pub(crate) fn new_key_value(node: &tombi_ast_syntax::KeyValue<'t>) -> Self {
        Self {
            kind: TableKind::KeyValue,
            key_values: Default::default(),
            span: node.syntax().span(),
            symbol_span: node.syntax().span(),
            header_comment_directives: None,
            body_comment_directives: None,
            group_boundary_comment_directives: None,
        }
    }

    pub(crate) fn new_parent_table(&self) -> Self {
        Self {
            kind: TableKind::ParentTable,
            key_values: Default::default(),
            span: self.span,
            symbol_span: self.symbol_span,
            header_comment_directives: self.header_comment_directives.clone(),
            body_comment_directives: None,
            group_boundary_comment_directives: None,
        }
    }

    pub(crate) fn new_parent_key(&self, parent_key: &Key<'t>) -> Self {
        Self {
            kind: TableKind::ParentKey,
            key_values: Default::default(),
            span: tombi_text::Span::new(parent_key.span().start, self.span.end),
            symbol_span: tombi_text::Span::new(parent_key.span().start, self.symbol_span.end),
            header_comment_directives: parent_key.comment_directives.clone(),
            body_comment_directives: None,
            group_boundary_comment_directives: None,
        }
    }

    #[inline]
    pub fn comment_directives(
        &self,
    ) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        if self.header_comment_directives.is_none() && self.body_comment_directives.is_none() {
            None
        } else {
            Some(itertools::chain!(
                self.header_comment_directives.iter().flatten(),
                self.body_comment_directives.iter().flatten()
            ))
        }
    }

    #[inline]
    pub fn header_comment_directives(
        &self,
    ) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.header_comment_directives.as_deref().map(|d| d.iter())
    }

    #[inline]
    pub fn body_comment_directives(
        &self,
    ) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.body_comment_directives.as_deref().map(|d| d.iter())
    }

    #[inline]
    pub fn group_boundary_comment_directives(
        &self,
    ) -> Option<impl Iterator<Item = &TombiValueCommentDirective> + '_> {
        self.group_boundary_comment_directives
            .as_deref()
            .map(|d| d.iter())
    }

    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.key_values.contains_key(key)
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &Key<'t>> {
        self.key_values.keys()
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Value<'t>> {
        self.key_values.values()
    }

    #[inline]
    pub fn key_values(&self) -> &tombi_hashmap::IndexMap<Key<'t>, Value<'t>> {
        &self.key_values
    }

    pub fn merge(&mut self, other: Self) -> Result<(), Vec<crate::Error>> {
        use TableKind::*;

        let mut errors = vec![];

        let mut is_conflict = false;
        match (self.kind, other.kind) {
            (KeyValue, KeyValue) => {
                // Iterate `other` and look up in `self`: `other` is typically small
                // (a single key-value) while `self` can be the huge root table.
                for (other_key, other_value) in other.key_values() {
                    if let Some(self_value) = self.key_values.get(other_key)
                        && match (self_value, other_value) {
                            (Value::Table(table1), _) => {
                                matches!(table1.kind(), TableKind::InlineTable { .. })
                            }
                            (_, Value::Table(table2)) => {
                                matches!(table2.kind(), TableKind::InlineTable { .. })
                            }
                            _ => false,
                        }
                    {
                        is_conflict = true;
                        break;
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
                range1: self.symbol_span,
                range2: other.symbol_span,
            });
            return Err(errors);
        }

        self.span += other.span;
        self.symbol_span += other.symbol_span;

        // Merge the key_values of the two tables recursively
        for (key, value2) in other.key_values {
            match self.key_values.entry(key.clone()) {
                tombi_hashmap::map::Entry::Occupied(mut entry) => {
                    let value1 = entry.get_mut();
                    match (value1, value2) {
                        (Value::Table(table1), Value::Table(table2)) => {
                            if let Err(errs) = table1.merge(table2) {
                                errors.extend(errs);
                            };
                        }
                        (Value::Array(array1), Value::Array(array2))
                            if can_merge_arrays(array1, &array2) =>
                        {
                            if let Err(errs) = array1.merge(array2) {
                                errors.extend(errs);
                            }
                        }
                        _ => {
                            let span = key.span();
                            errors.push(crate::Error::DuplicateKey {
                                key: key.value.into_string(),
                                span,
                            });
                        }
                    }
                }
                tombi_hashmap::map::Entry::Vacant(entry) => {
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

    pub(crate) fn insert(
        mut self,
        key: Key<'t>,
        value: Value<'t>,
    ) -> Result<Self, Vec<crate::Error>> {
        let mut errors = Vec::new();

        match self.key_values.entry(key) {
            tombi_hashmap::map::Entry::Occupied(mut entry) => {
                let existing_value = entry.get_mut();
                match (existing_value, value) {
                    (Value::Table(table1), Value::Table(table2)) => {
                        if let Err(errs) = table1.merge(table2) {
                            errors.extend(errs);
                        }
                    }
                    (Value::Array(array1), Value::Array(array2))
                        if can_merge_arrays(array1, &array2) =>
                    {
                        if let Err(errs) = array1.merge(array2) {
                            errors.extend(errs);
                        }
                    }
                    _ => {
                        errors.push(crate::Error::DuplicateKey {
                            key: entry.key().value.to_string(),
                            span: entry.key().span(),
                        });
                    }
                }
            }
            tombi_hashmap::map::Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }

        if errors.is_empty() {
            Ok(self)
        } else {
            Err(errors)
        }
    }

    pub fn entry(&mut self, key: Key<'t>) -> tombi_hashmap::map::Entry<'_, Key<'t>, Value<'t>> {
        self.key_values.entry(key)
    }

    pub fn get<K>(&self, key: &K) -> Option<&Value<'t>>
    where
        K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key<'t>>,
    {
        self.key_values.get(key)
    }

    pub fn get_mut<K>(&mut self, key: &K) -> Option<&mut Value<'t>>
    where
        K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key<'t>>,
    {
        self.key_values.get_mut(key)
    }

    pub fn get_key_value<K>(&self, key: &K) -> Option<(&Key<'t>, &Value<'t>)>
    where
        K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key<'t>>,
    {
        self.key_values.get_key_value(key)
    }

    pub fn get_key_value_mut<K>(&mut self, key: &K) -> Option<(&Key<'t>, &mut Value<'t>)>
    where
        K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key<'t>>,
    {
        self.key_values
            .get_full_mut(key)
            .map(|(_, key, value)| (key, value))
    }

    pub fn get_full<K>(&self, key: &K) -> Option<(usize, &Key<'t>, &Value<'t>)>
    where
        K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key<'t>>,
    {
        self.key_values.get_full(key)
    }

    pub fn get_full_mut<K>(&mut self, key: &K) -> Option<(usize, &Key<'t>, &mut Value<'t>)>
    where
        K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key<'t>>,
    {
        self.key_values.get_full_mut(key)
    }

    pub fn get_index_of<K>(&self, key: &K) -> Option<usize>
    where
        K: ?Sized + std::hash::Hash + tombi_hashmap::Equivalent<Key<'t>>,
    {
        self.key_values.get_index_of(key)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Key<'t>, &mut Value<'t>)> {
        self.key_values.iter_mut()
    }

    pub fn iter_mut2(&mut self) -> impl Iterator<Item = (&mut Key<'t>, &mut Value<'t>)> {
        tombi_hashmap::map::MutableKeys::iter_mut2(&mut self.key_values)
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
    pub fn span(&self) -> tombi_text::Span {
        self.span
    }

    #[inline]
    pub fn symbol_span(&self) -> tombi_text::Span {
        self.symbol_span
    }
}

#[inline]
fn can_merge_arrays<'t>(array1: &Array<'t>, array2: &Array<'t>) -> bool {
    array1.kind() != ArrayKind::Array && array2.kind() != ArrayKind::Array
}

impl<'t> std::fmt::Display for Table<'t> {
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

impl<'t> From<Table<'t>> for tombi_hashmap::IndexMap<Key<'t>, Value<'t>> {
    fn from(table: Table<'t>) -> tombi_hashmap::IndexMap<Key<'t>, Value<'t>> {
        table.key_values
    }
}

impl<'t> ValueImpl for Table<'t> {
    fn value_type(&self) -> ValueType {
        ValueType::Table
    }

    fn span(&self) -> tombi_text::Span {
        self.span()
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, crate::Table<'t>> for tombi_ast_syntax::Table<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Table<'t>> {
        let mut table = Table::new_table(&self);
        let mut errors = vec![];

        {
            let mut header_comment_directives = vec![];
            let mut body_comment_directives = vec![];

            for comment in self.header_leading_comments() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }

                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    header_comment_directives.push(comment_directive);
                }
            }

            if let Some(comment) = self.header_trailing_comment() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    header_comment_directives.push(comment_directive);
                }
            }

            for comment_group in self.dangling_comment_groups() {
                for comment in comment_group.comments() {
                    if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                        errors.push(error);
                    }
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        body_comment_directives.push(comment_directive);
                    }
                }
            }

            if !header_comment_directives.is_empty() {
                table.header_comment_directives = Some(header_comment_directives);
            }
            if !body_comment_directives.is_empty() {
                table.body_comment_directives = Some(body_comment_directives);
            }
        }

        let empty_table = table.clone();

        let Some(header_keys) = self.header() else {
            errors.push(crate::Error::IncompleteNode { span: self.span() });
            return DocumentTreeAndErrors {
                tree: empty_table,
                errors,
            };
        };

        let (mut header_keys, errs) = header_keys.into_document_tree_with_context(context).into();
        if !errs.is_empty() {
            errors.extend(errs);
        }

        {
            let mut group_boundary_comment_directives = Vec::new();
            for group in self.key_value_groups() {
                match group {
                    tombi_ast_syntax::DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                        for key_value in key_value_group.into_key_values() {
                            let (other, errs) =
                                key_value.into_document_tree_with_context(context).into();
                            if !errs.is_empty() {
                                errors.extend(errs);
                            }
                            if let Err(errs) = table.merge(other) {
                                errors.extend(errs)
                            }
                        }
                    }
                    tombi_ast_syntax::DanglingCommentGroupOr::DanglingCommentGroup(
                        comment_group,
                    ) => {
                        for comment in comment_group.comments() {
                            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                                group_boundary_comment_directives.push(comment_directive);
                            }
                        }
                    }
                }
            }
            if !group_boundary_comment_directives.is_empty() {
                table.group_boundary_comment_directives = Some(group_boundary_comment_directives);
            }
        }

        let parent_array_of_tables_counts = self.parent_array_of_tables_prefix_counts();

        let mut is_array_of_table = false;
        while let Some(mut key) = header_keys.pop() {
            key.comment_directives = table.header_comment_directives.clone();
            if is_array_of_table {
                if let Err(errs) =
                    insert_array_of_tables(&mut table, key, Array::new_parent_array_of_tables)
                {
                    errors.extend(errs);
                };
            } else if let Err(errs) = insert_table(&mut table, key) {
                errors.extend(errs);
            };

            is_array_of_table =
                is_parent_array_of_tables(&parent_array_of_tables_counts, header_keys.len());
        }

        DocumentTreeAndErrors {
            tree: table,
            errors,
        }
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, Table<'t>> for tombi_ast_syntax::ArrayOfTable<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<Table<'t>> {
        let mut table = Table::new_array_of_table(&self);
        let mut errors = vec![];

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

            for comment_group in self.dangling_comment_groups() {
                for comment in comment_group.comments() {
                    if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                        errors.push(error);
                    }
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        inner_comment_directives.push(comment_directive);
                    }
                }
            }

            if !comment_directives.is_empty() {
                table.header_comment_directives = Some(comment_directives);
            }
            if !inner_comment_directives.is_empty() {
                table.body_comment_directives = Some(inner_comment_directives);
            }
        }

        let empty_table = table.clone();

        let Some(header_keys) = self.header() else {
            errors.push(crate::Error::IncompleteNode { span: self.span() });

            return DocumentTreeAndErrors {
                tree: empty_table,
                errors,
            };
        };

        let (mut header_keys, errs) = header_keys.into_document_tree_with_context(context).into();

        if !errs.is_empty() {
            errors.extend(errs);
        }

        {
            let mut group_boundary_comment_directives = Vec::new();
            for group in self.key_value_groups() {
                match group {
                    tombi_ast_syntax::DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                        for key_value in key_value_group.into_key_values() {
                            let (other, errs) =
                                key_value.into_document_tree_with_context(context).into();
                            if !errs.is_empty() {
                                errors.extend(errs);
                            }
                            if let Err(errs) = table.merge(other) {
                                errors.extend(errs)
                            }
                        }
                    }
                    tombi_ast_syntax::DanglingCommentGroupOr::DanglingCommentGroup(
                        comment_group,
                    ) => {
                        for comment in comment_group.comments() {
                            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                                group_boundary_comment_directives.push(comment_directive);
                            }
                        }
                    }
                }
            }
            if !group_boundary_comment_directives.is_empty() {
                table.group_boundary_comment_directives = Some(group_boundary_comment_directives);
            }
        }

        let parent_array_of_tables_counts = self.parent_array_of_tables_prefix_counts();

        if let Some(mut key) = header_keys.pop() {
            key.comment_directives = table.header_comment_directives.take();
            if let Err(errs) = insert_array_of_tables(&mut table, key, Array::new_array_of_tables) {
                errors.extend(errs);
            }
        }

        let mut is_array_of_table =
            is_parent_array_of_tables(&parent_array_of_tables_counts, header_keys.len());
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

            is_array_of_table =
                is_parent_array_of_tables(&parent_array_of_tables_counts, header_keys.len());
        }

        DocumentTreeAndErrors {
            tree: table,
            errors,
        }
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, Table<'t>> for tombi_ast_syntax::TableOrArrayOfTable<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<Table<'t>> {
        match self {
            tombi_ast_syntax::TableOrArrayOfTable::Table(table) => {
                table.into_document_tree_with_context(context)
            }
            tombi_ast_syntax::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                array_of_table.into_document_tree_with_context(context)
            }
        }
    }
}

impl<'t> IntoDocumentTreeWithContext<'t, Table<'t>> for tombi_ast_syntax::KeyValue<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<Table<'t>> {
        let mut table = Table::new_key_value(&self);
        let mut errors = Vec::new();

        let mut header_comment_directives = vec![];

        for comment in self.leading_comments() {
            if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                errors.push(error);
            }
            if let Some(comment_directive) = comment.get_tombi_value_directive() {
                header_comment_directives.push(comment_directive);
            }
        }
        if !header_comment_directives.is_empty() {
            table.header_comment_directives = Some(header_comment_directives.clone());
        }

        let empty_table = table.clone();

        let Some(keys) = self.keys() else {
            errors.push(crate::Error::IncompleteNode { span: self.span() });
            return DocumentTreeAndErrors {
                tree: empty_table,
                errors,
            };
        };

        let (mut keys, errs) = keys.into_document_tree_with_context(context).into();
        if !errs.is_empty() {
            errors.extend(errs);
        }

        let mut body_comment_directives = vec![];
        let mut combined_comment_directives = vec![];
        let value = match self.value() {
            Some(value) => {
                let (mut value, errs) = value.into_document_tree_with_context(context).into();
                if !errs.is_empty() {
                    errors.extend(errs);
                }

                if let Some(directives) = value.comment_directives() {
                    body_comment_directives.extend(directives.cloned());
                }

                if !body_comment_directives.is_empty() {
                    table.body_comment_directives = Some(body_comment_directives.clone());
                }

                // Combine header (leading) + body (value's own) directives
                // for setting on both value and key, matching main branch behavior
                combined_comment_directives.extend(header_comment_directives.clone());
                combined_comment_directives.extend(body_comment_directives);
                if !combined_comment_directives.is_empty() {
                    value.set_comment_directives(combined_comment_directives.clone());
                }

                value
            }
            None => {
                errors.push(crate::Error::IncompleteNode { span: table.span() });
                Value::Incomplete {
                    span: tombi_text::Span::empty(self.span().end),
                }
            }
        };

        let mut table = if let Some(mut key) = keys.pop() {
            table.span = key.span() + value.span();
            table.symbol_span = key.span() + value.symbol_span();
            if !combined_comment_directives.is_empty() {
                key.comment_directives = Some(combined_comment_directives.clone());
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
            if !combined_comment_directives.is_empty() {
                key.comment_directives = Some(combined_comment_directives.clone());
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

impl<'t> IntoDocumentTreeWithContext<'t, crate::Value<'t>> for tombi_ast_syntax::InlineTable<'t> {
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Value<'t>> {
        let mut table = Table::new_inline_table(&self);
        let table_kind = table.kind;
        let mut errors = vec![];
        {
            let mut header_comment_directives = vec![];
            let mut body_comment_directives = vec![];

            for comment in self.leading_comments() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    body_comment_directives.push(comment_directive);
                }
            }

            if let Some(comment) = self.brace_start_trailing_comment() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    body_comment_directives.push(comment_directive);
                }
            }

            for comment_group in self.dangling_comment_groups() {
                for comment in comment_group.comments() {
                    if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                        errors.push(error);
                    }
                    if let Some(comment_directive) = comment.get_tombi_value_directive() {
                        body_comment_directives.push(comment_directive);
                    }
                }
            }

            if let Some(comment) = self.trailing_comment() {
                if let Err(error) = crate::support::comment::try_new_comment(&comment) {
                    errors.push(error);
                }
                if let Some(comment_directive) = comment.get_tombi_value_directive() {
                    header_comment_directives.push(comment_directive);
                }
            }
            if !header_comment_directives.is_empty() {
                table.header_comment_directives = Some(header_comment_directives);
            }
            if !body_comment_directives.is_empty() {
                table.body_comment_directives = Some(body_comment_directives);
            }
        }
        table.kind = TableKind::Table;

        let mut group_boundary_comment_directives = Vec::new();
        for group in self.key_value_with_comma_groups() {
            match group {
                tombi_ast_syntax::DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                    for comment in comment_group.comments() {
                        if let Some(comment_directive) = comment.get_tombi_value_directive() {
                            group_boundary_comment_directives.push(comment_directive);
                        }
                    }
                }
                tombi_ast_syntax::DanglingCommentGroupOr::ItemGroup(item_group) => {
                    for (key_value, comma) in item_group.key_values_with_comma() {
                        let keys = key_value.keys().map(|k| k.keys());
                        let (mut other, errs) =
                            key_value.into_document_tree_with_context(context).into();

                        if let Some(comma) = comma {
                            let mut header_comment_directives = vec![];
                            for comment in comma.leading_comments() {
                                if let Err(error) =
                                    crate::support::comment::try_new_comment(&comment)
                                {
                                    errors.push(error);
                                }
                                if let Some(comment_directive) = comment.get_tombi_value_directive()
                                {
                                    header_comment_directives.push(comment_directive);
                                }
                            }
                            if let Some(comment) = comma.trailing_comment() {
                                if let Err(error) =
                                    crate::support::comment::try_new_comment(&comment)
                                {
                                    errors.push(error);
                                }
                                if let Some(comment_directive) = comment.get_tombi_value_directive()
                                {
                                    header_comment_directives.push(comment_directive);
                                }
                            }

                            if !header_comment_directives.is_empty() {
                                if let Some(keys) = keys {
                                    append_header_comment_directives(
                                        &mut other,
                                        keys.into_iter(),
                                        &header_comment_directives,
                                    );
                                }
                                other.header_comment_directives = Some(header_comment_directives);
                            }
                        }

                        if !errs.is_empty() {
                            errors.extend(errs)
                        }
                        if let Err(errs) = table.merge(other) {
                            errors.extend(errs)
                        }
                    }
                }
            }
        }
        if !group_boundary_comment_directives.is_empty() {
            table.group_boundary_comment_directives = Some(group_boundary_comment_directives);
        }

        table.kind = table_kind;

        DocumentTreeAndErrors {
            tree: crate::Value::Table(table),
            errors,
        }
    }
}

impl<'t, T> IntoDocumentTreeWithContext<'t, crate::Table<'t>> for Vec<T>
where
    T: IntoDocumentTreeWithContext<'t, crate::Table<'t>>,
{
    fn into_document_tree_with_context(
        self,
        context: &crate::DocumentTreeContext<'t>,
    ) -> DocumentTreeAndErrors<crate::Table<'t>> {
        let mut errors = Vec::new();
        let tables = self
            .into_iter()
            .map(|value| {
                let (table, errs) = value.into_document_tree_with_context(context).into();
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

impl<'t, T> crate::IntoDocumentTreeAndErrors<'t, crate::Table<'t>> for Vec<T>
where
    T: IntoDocumentTreeWithContext<'t, crate::Table<'t>>,
{
    fn into_document_tree_and_errors(
        self,
        toml_version: tombi_toml_version::TomlVersion,
        decoded: &'t crate::DecodedTextResolver,
    ) -> DocumentTreeAndErrors<crate::Table<'t>> {
        let context = crate::DocumentTreeContext::new(toml_version, decoded);
        self.into_document_tree_with_context(&context)
    }
}

impl<'t> IntoIterator for Table<'t> {
    type Item = (Key<'t>, Value<'t>);
    type IntoIter = tombi_hashmap::map::IntoIter<Key<'t>, Value<'t>>;

    fn into_iter(self) -> Self::IntoIter {
        self.key_values.into_iter()
    }
}

/// Whether the header prefix of `prefix_len` keys is a parent `[[array_of_tables]]`.
fn is_parent_array_of_tables(counts: &[usize], prefix_len: usize) -> bool {
    prefix_len
        .checked_sub(1)
        .and_then(|i| counts.get(i))
        .is_some_and(|count| *count > 0)
}

fn insert_table<'t>(table: &mut Table<'t>, key: Key<'t>) -> Result<(), Vec<crate::Error>> {
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

fn insert_array_of_tables<'t>(
    table: &mut Table<'t>,
    key: Key<'t>,
    new_array_of_tables_fn: impl Fn(&Table<'t>) -> Array<'t>,
) -> Result<(), Vec<crate::Error>> {
    let mut array = new_array_of_tables_fn(table);
    let new_table = table.new_parent_table();
    array.push(Value::Table(std::mem::replace(table, new_table)));
    array.header_comment_directives = key.comment_directives.clone();
    match table.new_parent_table().insert(key, Value::Array(array)) {
        Ok(t) => {
            *table = t;
            Ok(())
        }
        Err(errors) => Err(errors),
    }
}

fn append_header_comment_directives<'t>(
    table: &mut Table<'t>,
    mut keys: impl Iterator<Item = tombi_ast_syntax::Key<'t>>,
    header_comment_directives: &Vec<TombiValueCommentDirective>,
) {
    // Get the next key in the path
    let Some(ast_key) = keys.next() else {
        // No more keys, append comment directives to the final table
        if let Some(table_comment_directives) = table.header_comment_directives.as_mut() {
            table_comment_directives.extend(header_comment_directives.iter().cloned());
        } else {
            table.header_comment_directives = Some(header_comment_directives.clone());
        }
        return;
    };

    // Since Table has only one key, we can directly access it without iteration
    let temp_key_values = std::mem::replace(&mut table.key_values, tombi_hashmap::IndexMap::new());

    // Extract the single key-value pair
    let Some((mut key, value)) = temp_key_values.into_iter().next() else {
        return;
    };

    // Check if this is the key we're looking for
    if key == ast_key {
        // Update the key's comment directives
        if let Some(key_comment_directives) = key.comment_directives.as_mut() {
            key_comment_directives.extend(header_comment_directives.iter().cloned());
        } else {
            key.comment_directives = Some(header_comment_directives.clone());
        }

        if let Value::Table(mut nested_table) = value {
            // Recursively process the remaining keys
            append_header_comment_directives(&mut nested_table, keys, header_comment_directives);
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
