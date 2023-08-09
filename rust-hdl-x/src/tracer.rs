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
