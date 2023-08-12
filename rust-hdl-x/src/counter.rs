use rust_hdl::prelude::Bits;

use crate::{
    synchronous::Synchronous,
    tracer::{TraceID, Tracer},
    tracer_builder::TracerBuilder,
};

struct BitCounter<const N: usize> {
    trace_id: Option<TraceID>,
}

impl<const N: usize> Synchronous for BitCounter<N> {
    type State = Bits<N>;
    type Input = bool;
    type Output = Bits<N>;

    fn setup(&mut self, builder: impl TracerBuilder) {
        self.trace_id = Some(Self::register_trace_types(builder));
    }
    fn trace_id(&self) -> Option<TraceID> {
        self.trace_id
    }
    fn compute(
        &self,
        _tracer: impl Tracer,
        input: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State) {
        let new_state = if input { state + 1 } else { state };
        let output = new_state;
        (output, new_state)
    }
}

#[test]
fn test_bit_counter_with_tracing() {
    let mut counter = BitCounter::<32> { trace_id: None };
    let mut tracer_builder = crate::basic_tracer::BasicTracerBuilder::default();
    counter.setup(&mut tracer_builder);
    let mut tracer = tracer_builder.build();
    let mut state = Bits::<32>::default();
    let mut last_output = Bits::<32>::default();
    for cycle in 0..10_000_000 {
        let (output, new_state) = counter.update(&mut tracer, cycle % 2 == 0, state);
        state = new_state;
        last_output = output;
        //        println!("{} {}", output, state);
    }
    println!("Last output {last_output:x}");
    println!("{}", tracer);
}
