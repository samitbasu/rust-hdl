use num_traits::Num;
use rust_hdl::prelude::Bits;

pub struct RevBitIter<T> {
    val: T,
    index: u64,
    selector: T,
}

pub trait RevBitIterable:
    Num
    + Copy
    + std::ops::Shl<u64, Output = Self>
    + std::ops::BitAnd<Output = Self>
    + std::ops::Shr<u64, Output = Self>
{
}

impl<T: RevBitIterable> RevBitIter<T> {
    pub fn new(val: T, bits: u64) -> Self {
        Self {
            val,
            index: bits,
            selector: (T::one() << (bits - 1)),
        }
    }
}

impl<T: RevBitIterable> Iterator for RevBitIter<T> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }
        let next_bit = self.val & self.selector != T::zero();
        self.selector = self.selector >> 1;
        self.index -= 1;
        Some(next_bit)
    }
}

impl RevBitIterable for u8 {}
impl RevBitIterable for u16 {}
impl RevBitIterable for u32 {}
impl RevBitIterable for u64 {}
impl RevBitIterable for u128 {}

#[test]
fn test_rev_bit_iter_for_u32() {
    let mut iter = RevBitIter::new(0xDEAD_BEEF_u32, 32);
    let bits = iter.collect::<Vec<_>>();
    let mut accum = 0_u32;
    for bit in bits {
        accum = (accum << 1) | bit as u32;
    }
    assert_eq!(accum, 0xDEAD_BEEF_u32)
}
