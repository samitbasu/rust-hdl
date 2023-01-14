use super::bits::Bits;
use crate::bits::{bit_cast, LiteralType, LITERAL_BITS};
use num_bigint::{BigInt, Sign};
use num_traits::cast::ToPrimitive;
use std::fmt::{Debug, Formatter, LowerHex, UpperHex};
use std::num::Wrapping;

pub type SignedLiteralType = i64;
pub const SIGNED_LITERAL_BITS: usize = 64;

#[derive(Clone, Debug, Copy, PartialEq, Default)]
pub struct Signed<const N: usize>(Bits<N>);

pub trait ToSignedBits {
    fn to_signed_bits<const N: usize>(self) -> Signed<N>;
}

impl ToSignedBits for i8 {
    fn to_signed_bits<const N: usize>(self) -> Signed<N> {
        assert!(N <= 8);
        (self as SignedLiteralType).into()
    }
}

impl ToSignedBits for i16 {
    fn to_signed_bits<const N: usize>(self) -> Signed<N> {
        assert!(N <= 16);
        (self as SignedLiteralType).into()
    }
}

impl ToSignedBits for i32 {
    fn to_signed_bits<const N: usize>(self) -> Signed<N> {
        assert!(N <= 32);
        (self as SignedLiteralType).into()
    }
}

impl ToSignedBits for i64 {
    fn to_signed_bits<const N: usize>(self) -> Signed<N> {
        assert!(N <= 64);
        (self as SignedLiteralType).into()
    }
}

impl<const N: usize> LowerHex for Signed<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.bigint(), f)
    }
}

impl<const N: usize> UpperHex for Signed<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.bigint(), f)
    }
}

impl<const N: usize> Signed<N> {
    pub fn min() -> BigInt {
        // Largest negative value is 1000...0 for N bits
        let ret = -Self::max() - 1;
        ret
    }
    pub fn max() -> BigInt {
        // Largest positive value is 0111...1 for N bits
        // Which is 2^N-1
        BigInt::from(2).pow((N - 1) as u32) - 1
    }
    pub fn sign_bit(&self) -> bool {
        self.0.get_bit(N - 1)
    }
    pub fn bigint(&self) -> BigInt {
        let mut ret = BigInt::default();
        if !self.sign_bit() {
            for i in 0..N {
                ret.set_bit(i as u64, self.get_bit(i))
            }
            ret
        } else {
            for i in 0..N {
                ret.set_bit(i as u64, !self.get_bit(i))
            }
            -ret - 1
        }
    }
    pub fn get_bit(&self, ndx: usize) -> bool {
        self.0.get_bit(ndx)
    }
    pub fn get_bits<const M: usize>(&self, index: usize) -> Signed<M> {
        Signed(self.0.get_bits::<M>(index))
    }
    pub fn inner(&self) -> Bits<N> {
        self.0
    }
}

impl<const N: usize> From<BigInt> for Signed<N> {
    fn from(x: BigInt) -> Self {
        assert!(x.bits() <= N as u64);
        if N <= LITERAL_BITS {
            if x.sign() == Sign::Minus {
                -Signed(Bits::from((-x).to_u64().unwrap()))
            } else {
                Signed(Bits::from(x.to_u64().unwrap()))
            }
        } else {
            if x.sign() == Sign::Minus {
                -Signed(Bits::from((-x).to_biguint().unwrap()))
            } else {
                Signed(Bits::from(x.to_biguint().unwrap()))
            }
        }
    }
}

impl<const N: usize> std::ops::Neg for Signed<N> {
    type Output = Signed<N>;

    fn neg(self) -> Self::Output {
        Signed(match self.0 {
            Bits::Short(x) => Bits::Short((Wrapping(0) - Wrapping(x.short())).0.into()),
            Bits::Long(x) => {
                let mut val = [false; N];
                for ndx in 0..N {
                    val[ndx] = !x.get_bit(ndx);
                }
                Bits::Long(val.into()) + 1
            }
        })
    }
}

impl<const N: usize> std::ops::Add<Signed<N>> for Signed<N> {
    type Output = Signed<N>;

