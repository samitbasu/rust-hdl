use crate::bitvec::BitVec;
use crate::shortbitvec::{ShortBitVec, ShortType, SHORT_BITS};
use crate::synth::VCDValue;
use std::cmp::Ordering;
use std::fmt::{Binary, Debug, Formatter, LowerHex, UpperHex};
use std::hash::Hasher;
use std::num::Wrapping;

// This comes with a few invariants that must be maintained for short representation
// The short value must be less than 2^N
// N <= SHORT_BITS --> Short repr, otherwise Long repr

// Compute the minimum number of bits to represent a container with t items.
// This is basically ceil(log2(t)) as a constant (compile time computable) function.
pub const fn clog2(t: usize) -> usize {
    let mut p = 0;
    let mut b = 1;
    while b < t {
        p += 1;
        b *= 2;
    }
    p
}

#[test]
fn test_clog2_is_correct() {
    assert_eq!(clog2(1024), 10);
}

#[derive(Clone, Debug, Copy)]
pub enum Bits<const N: usize> {
    Short(ShortBitVec<N>),
    Long(BitVec<N>),
}

impl<const N: usize> Binary for Bits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..N {
            if self.get_bit(N - 1 - i) {
                write!(f, "1")?;
            } else {
                write!(f, "0")?;
            }
        }
        Ok(())
    }
}

impl<const N: usize> LowerHex for Bits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let m: usize = N + (4 - (N % 4)) % 4; // Round up to an integer number of nibbles
        let digits: usize = m / 4;
        for digit in 0..digits {
            let nibble: Bits<4> = self.get_bits(4 * (digits - 1 - digit));
            let nibble_u8: u8 = nibble.into();
            std::fmt::LowerHex::fmt(&nibble_u8, f)?;
        }
        Ok(())
    }
}

impl<const N: usize> UpperHex for Bits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let m: usize = N + (4 - (N % 4)) % 4; // Round up to an integer number of nibbles
        let digits: usize = m / 4;
        for digit in 0..digits {
            let nibble: Bits<4> = self.get_bits(4 * (digits - 1 - digit));
            let nibble_u8: u8 = nibble.into();
            std::fmt::UpperHex::fmt(&nibble_u8, f)?;
        }
        Ok(())
    }
}

#[inline(always)]
pub fn bits<const N: usize>(x: u128) -> Bits<N> {
    let t: Bits<N> = x.into();
    t
}

#[inline(always)]
pub fn bit_cast<const M: usize, const N: usize>(x: Bits<N>) -> Bits<M> {
    match x {
        Bits::Short(t) => {
            let t: ShortType = t.into();
            let k: Bits<M> = t.into();
            k
        }
        Bits::Long(t) => {
            if M > SHORT_BITS {
                Bits::Long(t.resize())
            } else {
                let k: ShortType = t.into();
                Bits::Short(k.into())
            }
        }
    }
}

impl<const N: usize> Into<VCDValue> for Bits<N> {
    fn into(self) -> VCDValue {
        if N == 1 {
            if self.get_bit(0) {
                VCDValue::Single(vcd::Value::V1)
            } else {
                VCDValue::Single(vcd::Value::V0)
            }
        } else {
            let mut x = vec![];
            for i in 0..N {
                if self.get_bit(N - 1 - i) {
                    x.push(vcd::Value::V1)
                } else {
                    x.push(vcd::Value::V0)
                }
            }
            VCDValue::Vector(x)
        }
    }
}

impl<const N: usize> Bits<N> {
    #[inline(always)]
    pub fn any(&self) -> bool {
        match self {
            Bits::Short(x) => x.any(),
            Bits::Long(x) => x.any(),
        }
    }

    #[inline(always)]
    pub fn all(&self) -> bool {
        match self {
            Bits::Short(x) => x.all(),
            Bits::Long(x) => x.all(),
        }
    }

