use crate::{log::TagID, loggable::Loggable};

pub trait Logger: Sized {
    fn log<L: Loggable>(&mut self, tag: TagID<L>, val: L) {
        val.record(tag, self)
    }
    fn write_bool<L: Loggable>(&mut self, tag: TagID<L>, val: bool);
    fn write_small<L: Loggable>(&mut self, tag: TagID<L>, val: u64);
    fn write_large<L: Loggable>(&mut self, tag: TagID<L>, val: &[bool]);
    fn write_string<L: Loggable>(&mut self, tag: TagID<L>, val: &'static str);
}

impl<T: Logger> Logger for &mut T {
    fn write_bool<L: Loggable>(&mut self, tag: TagID<L>, val: bool) {
        (**self).write_bool(tag, val)
    }

    fn write_small<L: Loggable>(&mut self, tag: TagID<L>, val: u64) {
        (**self).write_small(tag, val)
    }

    fn write_large<L: Loggable>(&mut self, tag: TagID<L>, val: &[bool]) {
        (**self).write_large(tag, val)
    }

    fn write_string<L: Loggable>(&mut self, tag: TagID<L>, val: &'static str) {
        (**self).write_string(tag, val)
    }
}

impl Logger for () {
    fn write_bool<L: Loggable>(&mut self, _: TagID<L>, _: bool) {}

    fn write_small<L: Loggable>(&mut self, _: TagID<L>, _: u64) {}

    fn write_large<L: Loggable>(&mut self, _: TagID<L>, _: &[bool]) {}

    fn write_string<L: Loggable>(&mut self, _: TagID<L>, _: &'static str) {}
}
