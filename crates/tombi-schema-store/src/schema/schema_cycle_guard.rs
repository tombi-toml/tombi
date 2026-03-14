use std::sync::{Arc, Mutex, MutexGuard};

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
pub struct SchemaVisits(Arc<Mutex<tombi_hashmap::HashSet<usize>>>);

impl SchemaVisits {
    pub fn get_cycle_guard(
        &self,
        schemas: &crate::ReferableValueSchemas,
    ) -> Option<SchemaCycleGuard> {
        let key = std::sync::Arc::as_ptr(schemas) as usize;
        self.get_cycle_guard_with_key(key)
    }

    pub fn get_value_schema_cycle_guard(
        &self,
        value_schema: &std::sync::Arc<crate::ValueSchema>,
    ) -> Option<SchemaCycleGuard> {
        let key = std::sync::Arc::as_ptr(value_schema) as usize;
        self.get_cycle_guard_with_key(key)
    }

    fn lock(&self) -> MutexGuard<'_, tombi_hashmap::HashSet<usize>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_cycle_guard_with_key(&self, key: usize) -> Option<SchemaCycleGuard> {
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
