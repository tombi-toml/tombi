use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::Accessor;

pub struct SchemaCycleGuard {
    visits: SchemaVisits,
    key: VisitKey,
}

impl Drop for SchemaCycleGuard {
    fn drop(&mut self) {
        self.visits.lock().remove(&self.key);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum VisitKey {
    Schemas(usize),
    /// `SchemaView` at an instance location. Location is part of the key so the
    /// same recursive schema can validate nested children; same-location re-entry
    /// (e.g. `{ "$ref": "#" }` on the current value) remains a cycle.
    SchemaView {
        ptr: usize,
        location: u64,
    },
}

#[derive(Debug, Default, Clone)]
pub struct SchemaVisits(Arc<Mutex<tombi_hashmap::HashSet<VisitKey>>>);

impl SchemaVisits {
    pub fn get_cycle_guard(
        &self,
        schemas: &crate::ReferableSchemaViews,
    ) -> Option<SchemaCycleGuard> {
        let key = VisitKey::Schemas(std::sync::Arc::as_ptr(schemas) as usize);
        self.get_cycle_guard_with_key(key)
    }

    pub fn get_schema_view_cycle_guard(
        &self,
        schema_view: &std::sync::Arc<crate::SchemaView>,
        accessors: &[Accessor],
    ) -> Option<SchemaCycleGuard> {
        let key = VisitKey::SchemaView {
            ptr: std::sync::Arc::as_ptr(schema_view) as usize,
            location: location_hash(accessors),
        };
        self.get_cycle_guard_with_key(key)
    }

    fn lock(&self) -> MutexGuard<'_, tombi_hashmap::HashSet<VisitKey>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_cycle_guard_with_key(&self, key: VisitKey) -> Option<SchemaCycleGuard> {
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

fn location_hash(accessors: &[Accessor]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    accessors.hash(&mut hasher);
    hasher.finish()
}