    #[inline(always)]
    pub fn xor(&self) -> bool {
        match self {
            Bits::Short(x) => x.xor(),
            Bits::Long(x) => x.xor(),
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Bits::Short(x) => x.short() as usize,
            Bits::Long(_x) => panic!("Cannot map long bit vector to index type"),
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        N
    }

    // Warning!! this can overflow
    pub fn count() -> usize {
        1 << N
    }

    #[inline(always)]
    pub fn get_bit(&self, index: usize) -> bool {
        assert!(index < N);
        match self {
            Bits::Short(x) => x.get_bit(index),
            Bits::Long(x) => x.get_bit(index),
        }
    }

    pub fn replace_bit(&self, index: usize, val: bool) -> Self {
        assert!(index < N);
        match self {
            Bits::Short(x) => Bits::Short(x.replace_bit(index, val)),
            Bits::Long(x) => Bits::Long(x.replace_bit(index, val)),
        }
    }

    #[inline(always)]
    pub fn get_bits<const M: usize>(&self, index: usize) -> Bits<M> {
        assert!(index <= N);
        bit_cast::<M, N>(*self >> index)
    }

    #[inline(always)]
    pub fn set_bits<const M: usize>(&mut self, index: usize, rhs: Bits<M>) {
        assert!(index <= N);
        assert!(index + M <= N);
        let mask = !(bit_cast::<N, M>(Bits::<M>::mask()) << index);
        let masked = *self & mask;
        let replace = bit_cast::<N, M>(rhs) << index;
        *self = masked | replace
    }

    #[inline(always)]
    pub fn mask() -> Bits<N> {
        if N <= SHORT_BITS {
            Bits::Short(ShortBitVec::<N>::mask())
        } else {
            Bits::Long([true; N].into())
        }
    }

    pub const fn width() -> usize {
        N
    }
}

impl From<bool> for Bits<1> {
    #[inline(always)]
    fn from(x: bool) -> Self {
        if x {
            1_usize.into()
        } else {
            0_usize.into()
        }
    }
}

impl Into<bool> for Bits<1> {
    #[inline(always)]
    fn into(self) -> bool {
        let p: usize = self.into();
        if p == 0 {
            false
        } else {
            true
        }
    }
}

macro_rules! define_from_uint {
    ($name:ident, $width:expr) => {
        impl<const N: usize> From<Wrapping<$name>> for Bits<N> {
            fn from(x: Wrapping<$name>) -> Self {
                x.0.into()
            }
        }

        impl<const N: usize> From<$name> for Bits<N> {
            #[inline(always)]
            fn from(x: $name) -> Self {
                if N > SHORT_BITS {
                    let y: BitVec<N> = x.into();
                    Bits::Long(y)
                } else {
                    Bits::Short((x as ShortType).into())
                }
            }
        }

        impl<const N: usize> From<Bits<N>> for $name {
            #[inline(always)]
            fn from(x: Bits<N>) -> Self {
                assert!(N <= $width);
                match x {
                    Bits::Short(t) => {
                        let p: ShortType = t.into();
                        p as $name
                    }
                    Bits::Long(t) => t.into(),
                }
            }
        }
    };
}

define_from_uint!(u8, 8);
define_from_uint!(u16, 16);
define_from_uint!(u32, 32);
define_from_uint!(u64, 64);
define_from_uint!(u128, 128);

#[cfg(target_pointer_width = "64")]
define_from_uint!(usize, 64);
#[cfg(target_pointer_width = "32")]
define_from_uint!(usize, 32);

#[inline(always)]
fn binop<Tshort, TLong, const N: usize>(
    a: Bits<N>,
    b: Bits<N>,
    short_op: Tshort,
    long_op: TLong,
) -> Bits<N>
where
    Tshort: Fn(ShortBitVec<N>, ShortBitVec<N>) -> ShortBitVec<N>,
    TLong: Fn(BitVec<N>, BitVec<N>) -> BitVec<N>,
{
    match a {
        Bits::Short(x) => match b {
            Bits::Short(y) => Bits::Short(short_op(x, y)),
            _ => {
                unreachable!()
            }
        },
        Bits::Long(x) => match b {
            Bits::Long(y) => Bits::Long(long_op(x, y)),
            _ => {
                unreachable!()
            }
        },
    }
}

macro_rules! op {
    ($func: ident, $method: ident, $op: tt) => {
        impl<const N: usize> std::ops::$method<Bits<N>> for Bits<N> {
            type Output = Bits<N>;

            #[inline(always)]
            fn $func(self, rhs: Bits<N>) -> Self::Output {
                binop(self, rhs, |a, b| a $op b, |a, b| a $op b)
            }
        }

        impl<const N: usize> std::ops::$method<usize> for Bits<N> {
            type Output = Bits<N>;

            #[inline(always)]
            fn $func(self, rhs: usize) -> Self::Output {
                binop(self, rhs.into(), |a, b| a $op b, |a, b| a $op b)
            }
        }

        impl<const N: usize> std::ops::$method<u32> for Bits<N> {
            type Output = Bits<N>;

            #[inline(always)]
            fn $func(self, rhs: u32) -> Self::Output {
                binop(self, rhs.into(), |a, b| a $op b, |a, b| a $op b)
            }
        }

        impl<const N: usize> std::ops::$method<Bits<N>> for usize {
            type Output = Bits<N>;

            #[inline(always)]
            fn $func(self, rhs: Bits<N>) -> Self::Output {
                binop(self.into(), rhs.into(), |a, b| a $op b, |a, b| a $op b)
            }
        }
    }
}

op!(add, Add, +);
op!(sub, Sub, -);
op!(bitor, BitOr, |);
op!(bitand, BitAnd, &);
op!(bitxor, BitXor, ^);
op!(shr, Shr, >>);
op!(shl, Shl, <<);

impl<const N: usize> std::default::Default for Bits<N> {
    fn default() -> Bits<N> {
        bits::<N>(0)
    }
}

impl<const N: usize> std::ops::Not for Bits<N> {
    type Output = Bits<N>;

