#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TraceID(usize);

impl From<usize> for TraceID {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl From<TraceID> for usize {
    fn from(id: TraceID) -> Self {
        id.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum TraceTag {
    #[default]
    Input,
    Output,
    StateD,
    StateQ,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TraceType {
    Input,
    Output,
    State,
}

pub trait Tracer {
    fn set_context(&mut self, id: TraceID);
    fn set_tag(&mut self, tag: TraceTag);
    fn write_bool(&mut self, val: bool);
    fn write_small(&mut self, val: u64);
    fn write_large(&mut self, val: &[bool]);
    fn write_string(&mut self, val: &'static str);
}

impl<T: Tracer> Tracer for &mut T {
    fn set_context(&mut self, id: TraceID) {
        (**self).set_context(id)
    }
    fn set_tag(&mut self, tag: TraceTag) {
        (**self).set_tag(tag)
    }
    fn write_bool(&mut self, val: bool) {
        (**self).write_bool(val)
    }
    fn write_small(&mut self, val: u64) {
        (**self).write_small(val)
    }
    fn write_large(&mut self, val: &[bool]) {
        (**self).write_large(val)
    }
    fn write_string(&mut self, val: &'static str) {
        (**self).write_string(val)
    }
}
