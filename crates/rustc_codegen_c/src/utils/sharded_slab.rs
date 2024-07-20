pub use sharded_slab::Entry;

pub use crate::utils::slab::Id;

pub struct ShardedSlab<T> {
    inner: sharded_slab::Slab<T>,
}

impl<T> Default for ShardedSlab<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ShardedSlab<T> {
    pub fn new() -> Self {
        Self { inner: sharded_slab::Slab::new() }
    }

    pub fn insert(&self, value: T) -> Id<T> {
        let index = self.inner.insert(value).expect("sharded slab is full");
        Id::new(index)
    }

    pub fn get(&self, id: Id<T>) -> Option<Entry<T>> {
        self.inner.get(id.index)
    }
}
