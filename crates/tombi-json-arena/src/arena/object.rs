use super::{StrId, ValueId};
use ahash::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectArena {
    /// Note that this is not an IndexMap. We prioritize memory efficiency since it is only used for interpreting JSON Schema.
    data: Vec<HashMap<StrId, ValueId>>,
}

impl Default for ObjectArena {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectArena {
    pub fn new() -> Self {
        ObjectArena { data: Vec::new() }
    }

    pub fn insert(&mut self, object: HashMap<StrId, ValueId>) -> ObjectId {
        let id = ObjectId(self.data.len());
        self.data.push(object);
        id
    }

    pub fn get(&self, id: ObjectId) -> Option<&HashMap<StrId, ValueId>> {
        self.data.get(id.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub usize);