    fn not(self) -> Self::Output {
        match self {
            Bits::Short(x) => Bits::Short(!x),
            Bits::Long(x) => Bits::Long(!x),
        }
    }
}

impl<const N: usize> std::cmp::Ord for Bits<N> {
    fn cmp(&self, other: &Bits<N>) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

// TODO - add the usize variants to this
impl<const N: usize> std::cmp::PartialOrd<Bits<N>> for Bits<N> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Bits<N>) -> Option<Ordering> {
        match self {
            Bits::Short(x) => match other {
                Bits::Short(y) => x.partial_cmp(y),
                _ => panic!("Short Long case"),
            },
            Bits::Long(x) => match other {
                Bits::Long(y) => x.partial_cmp(y),
                _ => panic!("Long short case"),
            },
        }
    }
}

// TODO - add the usize variants to this
impl<const N: usize> std::cmp::PartialEq<Bits<N>> for Bits<N> {
    #[inline(always)]
    fn eq(&self, other: &Bits<N>) -> bool {
        match self {
            Bits::Short(x) => match other {
                Bits::Short(y) => x == y,
                _ => panic!("Short Long case"),
            },
            Bits::Long(x) => match other {
                Bits::Long(y) => x == y,
                _ => panic!("Long Short case"),
            },
        }
    }
}

macro_rules! partial_eq_with_uint {
    ($kind: ty) => {
        impl<const N: usize> std::cmp::PartialEq<$kind> for Bits<N> {
            #[inline(always)]
            fn eq(&self, other: &$kind) -> bool {
                let other_as_bits: Bits<N> = (*other).into();
                self.eq(&other_as_bits)
            }
        }
    };
}

partial_eq_with_uint!(u8);
partial_eq_with_uint!(u16);
partial_eq_with_uint!(u32);
partial_eq_with_uint!(u64);
partial_eq_with_uint!(u128);
partial_eq_with_uint!(usize);

impl std::cmp::PartialEq<bool> for Bits<1> {
    #[inline(always)]
    fn eq(&self, other: &bool) -> bool {
        self.get_bit(0) == *other
    }
}

impl<const N: usize> std::cmp::Eq for Bits<N> {}

impl<const N: usize> std::hash::Hash for Bits<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Bits::Short(t) => t.hash(state),
            Bits::Long(t) => t.hash(state),
        }
    }
}

impl<const N: usize> std::ops::Add<bool> for Bits<N> {
    type Output = Bits<N>;

