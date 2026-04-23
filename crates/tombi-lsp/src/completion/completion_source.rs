use tombi_extension::CompletionHint;
use tombi_schema_store::{Accessor, CurrentSchema};

use crate::schema_resolver::{remaining_keys, resolve_accessors_for_document_or_schema};

pub(super) enum CompletionSource<'a> {
    Root {
        remaining_keys: &'a [tombi_document_tree::Key],
        accessors: Vec<Accessor>,
        current_schema: Option<CurrentSchema<'static>>,
    },
    Value {
        remaining_keys: &'a [tombi_document_tree::Key],
        accessors: Vec<Accessor>,
        current_schema: Option<CurrentSchema<'static>>,
    },
    Schema {
        remaining_keys: &'a [tombi_document_tree::Key],
        accessors: Vec<Accessor>,
        current_schema: CurrentSchema<'static>,
    },
}

impl<'a> CompletionSource<'a> {
    pub(super) async fn new(
        document_tree: &'a tombi_document_tree::DocumentTree,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        schema_context: &tombi_schema_store::SchemaContext<'_>,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        let accessors = tombi_document_tree::get_accessors(document_tree, keys, position);
        let (mut accessors, mut current_schema) =
            resolve_accessors_for_document_or_schema(document_tree, accessors, schema_context)
                .await;

        if matches!(
            completion_hint,
            None | Some(CompletionHint::DotTrigger { .. })
        ) && !keys.is_empty()
            && remaining_keys(keys, &accessors).is_empty()
            && matches!(
                tombi_document_tree::dig_accessors(document_tree, &accessors),
                Some((_, tombi_document_tree::Value::Incomplete { .. }))
            )
            && matches!(accessors.last(), Some(Accessor::Key(_)))
        {
            let parent_accessors = accessors[..accessors.len().saturating_sub(1)].to_vec();
            (accessors, current_schema) = resolve_accessors_for_document_or_schema(
                document_tree,
                parent_accessors,
                schema_context,
            )
            .await;
        }

        let remaining_keys = remaining_keys(keys, &accessors);

        if accessors.is_empty() {
            return Some(Self::Root {
                remaining_keys,
                accessors,
                current_schema,
            });
        }

        if tombi_document_tree::dig_accessors(document_tree, &accessors).is_some() {
            return Some(Self::Value {
                remaining_keys,
                accessors,
                current_schema,
            });
        }

        current_schema.map(|current_schema| Self::Schema {
            remaining_keys,
            accessors,
            current_schema,
        })
    }
}
