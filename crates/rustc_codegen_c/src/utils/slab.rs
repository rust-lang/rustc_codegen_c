use std::ops::{Index, IndexMut};

pub struct Slab<T> {
    data: Vec<T>,
}



impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Slab<T> {
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn insert(&mut self, value: T) -> Id<T> {
        let index = self.data.len();
        self.data.push(value);
        Id { index, _phantom: std::marker::PhantomData }
    }

    pub fn get(&self, id: Id<T>) -> Option<&T> {
        self.data.get(id.index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }
}

impl<T> IntoIterator for Slab<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> std::vec::IntoIter<T> {
        self.data.into_iter()
    }
}

impl<T> Index<Id<T>> for Slab<T> {
    type Output = T;
    fn index(&self, id: Id<T>) -> &T {
        &self.data[id.index]
    }
}

impl<T> IndexMut<Id<T>> for Slab<T> {
    fn index_mut(&mut self, id: Id<T>) -> &mut T {
        &mut self.data[id.index]
    }
}

pub struct Id<T> {
    index: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Copy for Id<T>{}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Id<T> {}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
