use crate::{tracer::Tracer, tracer_builder::TracerBuilder};

pub trait Traceable {
    fn register_trace_type(tracer: impl TracerBuilder);
    fn record(&self, tracer: impl Tracer);
}

impl Traceable for bool {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(1);
    }
    fn record(&self, mut tracer: impl Tracer) {
        tracer.write_bool(*self);
    }
}

impl Traceable for u8 {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(8);
    }
    fn record(&self, mut tracer: impl Tracer) {
        tracer.write_small(*self as u64);
    }
}

impl Traceable for u16 {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(16);
    }
    fn record(&self, mut tracer: impl Tracer) {
        tracer.write_small(*self as u64);
    }
}

impl Traceable for u32 {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(32);
    }
    fn record(&self, mut tracer: impl Tracer) {
        tracer.write_small(*self as u64);
    }
}
