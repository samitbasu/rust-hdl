use crate::{
    log::{LogBuilder, TagID},
    logger::Logger,
};

pub trait Loggable: Sized {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder);
    fn record<L: Loggable>(&self, tag: TagID<L>, logger: impl Logger);
}

impl Loggable for bool {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        builder.allocate(tag, 1);
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl Logger) {
        logger.write_bool(tag, *self);
    }
}

impl Loggable for u8 {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        builder.allocate(tag, 8);
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl Logger) {
        logger.write_small(tag, *self as u64);
    }
}

impl Loggable for u16 {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        builder.allocate(tag, 16);
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl Logger) {
        logger.write_small(tag, *self as u64);
    }
}
