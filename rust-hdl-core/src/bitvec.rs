#[derive(Debug, Clone, Copy, Hash)]
pub struct BitVec<const N: usize> {
    bits: [bool; N],
}

impl<const N: usize> From<[bool; N]> for BitVec<N> {
    fn from(x: [bool; N]) -> Self {
        BitVec { bits: x }
    }
}

impl<const N: usize> BitVec<N> {
    pub fn all(&self) -> bool {
        for i in 0..N {
            if !self.bits[i] {
                return false;
            }
        }
        true
    }

    pub fn any(&self) -> bool {
        for i in 0..N {
            if self.bits[i] {
                return true;
            }
        }
        false
    }

    pub fn xor(&self) -> bool {
        let mut ret = false;
        for i in 0..N {
            ret = ret ^ self.bits[i];
        }
        ret
    }

    pub fn get_bit(&self, ndx: usize) -> bool {
        assert!(ndx < N);
        self.bits[ndx]
    }

    pub fn replace_bit(&self, ndx: usize, val: bool) -> BitVec<N> {
        let mut t = self.bits.clone();
        t[ndx] = val;
        BitVec { bits: t }
    }

    pub fn resize<const M: usize>(&self) -> BitVec<M> {
        let mut t = [false; M];
        for i in 0..M.min(N) {
            t[i] = self.bits[i];
        }
        BitVec { bits: t }
    }
}

impl<const N: usize> std::ops::Shr<usize> for BitVec<N> {
    type Output = BitVec<N>;

    fn shr(self, rhs: usize) -> Self::Output {
        let mut bits = [false; N];
        for i in rhs..N {
            bits[i - rhs] = self.bits[i];
        }
        Self { bits }
    }
}

impl<const N: usize, const M: usize> std::ops::Shr<BitVec<M>> for BitVec<N> {
    type Output = BitVec<N>;

    fn shr(self, rhs: BitVec<M>) -> Self::Output {
        let rhs_u32: usize = rhs.into();
        self >> rhs_u32
    }
}

impl<const N: usize> std::ops::Shl<usize> for BitVec<N> {
    type Output = BitVec<N>;

    fn shl(self, rhs: usize) -> Self::Output {
        let mut bits = [false; N];
        for i in rhs..N {
            bits[i] = self.bits[i - rhs];
        }
        Self { bits }
    }
}

impl<const N: usize, const M: usize> std::ops::Shl<BitVec<M>> for BitVec<N> {
    type Output = BitVec<N>;

    fn shl(self, rhs: BitVec<M>) -> Self::Output {
        let rhs: usize = rhs.into();
        self << rhs
    }
}

impl<const N: usize> std::ops::BitOr for BitVec<N> {
    type Output = BitVec<N>;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.binop(&rhs, |a, b| a | b)
    }
}

impl<const N: usize> std::ops::BitAnd for BitVec<N> {
    type Output = BitVec<N>;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.binop(&rhs, |a, b| a & b)
    }
}

impl<const N: usize> std::ops::BitXor for BitVec<N> {
    type Output = BitVec<N>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.binop(&rhs, |a, b| a ^ b)
    }
}

impl<const N: usize> std::ops::Not for BitVec<N> {
    type Output = BitVec<N>;

    fn not(self) -> Self::Output {
        let mut bits = [false; N];
        for i in 0..N {
            bits[i] = !self.bits[i];
        }
        Self { bits }
    }
}

// Add with cary
// A  B  C  | X  Co | A ^ (B ^ C) | A B + B C-IN + A C-IN
// 0  0  0  | 0  0  | 0           | 0
// 0  0  1  | 1  0  | 1           | 0
// 0  1  0  | 1  0  | 1           | 0
// 0  1  1  | 0  1  | 0           | 1
// 1  0  0  | 1  0  | 1           | 0
// 1  0  1  | 0  1  | 0           | 1
// 1  1  0  | 0  1  | 0           | 1
// 1  1  1  | 1  1  | 1           | 1
impl<const N: usize> std::ops::Add<BitVec<N>> for BitVec<N> {
    type Output = BitVec<N>;

    fn add(self, rhs: BitVec<N>) -> Self::Output {
        let mut carry = false;
        let mut bits = [false; N];
        for i in 0..N {
            let a = self.bits[i];
            let b = rhs.bits[i];
            let c_i = carry;
            bits[i] = a ^ b ^ c_i;
            carry = (a & b) | (b & c_i) | (a & c_i);
        }
        Self { bits }
    }
}

impl<const N: usize> std::ops::Sub<BitVec<N>> for BitVec<N> {
    type Output = BitVec<N>;

    fn sub(self, rhs: BitVec<N>) -> Self::Output {
        self + !rhs + 1_u32.into()
    }
}

impl<const N: usize> std::cmp::PartialEq for BitVec<N> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..N {
            if self.bits[i] != other.bits[i] {
                return false;
            }
        }
        true
    }
}

