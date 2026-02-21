use std::{
    collections::HashSet,
    sync::{Arc, Mutex, MutexGuard},
};

pub struct SchemaCycleGuard {
    visits: SchemaVisits,
    key: usize,
}

impl Drop for SchemaCycleGuard {
    fn drop(&mut self) {
        self.visits.lock().remove(&self.key);
    }
}

#[derive(Debug, Default, Clone)]
pub struct SchemaVisits(Arc<Mutex<HashSet<usize>>>);

impl SchemaVisits {
    fn lock(&self) -> MutexGuard<'_, HashSet<usize>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn get_cycle_guard(
        &self,
        schemas: &crate::ReferableValueSchemas,
    ) -> Option<SchemaCycleGuard> {
        let key = std::sync::Arc::as_ptr(schemas) as usize;
        if self.lock().insert(key) {
            Some(SchemaCycleGuard {
                visits: self.clone(),
                key,
            })
        } else {
            None
        }
    }
}
