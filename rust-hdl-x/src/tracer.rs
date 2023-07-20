// For tracing, we would like to add a tracer to the update function.
// That means the trait needs to include a tracer

// trait Synchronous {
//     type Input;
//     type Output;
//     type State;
//     type Tracer;
//     fn update(&self, tracer: &mut Trace, q: Self::State, trigger: Self::Input) -> (Self::Output, Self::State);
//     fn default_output(&self) -> Self::Output;
// }

// Then, in the update function, we can trace the various signals.
// 
// For example, in the shot test, we would like to trace the trigger, the state, 
// and the counter.

