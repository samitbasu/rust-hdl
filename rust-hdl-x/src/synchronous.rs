use num_traits::Num;
use std::{
    io::Write,
    ops::{BitAnd, Shr},
    path::PathBuf,
};

use crate::tracer::{BitSerialize, Tracer};

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
    fn update(
        &self,
        tracer: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State);
    fn default_output(&self) -> Self::Output;
}
