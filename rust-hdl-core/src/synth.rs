use std::fmt::Debug;

use crate::bits::{Bit, Bits};
use crate::clock::Clock;

pub trait Synth: Default + Copy + PartialEq + Debug {
    const BITS: usize;
    const ENUM_TYPE: bool = false;
    const TYPE_NAME: &'static str = "Bits";
    fn name(_ndx: usize) -> &'static str {
        ""
    }
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
