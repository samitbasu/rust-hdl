use std::marker::PhantomData;

use crate::bits::bit_cast;
use crate::bits::Bits;
use crate::clock::{Clock, Domain};
use crate::prelude::Synth;
use std::cmp::Ordering;
use std::ops::{Add, BitAnd, Not, Sub};

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Tagged<T: Synth, F: Domain>(pub T, pub PhantomData<F>);

impl<T: Synth, F: Domain> Default for Tagged<T, F> {
    fn default() -> Self {
        Self(T::default(), PhantomData)
    }
}

impl<F: Domain, const N: usize> From<u32> for Tagged<Bits<N>, F> {
    fn from(x: u32) -> Self {
        Tagged(x.into(), PhantomData)
    }
}

impl<F: Domain, const N: usize> From<u8> for Tagged<Bits<N>, F> {
    fn from(x: u8) -> Self {
        Tagged(x.into(), PhantomData)
    }
}

pub fn tagged_bit_cast<F: Domain, const M: usize, const N: usize>(
    x: Tagged<Bits<N>, F>,
) -> Tagged<Bits<M>, F> {
    Tagged(bit_cast::<M, N>(x.0), PhantomData)
}

impl<T: Synth, F: Domain> Tagged<T, F> {
    pub fn raw(self) -> T {
        self.0
    }
}

impl<F: Domain, const N: usize> Tagged<Bits<N>, F> {
    pub fn any(self) -> bool {self.0.any()}
    pub fn all(self) -> bool {self.0.all()}
}

impl<F: Domain> Tagged<bool, F> {
    pub fn any(self) -> bool {self.0}
    pub fn all(self) -> bool {self.0}
}

impl<T: Synth + BitAnd<bool, Output = T>, F: Domain> BitAnd<bool> for Tagged<T, F> {
    type Output = Tagged<T, F>;

    fn bitand(self, rhs: bool) -> Self::Output {
        Tagged(self.0 & rhs, PhantomData)
    }
}

impl<T: Synth + BitAnd<T, Output = T>, F: Domain> BitAnd<Tagged<T, F>> for Tagged<T, F> {
    type Output = Tagged<T, F>;

    fn bitand(self, rhs: Tagged<T, F>) -> Self::Output {
        Tagged(self.0 & rhs.0, PhantomData)
    }
}

impl<F: Domain, const N: usize> Add<Tagged<bool, F>> for Tagged<Bits<N>, F> {
    type Output = Tagged<Bits<N>, F>;

    fn add(self, rhs: Tagged<bool, F>) -> Self::Output {
        Self(self.0 + rhs.0, PhantomData)
    }
}
impl<T: Synth + Add<u32, Output = T>, F: Domain> Add<u32> for Tagged<T, F> {
    type Output = Tagged<T, F>;

    fn add(self, rhs: u32) -> Self::Output {
        Tagged(self.0 + rhs, PhantomData)
    }
}

impl<T: Synth + Sub<u32, Output = T>, F: Domain> Sub<u32> for Tagged<T, F> {
    type Output = Tagged<T, F>;

    fn sub(self, rhs: u32) -> Self::Output {
        Tagged(self.0 - rhs, PhantomData)
    }
}

impl<T: Synth + Add<Output = T>, F: Domain> Add<Tagged<T, F>> for Tagged<T, F> {
    type Output = Tagged<T, F>;

    fn add(self, rhs: Tagged<T, F>) -> Self::Output {
        Tagged(self.0 + rhs.0, PhantomData)
    }
}

impl<T: Synth + Not<Output = T>, F: Domain> Not for Tagged<T, F> {
    type Output = Tagged<T, F>;

    fn not(self) -> Self::Output {
        Tagged(!self.0, PhantomData)
    }
}

impl<F: Domain> From<bool> for Tagged<bool, F> {
    fn from(x: bool) -> Self {
        Tagged(x, PhantomData)
    }
}

impl<F: Domain, const N: usize> From<Bits<N>> for Tagged<Bits<N>, F> {
    fn from(x: Bits<N>) -> Self {
        Tagged(x, PhantomData)
    }
}

impl<T: Synth, F: Domain> PartialEq<T> for Tagged<T, F> {
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

impl<T: Synth + PartialEq<u32>, F: Domain> PartialEq<u32> for Tagged<T, F> {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl<T: Synth + PartialOrd, F: Domain> PartialOrd for Tagged<T, F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<F: Domain> From<Clock> for Tagged<Clock, F> {
    fn from(x: Clock) -> Self {
        Tagged(x, PhantomData)
    }
}
