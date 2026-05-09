use crate::diagnostic::DiagnosticsResult;

#[derive(Debug, Default)]
pub struct WorkspaceDiagnosticsCache {
    workspace_targets: Option<tombi_hashmap::HashSet<tombi_uri::Uri>>,
    cache: tombi_hashmap::HashMap<tombi_uri::Uri, Option<DiagnosticsResult>>,
}

impl WorkspaceDiagnosticsCache {
    pub fn workspace_targets(&self) -> Option<Vec<tombi_uri::Uri>> {
        self.workspace_targets
            .as_ref()
            .map(|tracked_targets| tracked_targets.iter().cloned().collect())
    }

    pub fn set_workspace_targets(
        &mut self,
        workspace_targets: tombi_hashmap::HashSet<tombi_uri::Uri>,
    ) {
        self.workspace_targets = Some(workspace_targets);
    }

    pub fn get(&self, text_document_uri: &tombi_uri::Uri) -> Option<&DiagnosticsResult> {
        self.cache
            .get(text_document_uri)
            .and_then(|result| result.as_ref())
    }

    pub fn set(
        &mut self,
        text_document_uri: tombi_uri::Uri,
        diagnostics_result: DiagnosticsResult,
    ) {
        self.cache
            .insert(text_document_uri, Some(diagnostics_result));
    }

    pub fn track(&mut self, text_document_uri: tombi_uri::Uri, is_workspace_target: bool) {
        if is_workspace_target && let Some(tracked_targets) = self.workspace_targets.as_mut() {
            tracked_targets.insert(text_document_uri.clone());
        }
        self.cache.entry(text_document_uri).or_insert(None);
    }

    pub fn untrack(&mut self, text_document_uri: &tombi_uri::Uri) {
        if let Some(tracked_targets) = self.workspace_targets.as_mut() {
            tracked_targets.remove(text_document_uri);
        }
        self.cache.remove(text_document_uri);
    }

    pub fn clear(&mut self, text_document_uri: &tombi_uri::Uri) {
        self.cache.remove(text_document_uri);
    }

    pub fn close(&mut self, text_document_uri: &tombi_uri::Uri) {
        if let Some(tracked_targets) = self.workspace_targets.as_mut()
            && tracked_targets.contains(text_document_uri)
        {
            if let Some(diagnostics_result) = self
                .cache
                .get_mut(text_document_uri)
                .and_then(|result| result.as_mut())
            {
                diagnostics_result.version = None;
            }
        } else {
            self.cache.remove(text_document_uri);
        }
    }

    pub fn clear_all(&mut self) {
        self.cache.clear();
    }

    pub fn reset(&mut self) {
        self.workspace_targets = None;
        self.cache.clear();
    }
}
