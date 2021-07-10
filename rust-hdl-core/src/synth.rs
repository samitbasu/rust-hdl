use std::fmt::Debug;

use crate::bits::{Bit, Bits};
use crate::clock::Clock;
use num_bigint::BigUint;

pub trait Synth: Default + Copy + PartialEq + Debug
{
    const BITS: usize;
    const ENUM_TYPE: bool = false;
    const TYPE_NAME: &'static str = "Bits";
    fn name(_ndx: usize) -> &'static str {
        ""
    }
    fn big_uint(self) -> BigUint;
}

impl<const N: usize> Synth for Bits<N> {
    const BITS: usize = N;

    fn big_uint(self) -> BigUint {
        self.into()
    }
}

impl Synth for Bit {
    const BITS: usize = 1;

    fn big_uint(self) -> BigUint {
        if self {
            BigUint::from(1_u8)
        } else {
            BigUint::default()
        }
    }
}

impl Synth for Clock {
    const BITS: usize = 1;

    fn big_uint(self) -> BigUint {
        self.0.big_uint()
    }
}
