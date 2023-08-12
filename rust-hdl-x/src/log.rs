use std::marker::PhantomData;

use crate::{loggable::Loggable, synchronous::Synchronous};

#[derive(Debug, Clone, Copy)]
pub struct TagID<T: Loggable> {
    pub context: usize,
    pub id: usize,
    pub _marker: PhantomData<T>,
}

pub trait LogBuilder {
    type SubBuilder: LogBuilder;
    fn scope<S: Synchronous>(&self, name: &str) -> Self::SubBuilder;
    fn tag<T: Loggable>(&mut self, name: &str) -> TagID<T>;
    fn allocate<L: Loggable>(&self, tag: TagID<L>, width: usize);
    fn namespace<L: Loggable>(&self, name: &str) -> Self::SubBuilder;
}

impl<T: LogBuilder> LogBuilder for &mut T {
    type SubBuilder = T::SubBuilder;
    fn scope<S: Synchronous>(&self, name: &str) -> Self::SubBuilder {
        (**self).scope::<S>(name)
    }
    fn tag<L: Loggable>(&mut self, name: &str) -> TagID<L> {
        (**self).tag(name)
    }
    fn allocate<L: Loggable>(&self, tag: TagID<L>, width: usize) {
        (**self).allocate(tag, width)
    }
    fn namespace<L: Loggable>(&self, name: &str) -> Self::SubBuilder {
        (**self).namespace::<L>(name)
    }
}
