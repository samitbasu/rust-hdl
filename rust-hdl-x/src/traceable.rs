use rust_hdl::prelude::Bits;

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

impl<const N: usize> Traceable for Bits<N> {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(N);
    }
    fn record(&self, mut tracer: impl Tracer) {
        match self {
            Bits::Short(x) => tracer.write_small(x.short() as u64),
            Bits::Long(x) => tracer.write_large(&x.bits()),
        }
    }
}
