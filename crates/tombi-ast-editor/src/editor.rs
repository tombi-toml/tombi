use std::borrow::Cow;

use tombi_ast::AstNode;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_schema_store::{CurrentSchema, SchemaContext};

use crate::{change::Change, Edit};

pub struct Editor<'a> {
    root: tombi_ast::Root,
    #[allow(dead_code)]
    changes: Vec<Change>,
    source_path: Option<&'a std::path::Path>,
    schema_context: &'a SchemaContext<'a>,
}

impl<'a> Editor<'a> {
    pub fn new(
        root: tombi_ast::Root,
        source_path: Option<&'a std::path::Path>,
        schema_context: &'a SchemaContext<'a>,
    ) -> Self {
        Self {
            root,
            changes: vec![],
            source_path,
            schema_context,
        }
    }

    pub async fn edit(self) -> tombi_ast::Root {
        let Ok(document_tree) = self
            .root
            .clone()
            .try_into_document_tree(self.schema_context.toml_version)
        else {
            return self.root;
        };
        let new_root = self.root.clone_for_update();

        let current_schema = self.schema_context.root_schema.and_then(|document_schema| {
            document_schema
                .value_schema
                .as_ref()
                .map(|value_schema| CurrentSchema {
                    value_schema: Cow::Borrowed(value_schema),
                    schema_uri: Cow::Borrowed(&document_schema.schema_uri),
                    definitions: Cow::Borrowed(&document_schema.definitions),
                })
        });

        let changes = new_root
            .edit(
                &document_tree.into(),
                &[],
                self.source_path,
                current_schema.as_ref(),
                self.schema_context,
            )
            .await;

        for change in changes {
            match change {
                Change::AppendTop { new } => {
                    new_root.syntax().splice_children(0..0, new);
                }
                Change::Append { base, new } => {
                    let index = base.index() + 1;
                    if let Some(node) = base.parent().as_ref().or_else(|| base.as_node()) {
                        node.splice_children(index..index, new);
                    }
                }
                Change::Remove { target } => {
                    let index = target.index();
                    if let Some(node) = target.parent().as_ref().or_else(|| target.as_node()) {
                        node.splice_children(index..index + 1, Vec::with_capacity(0));
                    }
                }
                Change::ReplaceRange { old, new } => {
                    let start = old.start().index();
                    let end = old.end().index();
                    if let Some(node) = old
                        .start()
                        .parent()
                        .as_ref()
                        .or_else(|| old.start().as_node())
                    {
                        node.splice_children(start..end + 1, new);
                    }
                }
            }
        }

        new_root
    }
}
