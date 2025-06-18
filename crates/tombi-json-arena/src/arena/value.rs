use super::{ArrayArena, ObjectArena, StrArena};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub(crate) usize);

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct ValueArena {
    pub(crate) values: Vec<crate::Value>,
    pub(crate) str_arena: StrArena,
    pub(crate) array_arena: ArrayArena,
    pub(crate) object_arena: ObjectArena,
}


impl ValueArena {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn alloc(&mut self, value: crate::Value) -> ValueId {
        let id = ValueId(self.values.len());
        self.values.push(value);
        id
    }
    pub fn get(&self, id: &ValueId) -> Option<&crate::Value> {
        self.values.get(id.0)
    }
    pub fn str_arena(&self) -> &StrArena {
        &self.str_arena
    }
    pub fn str_arena_mut(&mut self) -> &mut StrArena {
        &mut self.str_arena
    }
    pub fn array_arena(&self) -> &ArrayArena {
        &self.array_arena
    }
    pub fn array_arena_mut(&mut self) -> &mut ArrayArena {
        &mut self.array_arena
    }
    pub fn object_arena(&self) -> &ObjectArena {
        &self.object_arena
    }
    pub fn object_arena_mut(&mut self) -> &mut ObjectArena {
        &mut self.object_arena
    }
}