    fn add(self, rhs: Signed<N>) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<const N: usize> std::ops::Sub<Signed<N>> for Signed<N> {
    type Output = Signed<N>;

    fn sub(self, rhs: Signed<N>) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul<Signed<16>> for Signed<16> {
    type Output = Signed<32>;

    fn mul(self, rhs: Signed<16>) -> Self::Output {
        Self::Output::from(self.bigint() * rhs.bigint())
    }
}

impl<const N: usize> std::cmp::PartialOrd for Signed<N> {
    fn partial_cmp(&self, other: &Signed<N>) -> Option<std::cmp::Ordering> {
        self.bigint().partial_cmp(&other.bigint())
    }
}

impl<const N: usize> From<SignedLiteralType> for Signed<N> {
    fn from(x: SignedLiteralType) -> Self {
        if x > 0 {
            Self(Bits::from(x as LiteralType))
        } else {
            -Self(Bits::from((-x) as LiteralType))
        }
    }
}

pub fn signed<const N: usize>(x: SignedLiteralType) -> Signed<N> {
    let t: Signed<N> = x.into();
    t
}

pub fn signed_bit_cast<const M: usize, const N: usize>(x: Signed<N>) -> Signed<M> {
    if x.sign_bit() {
        -signed_bit_cast(-x)
    } else {
        Signed(bit_cast(x.0))
    }
}

pub fn signed_cast<const N: usize>(x: Bits<N>) -> Signed<N> {
    Signed(x)
}

pub fn unsigned_cast<const N: usize>(x: Signed<N>) -> Bits<N> {
    x.0
}

pub fn unsigned_bit_cast<const M: usize, const N: usize>(x: Signed<N>) -> Bits<M> {
    bit_cast(x.0)
}

#[cfg(test)]
mod tests {
    use crate::bits::Bits;
    use crate::signed::{signed_bit_cast, unsigned_bit_cast, Signed};
    use num_bigint::BigInt;

    #[test]
    fn test_min_range_correct() {
        assert_eq!(Signed::<8>::min(), i8::MIN.into());
        assert_eq!(Signed::<16>::min(), i16::MIN.into());
        assert_eq!(Signed::<32>::min(), i32::MIN.into());
        assert_eq!(Signed::<64>::min(), i64::MIN.into());
        assert_eq!(Signed::<128>::min(), i128::MIN.into());
    }

    #[test]
    fn test_max_range_correct() {
        assert_eq!(Signed::<8>::max(), i8::MAX.into());
        assert_eq!(Signed::<16>::max(), i16::MAX.into());
        assert_eq!(Signed::<32>::max(), i32::MAX.into());
        assert_eq!(Signed::<64>::max(), i64::MAX.into());
        assert_eq!(Signed::<128>::max(), i128::MAX.into());
    }

    fn run_import_tests<const N: usize>(skip: u32) {
        // Test all possibilities for a 4 bit signed integer.
        let mut q = Signed::<N>::min();
        while q <= Signed::<N>::max() {
            let x: Signed<N> = q.clone().into();
            for i in 0..N {
                assert_eq!(x.get_bit(i), q.bit(i as u64))
            }
            assert_eq!(x.bigint(), q);
            q += skip;
        }
    }

    #[test]
    fn test_signed_import_small() {
        run_import_tests::<5>(1);
    }

    #[test]
    fn test_signed_import_large() {
        run_import_tests::<34>(1 << 16);
    }

    #[test]
    fn time_adds_bigint() {
        let now = std::time::Instant::now();
        for _iter in 0..10 {
            let mut q = BigInt::from(0_u32);
            for _i in 0..1_000_000 {
                q = q + 1;
            }
        }
        let elapsed = std::time::Instant::now() - now;
        println!("Duration: {}", elapsed.as_micros());
    }

    #[test]
    fn time_adds_bitvec() {
        let now = std::time::Instant::now();
        for _iter in 0..10 {
            let mut q = Bits::<40>::from(0);
            for _i in 0..1_000_000 {
                q = q + 1;
            }
        }
        let elapsed = std::time::Instant::now() - now;
        println!("Duration: {}", elapsed.as_micros());
    }

    #[test]
    fn time_adds_bitvec_small() {
        let now = std::time::Instant::now();
        for _iter in 0..10 {
            let mut q = Bits::<16>::from(0);
            for _i in 0..1_000_000 {
                q = q + 1;
            }
        }
        let elapsed = std::time::Instant::now() - now;
        println!("Duration: {}", elapsed.as_micros());
    }

    #[test]
    fn signed_displays_correctly() {
        println!("{:x}", Signed::<16>::from(-23));
    }

    #[test]
    fn test_signed_cast() {
        let x = Signed::<16>::from(-23);
        let y: Signed<40> = signed_bit_cast(x);
        assert_eq!(y, Signed::<40>::from(-23));
    }

    #[test]
    fn test_unsigned_cast() {
        let x = Signed::<16>::from(-23);
        let y: Bits<16> = unsigned_bit_cast(x);
        assert_eq!(y, Bits::<16>::from(0xFFe9))
    }

    #[test]
    fn test_neg_operator() {
        let x = Signed::<16>::from(23);
        assert_eq!(x.bigint(), BigInt::from(23));
        let x = -x;
        assert_eq!(x.bigint(), BigInt::from(-23));
        let x = -x;
        assert_eq!(x.bigint(), BigInt::from(23));
    }

    #[test]
    fn test_neg_operator_larger() {
        let t: BigInt = BigInt::from(23) << 32;
        let x = Signed::<48>::from(t.clone());
        assert_eq!(x.bigint(), t.clone());
        let x = -x;
        assert_eq!(x.bigint(), -t.clone());
        let x = -x;
        assert_eq!(x.bigint(), t.clone());
    }
}
