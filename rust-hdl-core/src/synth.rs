use std::fmt::Debug;

use crate::clock::Clock;
use rust_hdl_bitvec::{Bit, Bits};

pub trait Synth: Default + Copy + PartialEq + Debug {
    const BITS: usize;
}

impl<const N: usize> Synth for Bits<N> {
    const BITS: usize = N;
}

impl Synth for Bit {
    const BITS: usize = 1;
}

impl Synth for Clock {
    const BITS: usize = 1;
}
