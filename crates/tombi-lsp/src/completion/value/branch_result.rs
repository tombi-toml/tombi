use tombi_schema_store::{Accessor, CurrentSchema, SchemaAccessor, ValueSchema};

use crate::completion::{CompletionContent, CompletionHint, FindCompletionContents};

#[derive(Debug)]
pub(super) struct BranchCompletionResult {
    pub has_key: bool,
    pub is_valid: bool,
    pub is_recoverable: bool,
    pub items: Vec<CompletionContent>,
}

impl BranchCompletionResult {
    fn should_include_in_fallback(&self, valid_branches: bool, narrow_branches: bool) -> bool {
        if narrow_branches {
            self.has_key || self.is_recoverable
        } else if valid_branches {
            self.is_valid || self.is_recoverable
        } else {
            true
        }
    }
}

/// Evaluate each branch of a composite schema (oneOf/anyOf) and collect completion items.
///
/// Narrowing: when completing the value of a single key (e.g. `license = { file = "..." }`),
/// only consider branches that are a table containing that key. Otherwise we would merge
/// completions from all branches (e.g. file path and string variant like "MIT"). Requires
/// exactly one non-empty key so we do not narrow when completing after a dot (e.g. `path.`
/// yields `keys = ["path", ""]`). Only narrows when at least one branch has the key, so we
/// never return `[]` by over-narrowing.
///
/// Returns the collected items and the `narrow_branches` flag for the caller to decide
/// whether to include composite-level default/examples.
pub(super) async fn collect_branch_completions<'a, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [Accessor],
    resolved_schemas: &'a [CurrentSchema<'a>],
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    completion_hint: Option<CompletionHint>,
) -> (Vec<CompletionContent>, bool)
where
    T: FindCompletionContents + tombi_validator::Validate + Sync + Send + std::fmt::Debug,
{
    let first_key = (keys.len() == 1 && !keys[0].value.is_empty()).then(|| &keys[0].value);

    let mut branch_results = Vec::new();
    for resolved_schema in resolved_schemas {
        let branch_has_key = if let Some(ref first_key) = first_key {
            match resolved_schema.value_schema.as_ref() {
                ValueSchema::Table(table_schema) => table_schema
                    .properties
                    .read()
                    .await
                    .contains_key(&SchemaAccessor::Key(first_key.to_string())),
                _ => false,
            }
        } else {
            false
        };
        let (branch_is_valid, branch_is_recoverable) = match value
            .validate(accessors, Some(resolved_schema), schema_context)
            .await
        {
            Ok(_) => (true, true),
            Err(tombi_validator::Error { diagnostics, .. })
                if diagnostics
                    .iter()
                    .all(tombi_diagnostic::Diagnostic::is_warning) =>
            {
                (true, true)
            }
            Err(tombi_validator::Error { diagnostics, .. }) => (
                false,
                diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.range().contains(position)),
            ),
        };

        let schema_completions = value
            .find_completion_contents(
                position,
                keys,
                accessors,
                Some(resolved_schema),
                schema_context,
                completion_hint,
            )
            .await;

        branch_results.push(BranchCompletionResult {
            has_key: branch_has_key,
            is_valid: branch_is_valid,
            is_recoverable: branch_is_recoverable,
            items: schema_completions,
        });
    }

    let valid_branches = branch_results.iter().any(|branch| branch.is_valid);
    let narrow_branches = branch_results.iter().any(|branch| branch.has_key);

    let mut completion_items = Vec::new();
    for branch in &branch_results {
        if narrow_branches {
            if branch.has_key {
                completion_items.extend(branch.items.iter().cloned());
            }
        } else if valid_branches {
            if branch.is_valid {
                completion_items.extend(branch.items.iter().cloned());
            }
        } else {
            completion_items.extend(branch.items.iter().cloned());
        }
    }

    // Fallback: if the precision-focused first pass yielded nothing, relax by also
    // including branches whose validation errors are at the cursor — the user is still
    // typing there, so that branch may become valid.
    if completion_items.is_empty() {
        completion_items = branch_results
            .into_iter()
            .filter(|branch| branch.should_include_in_fallback(valid_branches, narrow_branches))
            .flat_map(|branch| branch.items)
            .collect();
    }

    (completion_items, narrow_branches)
}
