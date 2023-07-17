use rust_hdl::prelude::{Bits, ToBits};

use crate::bit_slice::BitSlice;

pub struct BitIter<T> {
    val: T,
    index: usize,
}

impl<T> BitIter<T> {
    pub fn new(val: T) -> Self {
        Self { val, index: 0 }
    }
}

impl<T: BitSlice> Iterator for BitIter<T> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < T::BITS {
            let ret = (self.val & T::one()) != T::zero();
            self.val = self.val >> T::one();
            self.index += 1;
            Some(ret)
        } else {
            None
        }
    }
}

impl<const N: usize> Iterator for BitIter<Bits<N>> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < N {
            let ret = self.val.get_bit(self.index);
            self.index += 1;
            Some(ret)
        } else {
            None
        }
    }
}

#[test]
fn test_iter_i8() {
    let x = 0b0100_0101_i8;
    let y = BitIter::new(x);
    assert_eq!(
        y.collect::<Vec<_>>(),
        vec![true, false, true, false, false, false, true, false]
    );
}

#[test]
fn test_bits_iter() {
    let x: Bits<7> = 0b0100_0101_u8.to_bits();
    let y = BitIter::new(x);
    assert_eq!(
        y.collect::<Vec<_>>(),
        vec![true, false, true, false, false, false, true]
    );
}
