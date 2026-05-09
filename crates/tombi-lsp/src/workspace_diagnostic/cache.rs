use crate::diagnostic::DiagnosticsResult;

#[derive(Debug, Default)]
pub struct WorkspaceDiagnosticsCache(
    Option<tombi_hashmap::HashMap<tombi_uri::Uri, Option<DiagnosticsResult>>>,
);

impl WorkspaceDiagnosticsCache {
    pub fn get(&self, text_document_uri: &tombi_uri::Uri) -> Option<&DiagnosticsResult> {
        self.0
            .as_ref()
            .and_then(|diagnostics_cache| diagnostics_cache.get(text_document_uri))
            .and_then(|diagnostics_result| diagnostics_result.as_ref())
    }

    pub fn set(
        &mut self,
        text_document_uri: tombi_uri::Uri,
        diagnostics_result: DiagnosticsResult,
    ) {
        let diagnostics_cache = self.0.get_or_insert_default();
        diagnostics_cache.insert(text_document_uri, Some(diagnostics_result));
    }

    pub fn tracked(&self) -> Option<Vec<tombi_uri::Uri>> {
        self.0
            .as_ref()
            .map(|diagnostics_cache| diagnostics_cache.keys().cloned().collect())
    }

    pub fn track(&mut self, text_document_uri: tombi_uri::Uri) {
        let diagnostics_cache = self.0.get_or_insert_default();
        diagnostics_cache.entry(text_document_uri).or_insert(None);
    }

    pub fn untrack(&mut self, text_document_uri: &tombi_uri::Uri) {
        if let Some(diagnostics_cache) = self.0.as_mut() {
            diagnostics_cache.remove(text_document_uri);
        }
    }

    pub fn clear(&mut self, text_document_uri: &tombi_uri::Uri) {
        let Some(diagnostics_cache) = self.0.as_mut() else {
            return;
        };

        if let Some(entry) = diagnostics_cache.get_mut(text_document_uri) {
            *entry = None;
        }
    }

    pub fn close(&mut self, text_document_uri: &tombi_uri::Uri) {
        let Some(diagnostics_cache) = self.0.as_mut() else {
            return;
        };

        let Some(Some(diagnostics_result)) = diagnostics_cache.get_mut(text_document_uri) else {
            return;
        };

        diagnostics_result.version = None;
    }

    pub fn clear_all(&mut self) {
        let Some(diagnostics_cache) = self.0.as_mut() else {
            return;
        };

        for diagnostics_result in diagnostics_cache.values_mut() {
            *diagnostics_result = None;
        }
    }

    pub fn reset(&mut self) {
        self.0 = None;
    }
}
