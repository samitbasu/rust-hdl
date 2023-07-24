use num_traits::Num;
use ruint::Uint;
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
    fn new(tracer: &'a T, name: &str) -> Self {
        tracer.enter(name);
        Self { tracer }
    }
}

impl<'a, T: Tracer> Drop for Scope<'a, T> {
    fn drop(&mut self) {
        self.tracer.exit();
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

trait Binary<const T: usize> {}

impl Binary<1> for Uint<1, 1> {}
impl Binary<2> for Uint<2, 1> {}
impl Binary<4> for Uint<4, 1> {}
