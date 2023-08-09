use crate::tracer::{TraceID, TraceType};

pub trait TracerBuilder {
    type SubBuilder: TracerBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder;
    fn trace_id(&self) -> TraceID;
    fn set_kind(&mut self, kind: TraceType);
    fn register(&self, width: usize);
    fn namespace(&self, name: &str) -> Self::SubBuilder;
}

impl<T: TracerBuilder> TracerBuilder for &mut T {
    type SubBuilder = T::SubBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder {
        (**self).scope(name)
    }
    fn trace_id(&self) -> TraceID {
        (**self).trace_id()
    }
    fn register(&self, width: usize) {
        (**self).register(width)
    }
    fn namespace(&self, name: &str) -> Self::SubBuilder {
        (**self).namespace(name)
    }
    fn set_kind(&mut self, kind: TraceType) {
        (**self).set_kind(kind)
    }
}
