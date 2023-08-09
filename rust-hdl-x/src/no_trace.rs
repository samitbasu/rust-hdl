use crate::tracer::{TraceID, TraceTag, Tracer};

impl Tracer for () {
    fn set_context(&mut self, _id: TraceID) {}
    fn set_tag(&mut self, _tag: TraceTag) {}
    fn write_bool(&mut self, _val: bool) {}
    fn write_small(&mut self, _val: u64) {}
    fn write_large(&mut self, _val: &[bool]) {}
    fn write_string(&mut self, _val: &'static str) {}
}
