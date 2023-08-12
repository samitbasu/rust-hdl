use crate::{loggable::Loggable, logger::Logger};

pub trait Synchronous: Sized {
    type Input: Copy + Loggable;
    type Output: Copy + Loggable + Default;
    type State: Copy + Default + Loggable;
    // User provided
    fn compute(
        &self,
        tracer: impl Logger,
        inputs: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State);
}
