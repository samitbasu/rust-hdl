use std::marker::PhantomData;

use crate::loggable::Loggable;

#[derive(Debug)]
pub struct TagID<T: Loggable> {
    pub context: usize,
    pub id: usize,
    pub _marker: PhantomData<*const T>,
}

impl<T: Loggable> Clone for TagID<T> {
    fn clone(&self) -> Self {
        Self {
            context: self.context,
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T: Loggable> Copy for TagID<T> {}

#[derive(Debug, Clone, Copy)]
pub struct ClockDetails {
    pub period_in_fs: u64,
    pub offset_in_fs: u64,
    pub initial_state: bool,
}

pub trait LogBuilder {
    type SubBuilder: LogBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder;
    fn tag<T: Loggable>(&mut self, name: &str) -> TagID<T>;
    fn allocate<L: Loggable>(&self, tag: TagID<L>, width: usize);
    fn namespace(&self, name: &str) -> Self::SubBuilder;
    fn add_clock(&mut self, clock: ClockDetails);
}

impl<T: LogBuilder> LogBuilder for &mut T {
    type SubBuilder = T::SubBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder {
        (**self).scope(name)
    }
    fn tag<L: Loggable>(&mut self, name: &str) -> TagID<L> {
        (**self).tag(name)
    }
    fn allocate<L: Loggable>(&self, tag: TagID<L>, width: usize) {
        (**self).allocate(tag, width)
    }
    fn namespace(&self, name: &str) -> Self::SubBuilder {
        (**self).namespace(name)
    }
    fn add_clock(&mut self, clock: ClockDetails) {
        (**self).add_clock(clock)
    }
}
