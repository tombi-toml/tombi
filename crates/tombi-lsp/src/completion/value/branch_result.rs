use crate::completion::CompletionContent;

#[derive(Debug)]
pub(super) struct BranchCompletionResult {
    pub has_key: bool,
    pub is_valid: bool,
    pub is_recoverable: bool,
    pub items: Vec<CompletionContent>,
}