impl<const N: usize> std::cmp::PartialOrd for BitVec<N> {
    fn partial_cmp(&self, other: &BitVec<N>) -> Option<std::cmp::Ordering> {
        for i in 0..N {
            let a = self.bits[N - 1 - i];
            let b = other.bits[N - 1 - i];
            if a & !b {
                return Some(std::cmp::Ordering::Greater);
            }
            if !a & b {
                return Some(std::cmp::Ordering::Less);
            }
        }
        Some(std::cmp::Ordering::Equal)
    }
}

impl<const N: usize> BitVec<N> {
    fn binop<T>(&self, rhs: &Self, op: T) -> Self
    where
        T: Fn(&bool, &bool) -> bool,
    {
        let mut bits = [false; N];
        for i in 0..N {
            bits[i] = op(&self.bits[i], &rhs.bits[i]);
        }
        Self { bits }
    }
}

macro_rules! define_vec_from_uint {
    ($name:ident) => {
        impl<const N: usize> From<$name> for BitVec<N> {
            fn from(mut x: $name) -> Self {
                let mut bits = [false; N];
                for i in 0..N {
                    bits[i] = (x & 1) != 0;
                    x = x >> 1;
                }
                Self { bits }
            }
        }
    };
}

define_vec_from_uint!(u8);
define_vec_from_uint!(u16);
define_vec_from_uint!(u32);
define_vec_from_uint!(u64);
define_vec_from_uint!(u128);
define_vec_from_uint!(usize);

macro_rules! define_uint_from_vec {
    ($name:ident, $width: expr) => {
        impl<const N: usize> From<BitVec<N>> for $name {
            fn from(t: BitVec<N>) -> Self {
                let mut x: $name = 0;
                for i in 0..N {
                    x = x << 1;
                    x = x | if t.bits[N - 1 - i] { 1 } else { 0 }
                }
                x
            }
        }
    };
}

define_uint_from_vec!(u8, 8);
define_uint_from_vec!(u16, 16);
define_uint_from_vec!(u32, 32);
define_uint_from_vec!(u64, 64);
define_uint_from_vec!(u128, 128);
#[cfg(target_pointer_width = "64")]
define_uint_from_vec!(usize, 64);
#[cfg(target_pointer_width = "32")]
define_uint_from_vec!(usize, 32);

#[cfg(test)]
mod tests {
    use std::num::Wrapping;

    use crate::bitvec::BitVec;

    #[test]
    fn or_test() {
        let a: BitVec<32> = 45_u32.into();
        let b: BitVec<32> = 10395_u32.into();
        let c = a | b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 45_u32 | 10395_u32)
    }
    #[test]
    fn and_test() {
        let a: BitVec<32> = 45_u32.into();
        let b: BitVec<32> = 10395_u32.into();
        let c = a & b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 45_u32 & 10395_u32)
    }
    #[test]
    fn xor_test() {
        let a: BitVec<32> = 45_u32.into();
        let b: BitVec<32> = 10395_u32.into();
        let c = a ^ b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 45_u32 ^ 10395_u32)
    }
    #[test]
    fn not_test() {
        let a: BitVec<32> = 45_u32.into();
        let c = !a;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, !45_u32);
    }
    #[test]
    fn shr_test() {
        let a: BitVec<32> = 10395_u32.into();
        let c = a >> 4;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10395_u32 >> 4);
    }
    #[test]
    fn shr_test_pair() {
        let a: BitVec<32> = 10395_u32.into();
        let b: BitVec<4> = 4_u32.into();
        let c = a >> b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10395_u32 >> 4);
    }
    #[test]
    fn shl_test() {
        let a: BitVec<32> = 10395_u32.into();
        let c = a << 24;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10395_u32 << 24);
    }
    #[test]
    fn shl_test_pair() {
        let a: BitVec<32> = 10395_u32.into();
        let b: BitVec<4> = 4_u32.into();
        let c = a << b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10395_u32 << 4);
    }
    #[test]
    fn add_works() {
        let a: BitVec<32> = 10234_u32.into();
        let b: BitVec<32> = 19423_u32.into();
        let c = a + b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10234_u32 + 19423_u32);
    }
    #[test]
    fn add_works_with_overflow() {
        let x = 2_042_102_334_u32;
        let y = 2_942_142_512_u32;
        let a: BitVec<32> = x.into();
        let b: BitVec<32> = y.into();
        let c = a + b;
        let c_u32: u32 = c.into();
        assert_eq!(Wrapping(c_u32), Wrapping(x) + Wrapping(y));
    }
    #[test]
    fn sub_works() {
        let x = 2_042_102_334_u32;
        let y = 2_942_142_512_u32;
        let a: BitVec<32> = x.into();
        let b: BitVec<32> = y.into();
        let c = a - b;
        let c_u32: u32 = c.into();
        assert_eq!(Wrapping(c_u32), Wrapping(x) - Wrapping(y));
    }
    #[test]
    fn eq_works() {
        let x = 2_032_142_351_u32;
        let y = 2_942_142_512_u32;
        let a: BitVec<32> = x.into();
        let b: BitVec<32> = x.into();
        let c: BitVec<32> = y.into();
        assert_eq!(a, b);
        assert_ne!(a, c)
    }
    #[test]
    fn all_works() {
        let a: BitVec<48> = 0xFFFF_FFFF_FFFF_u64.into();
        assert!(a.all());
        assert!(a.any());
    }
}
