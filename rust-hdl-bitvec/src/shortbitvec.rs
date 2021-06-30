use std::cmp::Ordering;
use std::num::Wrapping;

pub type ShortType = u32;

pub const SHORT_BITS: usize = 32;

#[derive(Clone, Debug, Copy)]
pub struct ShortBitVec<const N: usize>(ShortType);

impl<const N: usize> ShortBitVec<N> {
    pub fn set_bit(&self, index: usize, val: bool) -> Self {
        assert!(index < N);
        let m: ShortType = 1 << index;
        Self(if val { self.0 | m } else { self.0 & !m })
    }

    pub fn get_bit(&self, index: usize) -> bool {
        self.0 & (1 << index) != 0
    }

    const fn bit_mask() -> ShortType {
        if N == SHORT_BITS {
            !0
        } else {
            (1 << N) - 1
        }
    }

    pub fn mask() -> Self {
        Self::bit_mask().into()
    }

    pub fn any(&self) -> bool {
        self.0 != 0
    }

    pub fn all(&self) -> bool {
        self.0 == Self::bit_mask()
    }
}

impl<const N: usize> From<ShortType> for ShortBitVec<N> {
    fn from(t: ShortType) -> Self {
        Self(t & Self::bit_mask())
    }
}

impl<const N: usize> From<ShortBitVec<N>> for ShortType {
    fn from(t: ShortBitVec<N>) -> ShortType {
        t.0
    }
}

impl<const N: usize> From<ShortBitVec<N>> for usize {
    fn from(t: ShortBitVec<N>) -> usize {
        t.0 as usize
    }
}

impl<const N: usize> std::ops::Add<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn add(self, rhs: ShortBitVec<N>) -> Self::Output {
        (Wrapping(self.0) + Wrapping(rhs.0)).0.into()
    }
}

impl<const N: usize> std::ops::Sub<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn sub(self, rhs: ShortBitVec<N>) -> Self::Output {
        (Wrapping(self.0) + !Wrapping(rhs.0) + Wrapping(1_u32 as ShortType))
            .0
            .into()
    }
}

impl<const N: usize> std::ops::BitOr<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn bitor(self, rhs: ShortBitVec<N>) -> Self::Output {
        (self.0 | rhs.0).into()
    }
}

impl<const N: usize> std::ops::BitAnd<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn bitand(self, rhs: ShortBitVec<N>) -> Self::Output {
        (self.0 & rhs.0).into()
    }
}

impl<const N: usize> std::ops::BitXor<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn bitxor(self, rhs: ShortBitVec<N>) -> Self::Output {
        (self.0 ^ rhs.0).into()
    }
}

impl<const N: usize> std::ops::Not for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn not(self) -> Self::Output {
        (!self.0).into()
    }
}

impl<const N: usize> std::cmp::PartialEq for ShortBitVec<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const N: usize> std::cmp::PartialOrd for ShortBitVec<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<const N: usize> std::ops::Shr<usize> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn shr(self, rhs: usize) -> Self::Output {
        (Wrapping(self.0) >> rhs).0.into()
    }
}

impl<const M: usize, const N: usize> std::ops::Shr<ShortBitVec<M>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn shr(self, rhs: ShortBitVec<M>) -> Self::Output {
        let r: usize = rhs.into();
        self >> r
    }
}

impl<const N: usize> std::ops::Shl<usize> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn shl(self, rhs: usize) -> Self::Output {
        (Wrapping(self.0) << rhs).0.into()
    }
}

impl<const M: usize, const N: usize> std::ops::Shl<ShortBitVec<M>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    fn shl(self, rhs: ShortBitVec<M>) -> Self::Output {
        let r: usize = rhs.into();
        self << r
    }
}