    fn add(self, rhs: bool) -> Self::Output {
        if rhs {
            self + Bits::<N>::from(1_u8)
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::Wrapping;

    use crate::bits::{bit_cast, bits, clog2, Bits};

    #[test]
    fn test_short_from_u8() {
        let x: Bits<4> = 135_u8.into();
        let y: u8 = x.into();
        assert_eq!(y, 135 & (0x0F));
    }

    #[test]
    fn test_short_from_u16() {
        let x: Bits<12> = 14323_u16.into();
        let y: u16 = x.into();
        assert_eq!(y, 14323 & (0x0FFF));
    }

    #[test]
    fn test_short_from_u32() {
        let x: Bits<100> = 12434234_u128.into();
        let y: u128 = x.into();
        assert_eq!(y, 12434234_u128);
    }

    #[test]
    fn or_test() {
        let a: Bits<32> = 45_u32.into();
        let b: Bits<32> = 10395_u32.into();
        let c = a | b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 45_u32 | 10395_u32)
    }
    #[test]
    fn and_test() {
        let a: Bits<32> = 45_u32.into();
        let b: Bits<32> = 10395_u32.into();
        let c = a & b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 45_u32 & 10395_u32)
    }
    #[test]
    fn xor_test() {
        let a: Bits<32> = 45_u32.into();
        let b: Bits<32> = 10395_u32.into();
        let c = a ^ b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 45_u32 ^ 10395_u32)
    }
    #[test]
    fn not_test() {
        let a: Bits<32> = 45_u32.into();
        let c = !a;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, !45_u32);
    }
    #[test]
    fn shr_test() {
        let a: Bits<32> = 10395_u32.into();
        let c: Bits<32> = a >> 4_u32;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10395_u32 >> 4);
    }
    #[test]
    fn shr_test_pair() {
        let a: Bits<32> = 10395_u32.into();
        let b: Bits<32> = 4_u32.into();
        let c = a >> b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10395_u32 >> 4);
    }
    #[test]
    fn shl_test() {
        let a: Bits<32> = 10395_u32.into();
        let c = a << 24_u32;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10395_u32 << 24);
    }
    #[test]
    fn shl_test_pair() {
        let a: Bits<32> = 10395_u32.into();
        let b: Bits<32> = 4_u32.into();
        let c = a << b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10395_u32 << 4);
    }
    #[test]
    fn add_works() {
        let a: Bits<32> = 10234_u32.into();
        let b: Bits<32> = 19423_u32.into();
        let c = a + b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10234_u32 + 19423_u32);
    }
    #[test]
    fn add_int_works() {
        let a: Bits<32> = 10234_u32.into();
        let b = 19423_u32;
        let c: Bits<32> = a + b;
        let c_u32: u32 = c.into();
        assert_eq!(c_u32, 10234_u32 + 19423_u32);
    }
    #[test]
    fn add_works_with_overflow() {
        let x = 2_042_102_334_u32;
        let y = 2_942_142_512_u32;
        let a: Bits<32> = x.into();
        let b: Bits<32> = y.into();
        let c = a + b;
        let c_u32: u32 = c.into();
        assert_eq!(Wrapping(c_u32), Wrapping(x) + Wrapping(y));
    }
    #[test]
    fn sub_works() {
        let x = 2_042_102_334_u32;
        let y = 2_942_142_512_u32;
        let a: Bits<32> = x.into();
        let b: Bits<32> = y.into();
        let c = a - b;
        let c_u32: u32 = c.into();
        assert_eq!(Wrapping(c_u32), Wrapping(x) - Wrapping(y));
    }
    #[test]
    fn sub_int_works() {
        let x = 2_042_102_334_u32;
        let y = 2_942_142_512_u32;
        let a: Bits<32> = x.into();
        let b: u32 = y.into();
        let c = a - b;
        let c_u32: u32 = c.into();
        assert_eq!(Wrapping(c_u32), Wrapping(x) - Wrapping(y));
    }
    #[test]
    fn eq_works() {
        let x = 2_032_142_351_u32;
        let y = 2_942_142_512_u32;
        let a: Bits<32> = x.into();
        let b: Bits<32> = x.into();
        let c: Bits<32> = y.into();
        assert_eq!(a, b);
        assert_ne!(a, c)
    }
    #[test]
    fn all_works() {
        let a: Bits<48> = 0xFFFF_FFFF_FFFF_u64.into();
        assert!(a.all());
        assert!(a.any());
    }
    #[test]
    fn mask_works() {
        let a: Bits<48> = 0xFFFF_FFFF_FFFF_u64.into();
        let b = Bits::<48>::mask();
        assert_eq!(a, b);
        let a: Bits<16> = 0xFFFF_u64.into();
        let b = Bits::<16>::mask();
        assert_eq!(a, b)
    }
    #[test]
    fn get_bit_works() {
        // 0101 = 5
        let a = bits::<48>(0xFFFF_FFFF_FFF5);
        assert!(a.get_bit(0));
        assert!(!a.get_bit(1));
        assert!(a.get_bit(2));
        assert!(!a.get_bit(3));
        let c = bits::<5>(3);
        assert!(!a.get_bit(c.into()));
    }
    #[test]
    fn test_bit_cast_short() {
        let a = bits::<8>(0xFF);
        let b: Bits<16> = bit_cast(a);
        assert_eq!(b, bits::<16>(0xFF));
        let c: Bits<4> = bit_cast(a);
        assert_eq!(c, bits::<4>(0xF));
    }
    #[test]
    fn test_bit_cast_long() {
        let a = bits::<48>(0xabcd_dead_cafe_babe);
        let b: Bits<44> = bit_cast(a);
        assert_eq!(b, bits::<44>(0xbcd_dead_cafe_babe));
        let b: Bits<32> = bit_cast(a);
        assert_eq!(b, bits::<32>(0xcafe_babe));
    }
    #[test]
    fn test_bit_extract_long() {
        let a = bits::<48>(0xabcd_dead_cafe_babe);
        let b: Bits<44> = a.get_bits(4);
        assert_eq!(b, bits::<44>(0xabcd_dead_cafe_bab));
        let b: Bits<32> = a.get_bits(16);
        assert_eq!(b, bits::<32>(0xdead_cafe));
    }
    #[test]
    fn test_set_bit() {
        let a = bits::<48>(0xabcd_dead_cafe_babe);
        let mut b = a;
        for i in 4..8 {
            b = b.replace_bit(i, false)
        }
        assert_eq!(b, bits::<48>(0xabcd_dead_cafe_ba0e));
    }
    #[test]
    fn test_set_bits() {
        let a = bits::<16>(0xdead);
        let b = bits::<4>(0xf);
        let mut c = a.clone();
        c.set_bits(4, b);
        assert_eq!(c, bits::<16>(0xdefd));
        let a = bits::<48>(0xabcd_dead_cafe_babe);
        let b = bits::<8>(0xde);
        let mut c = a.clone();
        c.set_bits(16, b);
        assert_eq!(c, bits::<48>(0xabcd_dead_cade_babe));
    }
    #[test]
    fn test_constants_and_bits() {
        let a = bits::<16>(0xdead);
        let b = a + 1_u32;
        let c = 1_usize + a;
        println!("{:x}", b);
        assert_eq!(b, bits::<16>(0xdeae));
        assert_eq!(b, c);
    }
    #[test]
    fn test_clog2() {
        const A_WIDTH: usize = clog2(250);
        let a = bits::<A_WIDTH>(153);
        println!("{:x}", a);
        assert_eq!(a.len(), 8);
        assert_eq!(clog2(1024), 10);
    }
    #[test]
    fn test_clog2_inline() {
        const A_WIDTH: usize = clog2(1000);
        let a = bits::<A_WIDTH>(1023);
        assert_eq!(a.len(), 10);
    }
    #[test]
    fn test_default() {
        const N: usize = 128;
        let a = Bits::<N>::default();
        assert_eq!(a, bits(0));
    }
    #[test]
    fn test_compare() {
        let a = bits::<16>(35);
        let b = bits::<16>(100);
        assert_ne!(a, b);
        assert!(a < b);
        assert!(b > a);
    }
    #[test]
    fn test_compare_long() {
        let a = bits::<160>(35);
        let b = bits::<160>(100);
        assert_ne!(a, b);
        assert!(a < b);
        assert!(b > a);
    }
}

pub type Bit = bool;
