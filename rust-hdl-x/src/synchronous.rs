use crate::{loggable::Loggable, logger::Logger};

pub trait Synchronous: Sized {
    type Input: Copy + Loggable + PartialEq;
    type Output: Copy + Loggable + Default;
    type State: Copy + Default + Loggable;
    // User provided
    fn compute(
        &self,
        logger: impl Logger,
        inputs: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State);
}
