use std::ops::{BitAnd, Shr};

use num_traits::Num;
use rust_hdl::prelude::Bits;

pub trait BitSlice: Copy + Num + BitAnd<Output = Self> + Shr<Output = Self> {
    const BITS: usize;
}

impl BitSlice for i8 {
    const BITS: usize = 8;
}

impl BitSlice for u8 {
    const BITS: usize = 8;
}

impl BitSlice for i16 {
    const BITS: usize = 16;
}

impl BitSlice for u16 {
    const BITS: usize = 16;
}

impl BitSlice for i32 {
    const BITS: usize = 32;
}

impl BitSlice for u32 {
    const BITS: usize = 32;
}

impl BitSlice for i64 {
    const BITS: usize = 64;
}

impl BitSlice for u64 {
    const BITS: usize = 64;
}

impl BitSlice for i128 {
    const BITS: usize = 128;
}

impl BitSlice for u128 {
    const BITS: usize = 128;
}
