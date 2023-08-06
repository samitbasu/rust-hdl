use num_traits::Num;
use std::{
    io::Write,
    ops::{BitAnd, Shr},
    path::PathBuf,
};

use crate::{
    tracer::{BitSerialize, Tracer},
    TraceHandle, TraceType, TracerSetup,
};

pub struct Scope<'a, T: Tracer> {
    tracer: &'a T,
}

impl<'a, T: Tracer> Scope<'a, T> {
    fn new(tracer: &'a T, name: &'static str) -> Self {
        tracer.enter_module(name);
        Self { tracer }
    }
}

impl<'a, T: Tracer> Drop for Scope<'a, T> {
    fn drop(&mut self) {
        self.tracer.exit_module();
    }
}

pub trait Synchronous {
    type Input: Copy + BitSerialize;
    type Output: Copy + BitSerialize;
    type State: Copy + Default + BitSerialize;
    fn setup(&mut self, trace: impl TracerSetup);
    fn update(
        &self,
        tracer: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State);
    fn default_output(&self) -> Self::Output;
    fn register_trace_types(tracer: impl TracerSetup) -> TraceHandle {
        Self::Input::register_trace_type(tracer, TraceType::Input);
        Self::Output::register_trace_type(tracer, TraceType::Output);
        Self::State::register_trace_type(tracer, TraceType::StateD);
        Self::State::register_trace_type(tracer, TraceType::StateQ);
        tracer.handle()
    }
}

impl<T: Synchronous> Synchronous for &T {
    type Input = T::Input;
    type Output = T::Output;
    type State = T::State;
    fn setup<U: TracerSetup>(&mut self, tracer: U) {
        (*self).setup(tracer)
    }
    fn update(
        &self,
        tracer: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        (*self).update(tracer, state, inputs)
    }
    fn default_output(&self) -> Self::Output {
        (*self).default_output()
    }
}

impl<T: Synchronous> Synchronous for &mut T {
    type Input = T::Input;
    type Output = T::Output;
    type State = T::State;
    fn setup<U: TracerSetup>(&mut self, tracer: U) {
        (**self).setup(tracer)
    }
    fn update(
        &self,
        tracer: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        (**self).update(tracer, state, inputs)
    }
    fn default_output(&self) -> Self::Output {
        (**self).default_output()
    }
}
