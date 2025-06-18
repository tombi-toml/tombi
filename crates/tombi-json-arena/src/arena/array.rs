use super::ValueId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayArena {
    data: Vec<Vec<ValueId>>,
}

impl Default for ArrayArena {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrayArena {
    pub fn new() -> Self {
        ArrayArena { data: Vec::new() }
    }

    pub fn insert(&mut self, array: Vec<ValueId>) -> ArrayId {
        let id = ArrayId(self.data.len());
        self.data.push(array);
        id
    }

    pub fn get(&self, id: ArrayId) -> Option<&Vec<ValueId>> {
        self.data.get(id.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayId(pub usize);
