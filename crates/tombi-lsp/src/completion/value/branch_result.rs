use tombi_schema_store::{Accessor, CurrentSchema, SchemaAccessor, SchemaView};

use crate::completion::{
    CompletionContent, CompletionHint, FindCompletionContents, dedup_composite_completion_contents,
    take_completion_schema_tooltip,
};

#[derive(Debug)]
pub(super) struct BranchCompletionResult {
    pub has_key: bool,
    pub is_valid: bool,
    pub is_recoverable: bool,
}

impl BranchCompletionResult {
    fn should_include(&self, valid_branches: bool, narrow_branches: bool) -> bool {
        if valid_branches {
            self.is_valid
        } else {
            !narrow_branches || self.has_key
        }
    }

    fn should_include_in_fallback(&self, valid_branches: bool, narrow_branches: bool) -> bool {
        if valid_branches {
            self.is_valid || self.is_recoverable
        } else if narrow_branches {
            self.has_key || self.is_recoverable
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
    applicator: tombi_validator::Applicator,
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree_syntax::Key],
    accessors: &'a [Accessor],
    resolved_schemas: &'a [CurrentSchema<'a>],
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    completion_hint: Option<CompletionHint>,
) -> (Vec<CompletionContent>, bool)
where
    T: FindCompletionContents + tombi_validator::Validate + Sync + Send + std::fmt::Debug,
{
    let first_key = (keys.len() == 1 && !keys[0].value().is_empty())
        .then(|| SchemaAccessor::Key(keys[0].value().to_owned()));
    let evaluation = tombi_validator::evaluate_applicator(
        applicator,
        value,
        accessors,
        resolved_schemas,
        schema_context,
    )
    .await;

    let mut branch_results = Vec::new();
    for (resolved_schema, branch_applicability) in resolved_schemas.iter().zip(&evaluation.branches)
    {
        let branch_has_key = if let Some(first_key) = &first_key {
            match resolved_schema.schema_view.as_ref() {
                SchemaView::Table(table_schema) => {
                    table_schema.properties.read().await.contains_key(first_key)
                }
                _ => false,
            }
        } else {
            false
        };
        let branch_is_valid = branch_applicability.is_applicable();
        let branch_is_recoverable = branch_applicability.is_recoverable_at(position);

        branch_results.push(BranchCompletionResult {
            has_key: branch_has_key,
            is_valid: branch_is_valid,
            is_recoverable: branch_is_recoverable,
        });
    }

    let valid_branches = branch_results.iter().any(|branch| branch.is_valid);
    let narrow_branches = branch_results.iter().any(|branch| branch.has_key);

    let mut completion_items = Vec::new();
    for (resolved_schema, branch) in resolved_schemas.iter().zip(&branch_results) {
        if branch.should_include(valid_branches, narrow_branches) {
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
            completion_items.extend(schema_completions.into_iter().map(|mut item| {
                let tooltip = take_completion_schema_tooltip(&mut item, resolved_schema);
                (item, tooltip)
            }));
        }
    }

    // Fallback: if the precision-focused first pass yielded nothing, relax by also
    // including branches whose validation errors are at the cursor — the user is still
    // typing there, so that branch may become valid.
    if completion_items.is_empty() {
        for (resolved_schema, branch) in resolved_schemas.iter().zip(&branch_results) {
            if !branch.should_include(valid_branches, narrow_branches)
                && branch.should_include_in_fallback(valid_branches, narrow_branches)
            {
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
                completion_items.extend(schema_completions.into_iter().map(|mut item| {
                    let tooltip = take_completion_schema_tooltip(&mut item, resolved_schema);
                    (item, tooltip)
                }));
            }
        }
    }

    (
        dedup_composite_completion_contents(completion_items),
        narrow_branches,
    )
}
