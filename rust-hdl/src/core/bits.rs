//! The [Bits] type is used to capture values with arbitrarily large (but known) bit length
//!
//! One significant difference between hardware design and software programming is the need
//! (and indeed ability) to easily manipulate collections of bits that are of various lengths.
//! While Rust has built in types to represent 8, 16, 32, 64, and 128 bits (at the time of this
//! writing, anyway), it is difficult to represent a 5 bit type.  Or a 256 bit type.  Or indeed
//! any bit length that differs from one of the supported values.
//!
//! In hardware design, the bit size is nearly always unusual, as bits occupy physical space,
//! and as a result, as a logic designer, you will intentionally use the smallest number of
//! bits needed to capture a value.  For example, if you are reading a single nibble at a
//! time from a bus, this is clearly a 4 bit value, and storing it in a `u8` is a waste of
//! space and resources.
//!
//! To model this behavior in RustHDL, we have the [Bits] type, which attempts to be as close
//! as possible to a hardware bit vector.  The size must be known at compile time, and there is
//! some internal optimization for short bitvectors being represented efficiently, but ideally
//! you should be able to think of it as a bit of arbitrary length.  Note that the [Bits]
//! type is `Copy`, which is quite important.  This means in your RustHDL code, you can freely
//! copy and assign bitvectors without worrying about the borrow checker or trying to call
//! `clone` in the midst of your HDL.
//!
//! For the most part, the [Bits] type is meant to act like a `u32` or `u128` type as far
//! as your code is concerned.  But the emulation of built-in types is not perfect, and
//! you may struggle with them a bit.
//!
//! # Constructing [Bits]
//! There are several ways to construct a [Bits] type.  It includes an implementation of
//! the [Default](std::default::Default), trait, so if you need a zero value, you can use
//! that form:
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<50> = Default::default();
//! ```
//! This will construct a length 50 bit vector that is initialized to all `0`.
//!
//! You can also convert from literals into bit vectors using the [From] and [Into] traits,
//! provided the literals are of the `u64` type.
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<50> = 0xBEEF.into();
//! ```
//!
//! In some cases, Rust complains about literals, and you may need to provide a suffix:
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<50> = 0xDEAD_BEEF_u64.into();
//! ```
//! However, in most cases, you can leave literals suffix-free, and Rust will automatically
//! determine the type from the context.
//!
//! You can construct a larger constant using the [bits] function.  If you have a literal of up to
//! 128 bits, it provides a functional form
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<200> = bits(0xDEAD_BEEE); // Works for up to 128 bit constants.
//! ```
//!
//! There is also the [ToBits] trait, which is implemented on the basic unsigned integer types.
//! This trait allows you to handily convert from different integer values
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<10> = 32_u8.to_bits();
//! ```
//!
//!
//! # Operations
//! Only a subset of operations are defined for [Bits].  These are the operations that can
//! be synthesized in hardware without surprises (generally speaking).  In Rust, you can
//! operate between [Bits] types and other [Bits] of the _same width_, or you can
//! use integer literals.  *Be careful!* Especially when manipulating signed quantities.  Use the
//! [Signed](crate::core::signed::Signed) type for those.
//!
//! ## Addition
//! You can perform wrapping addition using the `+` operator.
//! Here are some simple examples of addition. First the version using a literal
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<200> = bits(0xDEAD_BEEE);
//! let y: Bits<200> = x + 1;
//! assert_eq!(y, bits(0xDEAD_BEEF));
//! ```
//!
//! And now a second example that uses two [Bits] values
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<40> = bits(0xDEAD_0000);
//! let y: Bits<40> = bits(0x0000_CAFE);
//! let z = x + y;
//! assert_eq!(z, bits(0xDEAD_CAFE));
//! ```
//!
//! Note that the addition operator is _silently wrapping_.  In other words the carry
//! bit is discarded silently (again - this is what hardware typically does).  So you
//! may find this result surprising:
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<40> = bits(0xFF_FFFF_FFFF);
//! let y = x + 1;
//! assert_eq!(y, bits(0));
//! ```
//!
//! In this case, the addition of 1 caused [x] to wrap to all zeros.  This is totally normal,
//! and what one would expect from hardware addition (without a carry).  If you _need_ the
//! carry bit, then the solution is to first cast to 1 higher bit, and then add, or alternately,
//! to compute the carry directly.
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<40> = bits(0xFF_FFFF_FFFF);
//! let y = bit_cast::<41, 40>(x) + 1;
//! assert_eq!(y, bits(0x100_0000_0000));
//! ```
//!
//! The order of the arguments does not matter.  The bit width of the calculation will be
//! determined by the [Bits] width.
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x : Bits<25> = bits(0xCAFD);
//! let y = 1 + x;
//! assert_eq!(y, bits(0xCAFE));
//! ```
//!
//! However, you cannot combine two different width [Bits] values in a single expression.
//! ```compile_fail
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<20> = bits(0x1234);
//! let y: Bits<21> = bits(0x5123);
//! let z = x + y; // Won't compile!
//! ```
//!
//! ## Subtraction
//! Hardware subtraction is defined using 2-s complement representation for negative numbers.
//! This is pretty much a universal standard for representing negative numbers in binary, and
//! has the added advantage that a hardware subtractor can be built from an adder and some basic
//! gates.  Subtraction operates much like the [Wrapping] class.  Note that overflow and underflow
//! are _not_ detected in RustHDL (nor are they detected in most hardware implementations either).
//!
//! Here is a simple example with a literal and subtraction that does not cause udnerflow
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<40> = bits(0xDEAD_BEF0);
//! let y = x - 1;
//! assert_eq!(y, bits(0xDEAD_BEEF));
//! ```
//!
//! When values underflow, the representation is still valid as a 2-s complement number.  For
//! example,
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<16> = bits(0x40);
//! let y: Bits<16> = bits(0x60);
//! let z = x - y;
//! assert_eq!(z, bits(0xFFFF-0x20+1));
//! ```
//!
//! Here, we compare the value of `z` with `0xFFFF-0x20+1` which is the 2-s complement
//! representation of `-0x20`.
//!
//! You can also put the literal on the left side of the subtraction expression, as expected.  The
//! bitwidth of the computation will be driven by the width of the [Bits] in the expression.
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x = bits::<32>(0xBABE);
//! let z = 0xB_BABE - x;
//! assert_eq!(z, bits(0xB_0000));
//! ```
//!
//! ## Bitwise And
//!
//! You can combine [Bits] using the and operator `&`.  In general, avoid using the shortcut
//! logical operator `&&`, since this operator is really only defined for logical (scalar) values
//! of type `bool`.
//!
//! ```
//! # use  rust_hdl::core::prelude::*;
//! let x: Bits<32> = bits(0xDEAD_BEEF);
//! let y: Bits<32> = bits(0xFFFF_0000);
//! let z = x & y;
//! assert_eq!(z, bits(0xDEAD_0000));
//! ```
//!
//! Of course, you can also use a literal value in the `and` operation.
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<32> = bits(0xDEAD_BEEF);
//! let z = x & 0x0000_FFFF;
//! assert_eq!(z, bits(0xBEEF))
//! ```
//!
//! and similarly, the literal can appear on the left of the `and` expression.
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<32> = bits(0xCAFE_BEEF);
//! let z = 0xFFFF_0000 & x;
//! assert_eq!(z, bits(0xCAFE_0000));
//! ```
//!
//! Just like all other binary operations, you cannot mix widths (unless one of the
//! values is a literal).
//! ```compile_fail
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<16> = bits(0xFEED_FACE);
//! let y: Bits<17> = bits(0xABCE);
//! let z = x & y; // Won't compile!
//! ```
//!
//! ## Bitwise Or
//!
//! There is also a bitwise-OR operation using the `|` symbol.  Note that the logical OR
//! (or shortcut OR) operator `||` is not supported for [Bits], as it is only defined for
//! scalar boolean values.
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x : Bits<32> = bits(0xBEEF_0000);
//! let y : Bits<32> = bits(0x0000_CAFE);
//! let z = x | y;
//! assert_eq!(z, bits(0xBEEF_CAFE));
//! ```
//!
//! You can also use literals
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x : Bits<32> = bits(0xBEEF_0000);
//! let z = x | 0x0000_CAFE;
//! assert_eq!(z, bits(0xBEEF_CAFE));
//! ```
//!
//! The caveat about mixing [Bits] of different widths still applies.
//!
//! ## Bitwise Xor
//!
//! There is a bitwise-Xor operation using the `^` operator.  This will compute the
//! bitwise exclusive OR of the two values.
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x : Bits<32> = bits(0xCAFE_BABE);
//! let y : Bits<32> = bits(0xFF00_00FF);
//! let z = y ^ x;
//! let w = z ^ y; // XOR applied twice is a null-op
//! assert_eq!(w, x);
//! ```
//!
//! ## Bitwise comparison
//!
//! The equality operator `==` can compare two [Bits] for bit-wise equality.
//!
//!```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<16> = bits(0x5ea1);
//! let y: Bits<16> = bits(0xbadb);
//! assert_eq!(x == y, false)
//!```
//!
//! Again, it is a compile time failure to attempt to compare [Bits] of different
//! widths.
//!
//!```compile_fail
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<15> = bits(52);
//! let y: Bits<16> = bits(32);
//! let z = x == y; // Won't compile - bit widths must match
//!```
//!
//! You can compare to literals, as they will automatically extended (or truncated) to match the
//! bitwidth of the [Bits] value.
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x : Bits<16> = bits(32);
//! let z = x == 32;
//! let y = 32 == x;
//! assert!(z);
//! assert!(y);
//! ```
//!
//! ## Unsigned comparison
//!
//! The [Bits] type only supports unsigned comparisons.  If you compare a [Bits] value
//! to a signed integer, it will first convert the signed integer into 2s complement
//! representation and then perform an unsigned comparison.  That is most likely _not_ what
//! you want.  However, until there is full support for signed integer computations, that is
//! the behavior you get.  Hardware signed comparisons require more circuitry and logic
//! than unsigned comparisons, so the rationale is to not inadvertently bloat your hardware
//! designs with sign-aware circuitry when you don't explicitly invoke it.  If you want signed
//! values, use [Signed].
//!
//! Here are some simple examples.
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<16> = bits(52);
//! let y: Bits<16> = bits(13);
//! assert!(y < x)
//! ```
//!
//! We can also compare with literals, which RustHDL will expand out to match the bit width
//! of the [Bits] being compared to.
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<16> = bits(52);
//! let y = x < 135;  // Converts the 135 to a Bits<16> and then compares
//! assert!(y)
//! ```
//!
//! ## Shift Left
//!
//! RustHDl supports left shift bit operations using the `<<` operator.
//! Bits that shift off the left end of
//! the bit vector (most significant bits).
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<16> = bits(0xDEAD);
//! let y = x << 8;
//! assert_eq!(y, bits(0xAD00));
//! ```
//!
//! ## Shift Right
//!
//! Right shifting is also supported using the `>>` operator.
//! Bits that shift off the right end of the
//! the bit vector (least significant bits).
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<16> = bits(0xDEAD);
//! let y = x >> 8;
//! assert_eq!(y, bits(0x00DE));
//! ```

use crate::core::bitvec::BitVec;
use crate::core::short_bit_vec::{ShortBitVec, ShortType, SHORT_BITS};
use crate::core::synth::VCDValue;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::cmp::Ordering;
use std::fmt::{Binary, Debug, Formatter, LowerHex, UpperHex};
use std::hash::Hasher;
use std::num::Wrapping;

// This comes with a few invariants that must be maintained for short representation
// The short value must be less than 2^N
// N <= SHORT_BITS --> Short repr, otherwise Long repr

/// The [LiteralType] is used to set the type for literals that appear in RustHDL
/// expressions.  Because of how Rust's type inference currently works, an expression
/// like
/// ```
/// # use rust_hdl::core::prelude::*;
///
/// let x: Bits<32> = 0xDEADBEEF.into();
/// let y : Bits<32> = x + 1;
/// ```
/// only works if Rust can unambiguously assign a type to the literal (either `DEADBEEF` or `1`).
/// In earlier versions of RustHDL, this required adding a suffix to the literal like `0xDEADBEEF_u32`,
/// but those suffixes in turn littered the code and made it difficult to read.  Now, only one type
/// of literal is supported [LiteralType], which is an alias for [u64].  As such, any un-suffixed
/// number is assumed to be at most a 64 bit integer.  This does not limit you in any way from
/// using suffixed literals.  You can express, for example, up to 128 bit constants using standard
/// Rust notation, and using the [to_bits] trait to convert it to a [Bits] type.
/// ```
/// # use rust_hdl::core::prelude::*;
///
/// let x: Bits<128> = 0xDEADBEEF_CAFE_1234_u128.to_bits();  // Works!
/// ```
/// However, the following will fail, since the [From] trait is only implemented on [LiteralType]
/// to make the conversion unambiguous.
/// ```compile_fail
/// # use rust_hdl::core::prelude::*;
///
/// let x: Bits<128> = 0xDEADBEEF_CAFE_1234_u128.into();  // Fails to compile, since conversion from [u128] is not defined
/// ```
pub type LiteralType = u64;
/// [LITERAL_BITS] is set to the number of bits in the [LiteralType].  I.e., it is guaranteed that
/// the number of bits in [LiteralType] is [LITERAL_BITS].
pub const LITERAL_BITS: usize = 64;

/// Compute the minimum number of bits to represent a container with t items.
/// This is basically `ceil(log2(t))` as a constant (compile time computable) function.
/// You can use it where a const generic (bit width) argument is required.
///
/// Example
///
/// Unfortunately, with stable Rust, this function is not of much use.
/// For now, const generic arguments cannot be used in expressions yet.
/// Suppose we want to design a simple state machine that counts from
/// from 0 to some maximum number N-1, and then cycles again.  We
/// want to specify the maximum number, not the number of bits needed
/// to represent it.  In this case, we would like to use the
/// compile time `clog2` function to compute the bit width of
/// the signal that holds the count.
///
/// ```rust, compile_fail
/// # use rust_hdl::core::prelude::*;
///
/// #[derive(LogicBlock, Default)]
/// struct CountToN<const N: usize> {
///     signal_out: Signal<Out, Bits<{clog2({N})}>>,
/// }
/// ```
///
///
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

/// The [Bits] type holds a bit array of size [N].
#[derive(Clone, Debug, Copy)]
pub enum Bits<const N: usize> {
    #[doc(hidden)]
    Short(ShortBitVec<N>),
    #[doc(hidden)]
    Long(BitVec<N>),
}

/// Convert from a [BigUint] to a [Bits].  Will panic if the number of bits
/// needed to represent the value are greater than the width of the [Bits].
/// ```
/// # use num_bigint::BigUint;
/// # use rust_hdl::core::bits::Bits;
/// let x = BigUint::parse_bytes(b"10111000101", 2).unwrap();
/// let y : Bits<16> = x.into();
/// println!("y = {:x}", y); // Prints y = 02c5
/// ```
/// The following will panic, because the value cannot be represented in the
/// given number of bits.
/// ```
/// # use rust_hdl::core::prelude::*;
/// # use num_bigint::BigUint;
/// let x = BigUint::parse_bytes(b"10111000101", 2).unwrap();
/// let y : Bits<12> = x.into(); // Panics
/// ```
impl<const N: usize> From<BigUint> for Bits<N> {
    fn from(x: BigUint) -> Self {
        assert!(
            x.bits() <= N as u64,
            "cannot fit value from BigUInt with {} bits into Bits<{}>",
            x.bits(),
            N
        );
        if N <= SHORT_BITS {
            x.to_u64().unwrap().into()
        } else {
            let mut ret = [false; N];
            (0..N).for_each(|i| ret[i] = x.bit(i as u64));
            Bits::Long(ret.into())
        }
    }
}

/// Convert from a [Bits] to a [BigUint].
/// ```
/// # use rust_hdl::core::prelude::*;
/// # use num_bigint::BigUint;
/// let x : Bits<128> = 0xDEAD_BEEF_CAFE_BABE_1234_5678_u128.to_bits();
/// let y : BigUint = x.into();
/// println!("y = {:x}", y); // Prints 0xDEAD_BEEF_CAFE_BABE_1234_5678
/// ```
impl<const N: usize> From<Bits<N>> for BigUint {
    fn from(y: Bits<N>) -> Self {
        let mut x = BigUint::default();
        for i in 0..N {
            x.set_bit(i as u64, y.get_bit(i));
        }
        x
    }
}

#[cfg(test)]
fn random_bits<const N: usize>() -> Bits<N> {
    use rand::random;
    let mut x = Bits::default();
    for bit in 0..N {
        if random::<bool>() {
            x = x.replace_bit(bit, true);
        }
    }
    x
}

#[test]
fn test_biguint_roundtrip() {
    use rand::random;
    use seq_macro::seq;

    seq!(N in 5..150 {
        for _iters in 0..10 {
            let y: Bits<N> = random_bits();
            let z: BigUint = y.into();
            let h: Bits<N> = z.into();
            assert_eq!(h, y);
        }
    });
    seq!(N in 5..150 {
        for _iters in 0..10 {
            let bits = (0..N).map(|_| if random::<bool>() {
                b"1"[0]
            } else {
                b"0"[0]
            }).collect::<Vec<u8>>();
            let y = BigUint::parse_bytes(&bits, 2).unwrap();
            let z : Bits<N> = y.clone().into();
            let h : BigUint = z.into();
            assert_eq!(h, y);
        }
    });
}

#[test]
fn test_cast_from_biguint() {
    let x = BigUint::parse_bytes(b"1011000101", 2).unwrap();
    let y: Bits<16> = x.into();
    let p = format!("y = {:x}", y);
    assert_eq!(p, "y = 02c5");
    println!("y = {:x}", y);
}

/// Allows you to format a [Bits] as a binary string
/// ```
/// # use rust_hdl::core::bits::Bits;
/// let y = Bits::<16>::from(0b1011_0100_0010_0000);
/// println!("y = {:b}", y); // Prints y = 1011010000100000
/// ```
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

#[test]
fn test_print_as_binary() {
    let x = Bits::<16>::from(0b_1011_0100_1000_0000);
    let p = format!("x = {:b}", x);
    assert_eq!(p, "x = 1011010010000000")
}

/// Allows you to format a [Bits] as a lowercase hex string
/// ```
/// # use rust_hdl::core::bits::Bits;
/// let y = Bits::<16>::from(0xcafe);
/// println!("y = {:x}", y); // Prints y = cafe
/// ```
impl<const N: usize> LowerHex for Bits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let m: usize = N + (4 - (N % 4)) % 4; // Round up to an integer number of nibbles
        let digits: usize = m / 4;
        for digit in 0..digits {
            let nibble: Bits<4> = self.get_bits(4 * (digits - 1 - digit));
            let nibble_u8: LiteralType = nibble.into();
            std::fmt::LowerHex::fmt(&nibble_u8, f)?;
        }
        Ok(())
    }
}

#[test]
fn test_print_as_lowercase_hex() {
    let x = Bits::<16>::from(0xcafe);
    let p = format!("x = {:x}", x);
    assert_eq!(p, "x = cafe");
}

/// Allows you to format a [Bits] as an uppercase hex string
/// ```
/// # use rust_hdl::core::bits::Bits;
/// let y = Bits::<16>::from(0xcafe);
/// println!("y = {:X}", y); // Prints y = CAFE
/// ```
impl<const N: usize> UpperHex for Bits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let m: usize = N + (4 - (N % 4)) % 4; // Round up to an integer number of nibbles
        let digits: usize = m / 4;
        for digit in 0..digits {
            let nibble: Bits<4> = self.get_bits(4 * (digits - 1 - digit));
            let nibble_u8: LiteralType = nibble.into();
            std::fmt::UpperHex::fmt(&nibble_u8, f)?;
        }
        Ok(())
    }
}

#[test]
fn test_print_as_uppercase_hex() {
    let x = Bits::<16>::from(0xcafe);
    let p = format!("x = {:X}", x);
    assert_eq!(p, "x = CAFE");
}

/// Convenience function to construct [Bits] from an unsigned literal
/// Sometimes, you know you will be working with a value that is smaller than
/// 128 bits (the current maximum sized built-in unsigned integer in Rust).
/// In those cases, the [bits] function can make construction slightly
/// simpler.
/// ```
/// # use rust_hdl::core::prelude::*;
/// let x : Bits<14> = bits(0xDEA);
/// assert_eq!("0dea", format!("{:x}", x))
/// ```
/// In most cases, it's easier to use `into`:
/// ```
/// # use rust_hdl::core::prelude::*;
/// let x: Bits<14> = 0xDEA.into();
/// assert_eq!("0dea", format!("{:x}", x))
/// ```
pub fn bits<const N: usize>(x: LiteralType) -> Bits<N> {
    let t: Bits<N> = x.into();
    t
}

/// The [ToBits] trait is used to provide a way to convert Rust standard unsigned
/// types (currently `u8, u16, u32, u64, u128`) into [Bits] of different lengths.
/// Note that RustHDL will panic if you attempt to convert an unsigned type into
/// a [Bits] that is too small to hold the value.
pub trait ToBits {
    /// Convert the underlying type to a [Bits] of the specified size.  Invoked
    /// using
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<4> = 0xF_u8.to_bits();
    ///```
    fn to_bits<const N: usize>(self) -> Bits<N>;
}

impl ToBits for u8 {
    fn to_bits<const N: usize>(self) -> Bits<N> {
        (self as LiteralType).into()
    }
}

impl ToBits for u16 {
    fn to_bits<const N: usize>(self) -> Bits<N> {
        (self as LiteralType).into()
    }
}

impl ToBits for u32 {
    fn to_bits<const N: usize>(self) -> Bits<N> {
        (self as LiteralType).into()
    }
}

impl ToBits for u64 {
    fn to_bits<const N: usize>(self) -> Bits<N> {
        (self as LiteralType).into()
    }
}

impl ToBits for usize {
    fn to_bits<const N: usize>(self) -> Bits<N> {
        (self as LiteralType).into()
    }
}

impl ToBits for u128 {
    fn to_bits<const N: usize>(self) -> Bits<N> {
        Bits::<N>::from(BigUint::from(self))
    }
}

/// Cast from one bit width to another with truncation or zero padding
/// The [bit_cast] function allows you to convert from one bit width
/// to another.  It handles the different widths in the following simplified
/// manner:
///    - if casting to a narrower bit width, the most significant bits are
///      discarded until the new value fits into the specified bits
///    - if casting to a wider bit width, the most significant bits are
///      padded with zeros until the new value occupies the specified bits
/// This may seem a bit counterintuitive, but it fits logical circuitry
/// behavior.  Narrowing is usually done by preserving the least significant
/// bits (so that the carry bits are discarded when adding, for example).
/// Widening is also usually done (for unsigned values) by zero extending
/// the most significant bits.  The [bit_cast] operation does both of
/// these operations depending on the arguments.
///
/// First, an example of widening, in this case, an extra nibble is
/// added to the most significant bits, and is set to zero.
///```
/// # use rust_hdl::core::prelude::*;
/// let x : Bits<12> = bits(0xEAF);
/// let y : Bits<16> = bit_cast(x); // M = 16, N = 12
/// assert_eq!(y, bits::<16>(0x0EAF));
///```
///
/// In the second example, we downcast, this time, discarding the most
/// significant nibble.
/// ```
/// # use rust_hdl::core::prelude::*;
/// let x : Bits<16> = bits(0xDEAF);
/// let y : Bits<12> = bit_cast(x); // M = 12, N = 16
/// assert_eq!(y, bits::<12>(0xEAF));
/// ```
///
/// Note that internally, you can [bit_cast] from an arbitrary bit length
/// to another arbitrary bit length without losing information because of
/// any internal Rust limit.
///
/// Note also that bit-casting does _not_ preserve signedness.  Generally,
/// RustHDL follows hardware conventions that values are unsigned.  If you
/// want to work with signed bit vectors, use [Signed] instead.
pub fn bit_cast<const M: usize, const N: usize>(x: Bits<N>) -> Bits<M> {
    match x {
        Bits::Short(t) => {
            let t: ShortType = t.into();
            let t = if M < N {
                t & ShortBitVec::<M>::mask().short()
            } else {
                t
            };
            let k: Bits<M> = (t as LiteralType).into();
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

#[doc(hidden)]
impl<const N: usize> From<Bits<N>> for VCDValue {
    fn from(val: Bits<N>) -> Self {
        if N == 1 {
            if val.get_bit(0) {
                VCDValue::Single(vcd::Value::V1)
            } else {
                VCDValue::Single(vcd::Value::V0)
            }
        } else {
            let mut x = vec![];
            for i in 0..N {
                if val.get_bit(N - 1 - i) {
                    x.push(vcd::Value::V1)
                } else {
                    x.push(vcd::Value::V0)
                }
            }
            VCDValue::Vector(x)
        }
    }
}

#[test]
fn test_bits_from_int_via_bits() {
    let x: Bits<23> = bits(23);
    let u: LiteralType = x.into();
    assert_eq!(u, 23);
}

impl<const N: usize> Bits<N> {
    #[inline(always)]
    /// The [any] function returns true if any of the
    /// individual bits are true, and false otherwise.
    /// This reduction operation is equivalent to a logical
    /// OR of all the bits in the vector.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<14> = bits(0xDEA);
    /// assert_eq!(x.any(), true);
    /// let y : Bits<14> = Bits::default();
    /// assert_eq!(y.any(), false);
    /// ```
    pub fn any(&self) -> bool {
        match self {
            Bits::Short(x) => x.any(),
            Bits::Long(x) => x.any(),
        }
    }

    #[inline(always)]
    /// The [all] function returns true if all of the individual
    /// bits are true, and false otherwise.  This reduction
    /// operation is equivalent to a logical AND of all the bits
    /// in the vector.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<14> = bits(0xDEA);
    /// assert_eq!(x.all(), false);
    /// let y : Bits<14> = bits(0x3FFF);
    /// assert_eq!(y.all(), true);
    /// ```
    pub fn all(&self) -> bool {
        match self {
            Bits::Short(x) => x.all(),
            Bits::Long(x) => x.all(),
        }
    }

    #[inline(always)]
    /// The [xor] function computes the exclusive OR of all
    /// the bits in the vector.  This is equivalent to counting
    /// the number of ones.  If the number is odd, the XOR will
    /// be true.  If even, it will be false.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<12> = bits(0b1100_0100_1100);
    /// assert_eq!(x.xor(), true);
    /// let y: Bits<12> = bits(0b1100_0110_1100);
    /// assert_eq!(y.xor(), false);
    /// ```
    pub fn xor(&self) -> bool {
        match self {
            Bits::Short(x) => x.xor(),
            Bits::Long(x) => x.xor(),
        }
    }

    /// The [index] function is used when a [Bits] is going
    /// to be used to index into an array or some other bit vector.
    /// This is typically a very specialized hardware operation,
    /// so there are limited cases in which it can be used.  Also,
    /// there is an assumption that the [Bits] being used as
    /// an index is sufficiently small to fit in a natural word (assume 32 bits, here
    /// for safety).  In practice, that means, that if you are
    /// indexing into a register using some other register/value,
    /// the _length_ of the register is limited to a few billion bits.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<12> = bits(0b1100_0100_1100);
    /// assert_eq!(x.index(), 0b1100_0100_1100_usize);
    /// ```
    pub fn index(&self) -> usize {
        match self {
            Bits::Short(x) => x.short() as usize,
            Bits::Long(_x) => panic!("Cannot map long bit vector to index type"),
        }
    }

    #[inline(always)]
    /// Return the number of bits in the current [Bits].
    /// Because this is determined at compile time, it is
    /// of limited use as a runtime function, but is there
    /// nonetheless.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<14> = Bits::default();
    /// assert_eq!(x.len(), 14);
    /// ```
    pub fn len(&self) -> usize {
        N
    }

    #[inline(always)]
    /// Return true if this [Bits] contains no actual bits (i.e., N = 0).
    pub fn is_empty(&self) -> bool {
        N == 0
    }

    /// Compute the number of possible values that a [Bits]
    /// can take.  This is basically 2 raised to the Nth
    /// power.  Because the result is returned as a [usize],
    /// you must be careful, since this can easily overflow.
    /// A [Bits<256>] for example, cannot represent [count]
    /// on a normal 64 bit machine.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// assert_eq!(Bits::<16>::count(), 1 << 16);
    /// ```
    pub fn count() -> u128 {
        1 << N
    }

    #[inline(always)]
    /// Extract the [index] bit from the given [Bits]. This will
    /// cause a runtime panic if the [index] bit is out of range
    /// of the width of the bitvector.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<14> = bits(0b10110);
    /// assert_eq!(x.get_bit(0), false);
    /// assert_eq!(x.get_bit(1), true);
    /// assert_eq!(x.get_bit(2), true); // You get the idea
    /// ```
    pub fn get_bit(&self, index: usize) -> bool {
        assert!(index < N);
        match self {
            Bits::Short(x) => x.get_bit(index),
            Bits::Long(x) => x.get_bit(index),
        }
    }

    /// Replace the given bit of a [Bits] with a new bit value.
    /// This method leaves the original value alone, and returns
    /// a new [Bits] with all bits except the designated one left
    /// alone.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<16> = bits(0b1100_0000);
    /// let x = x.replace_bit(0, true);
    /// let x = x.replace_bit(7, false);
    /// assert_eq!(x, bits(0b0100_0001));
    /// ```
    pub fn replace_bit(&self, index: usize, val: bool) -> Self {
        assert!(index < N);
        match self {
            Bits::Short(x) => Bits::Short(x.replace_bit(index, val)),
            Bits::Long(x) => Bits::Long(x.replace_bit(index, val)),
        }
    }

    #[inline(always)]
    /// Return a subset of bits from a [Bits] value, with a given offset.
    /// To preserve the feasibility of representing this in hardware, the width
    /// of the result must be fixed (the argument [M]), and only the offset
    /// can be computed.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<40> = bits(0xDEAD_BEEF_CA);
    /// let y = x.get_bits::<32>(8);
    /// assert_eq!(y, bits(0xDEAD_BEEF))
    /// ```
    pub fn get_bits<const M: usize>(&self, index: usize) -> Bits<M> {
        assert!(index <= N);
        bit_cast::<M, N>(*self >> index as LiteralType)
    }

    #[inline(always)]
    /// Set a group of bits in a value.  This operation modifies the
    /// value in place.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let mut x: Bits<40> = bits(0xDEAD_BEEF_CA);
    /// x.set_bits::<16>(8, bits(0xCAFE));
    /// assert_eq!(x, bits(0xDEAD_CAFE_CA));
    /// ```
    pub fn set_bits<const M: usize>(&mut self, index: usize, rhs: Bits<M>) {
        assert!(index <= N);
        assert!(index + M <= N);
        let mask = !(bit_cast::<N, M>(Bits::<M>::mask()) << index as LiteralType);
        let masked = *self & mask;
        let replace = bit_cast::<N, M>(rhs) << index as LiteralType;
        *self = masked | replace
    }

    #[inline(always)]
    /// Returns a [Bits] value that contains [N] ones.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x = Bits::<40>::mask();
    /// assert_eq!(x, bits(0xFF_FFFF_FFFF));
    /// ```
    pub fn mask() -> Bits<N> {
        if N <= SHORT_BITS {
            Bits::Short(ShortBitVec::<N>::mask())
        } else {
            Bits::Long([true; N].into())
        }
    }

    /// Returns the width in bits of the [BitVec].
    /// Note that this is the number of bits allocated.
    /// It does not depend on the value at all.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// assert_eq!(Bits::<40>::width(), 40);
    /// ```
    pub const fn width() -> usize {
        N
    }

    /// Convert [Bits] to an [u8].
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<6> = 12.into();
    /// let y = x.to_u8();
    /// assert_eq!(y, 12_u8);
    /// ```
    /// Note that this will panic if the width of the
    /// bitvector is larger than 8 bits.
    /// ```should_panic
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<12> = 0xADE.into();
    /// let y = x.to_u8(); // Panics - too many bits
    /// ```
    pub fn to_u8(self) -> u8 {
        assert!(N <= 8, "Cannot convert Bits::<{}> to u8 - too many bits", N);
        let x: LiteralType = self.into();
        x as u8
    }

    /// Convert [Bits] to an [u16].
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<12> = 12.into();
    /// let y = x.to_u16();
    /// assert_eq!(y, 12_u16);
    /// ```
    /// Note that this will panic if the width of the
    /// bitvector is larger than 16 bits.
    /// ```should_panic
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<20> = 0xADE.into();
    /// let y = x.to_u16(); // Panics - too many bits
    /// ```
    pub fn to_u16(self) -> u16 {
        assert!(
            N <= 16,
            "Cannot convert Bits::<{}> to u16 - too many bits",
            N
        );
        let x: LiteralType = self.into();
        x as u16
    }

    /// Convert [Bits] to an [u32].
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<24> = 12.into();
    /// let y = x.to_u32();
    /// assert_eq!(y, 12_u32);
    /// ```
    /// Note that this will panic if the width of the
    /// bitvector is larger than 32 bits.
    /// ```should_panic
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<40> = 0xADE.into();
    /// let y = x.to_u32(); // Panics - too many bits
    /// ```
    pub fn to_u32(self) -> u32 {
        assert!(
            N <= 32,
            "Cannot convert Bits::<{}> to u32 - too many bits",
            N
        );
        let x: LiteralType = self.into();
        x as u32
    }

    /// Convert [Bits] to an [u64].
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<40> = 12.into();
    /// let y = x.to_u64();
    /// assert_eq!(y, 12_u64);
    /// ```
    /// Note that this will panic if the width of the
    /// bitvector is larger than 64 bits.
    /// ```should_panic
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<80> = 0xADE.into();
    /// let y = x.to_u64(); // Panics - too many bits
    /// ```
    pub fn to_u64(self) -> u64 {
        assert!(
            N <= 64,
            "Cannot convert Bits::<{}> to u64 - too many bits",
            N
        );
        let x: LiteralType = self.into();
        x
    }

    /// Convert [Bits] to an [u128].
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<80> = 12.into();
    /// let y = x.to_u128();
    /// assert_eq!(y, 12_u128);
    /// ```
    /// Note that this will panic if the width of the
    /// bitvector is larger than 128 bits.
    /// ```should_panic
    /// # use rust_hdl::core::prelude::*;
    /// let x: Bits<140> = 0xADE.into();
    /// let y = x.to_u128(); // Panics - too many bits
    /// ```
    pub fn to_u128(self) -> u128 {
        match self {
            Bits::Short(x) => x.to_u128(),
            Bits::Long(x) => x.to_u128(),
        }
    }
}

impl From<bool> for Bits<1> {
    #[inline(always)]
    /// Convenience method that allows you to convert
    /// a boolean into a single bit-width [Bits].
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<1> = true.into();
    /// assert_eq!(x, bits(1))
    /// ```
    fn from(x: bool) -> Self {
        if x {
            1.into()
        } else {
            0.into()
        }
    }
}

impl From<Bits<1>> for bool {
    #[inline(always)]
    /// Convenience method for converting a 1-bit
    /// width [Bits] value into a boolean value.
    /// ```
    /// # use rust_hdl::core::prelude::*;
    /// let x : Bits<1> = bits(1);
    /// let y : bool = x.into();
    /// assert!(y)
    /// ```
    fn from(val: Bits<1>) -> Self {
        val.get_bit(0)
    }
}

/// Convert from [LiteralType] to [Bits].  Because of some restrictions on
/// how Rust's type inference works, when you work with unsized
/// literals (e.g., `x = 3`), there must either be a unique integer type
/// that can fit the expression, or it will default to `i32`.  Unfortunately,
/// in general, [Bits] are used to represent unsigned types.  The upshot
/// of all this is that RustHDL selects one unsigned integer type to represent
/// literals.  Although not ideal, RustHDL uses a [LiteralType] (currently 'u64') to represent literals
/// so as to make HDL expressions close to Verilog or VHDL.  This choice should not affect
/// any hardware implementation, as hardware registers need to be of [Bits] type.
/// ```
/// # use rust_hdl::core::prelude::*;
/// let x: Bits<16> = 0xDEAD.into(); // This is interpreteed as a 128 bit constant by Rust
/// ```
/// This example is the largest bit width literal you can express using current
/// edition Rust:
/// ```
/// # use rust_hdl::core::prelude::*;
/// let x: Bits<128> = 0xDEADBEEF_CAFEBABE_1234ABCD_00005EA1_u128.to_bits();
/// ```
/// From a safety perspective, RustHDL will panic if the argument is too large to fit
/// into the bit vector.  Thus, this example will panic, since the literal cannot be
/// fit into 16 bits without truncation:
/// ```should_panic
/// # use rust_hdl::core::prelude::*;
/// let x: Bits<16> = 0xDEADBEEF.into(); // This will panic!
/// ```
impl<const N: usize> From<LiteralType> for Bits<N> {
    fn from(x: LiteralType) -> Self {
        if N > SHORT_BITS {
            let y: BitVec<N> = x.into();
            Bits::Long(y)
        } else {
            assert!(
                x <= ShortBitVec::<N>::max_legal(),
                "Value 0x{:x} does not fit into bitvector of length {}",
                x,
                N
            );
            Bits::Short((x as ShortType).into())
        }
    }
}

impl<const N: usize> From<Wrapping<LiteralType>> for Bits<N> {
    fn from(x: Wrapping<LiteralType>) -> Self {
        x.0.into()
    }
}

/// Convert a [Bits] back to [u128].  Until Rust supports larger integers, the [u128] is
/// the largest integer type you can use without resorting to [BigUint].  RustHDL will panic
/// if you try to convert a [Bits] that is more than 128 bits to a literal.  Even if the
/// value in the bitvector would fit.
///```
///# use rust_hdl::core::prelude::*;
/// let x: Bits<16> = 0xDEAD.into();
/// let y: u128 = x.to_u128();
/// assert_eq!(y, 0xDEAD);
///```
///The following will panic even through the literal value stored in the 256 bit vector
///is less than 128 bits.
///```should_panic
///# use rust_hdl::core::prelude::*;
///let x : Bits<256> = 42.into();
///let y: u128 = x.to_u128(); // Panics!
/// ```
impl<const N: usize> From<Bits<N>> for LiteralType {
    fn from(x: Bits<N>) -> Self {
        assert!(N <= LITERAL_BITS);
        match x {
            Bits::Short(t) => {
                let p: ShortType = t.into();
                p as LiteralType
            }
            Bits::Long(t) => t.into(),
        }
    }
}

#[inline(always)]
#[doc(hidden)]
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
        #[doc(hidden)]
        impl<const N: usize> std::ops::$method<Bits<N>> for Bits<N> {
            type Output = Bits<N>;

            #[inline(always)]
            fn $func(self, rhs: Bits<N>) -> Self::Output {
                binop(self, rhs, |a, b| a $op b, |a, b| a $op b)
            }
        }

        impl<const N: usize> std::ops::$method<LiteralType> for Bits<N> {
            type Output = Bits<N>;

            fn $func(self, rhs: LiteralType) -> Self::Output {
                binop(self, rhs.into(), |a, b| a $op b, |a, b| a $op b)
            }
        }

        impl<const N: usize> std::ops::$method<Bits<N>> for LiteralType {
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

/// Construct a default [Bits] - i.e., a zero bit vector of length N.
/// ```
/// # use rust_hdl::core::prelude::*;
/// let x : Bits<200> = Default::default();
/// assert_eq!(x, bits(0));
/// ```
impl<const N: usize> Default for Bits<N> {
    fn default() -> Bits<N> {
        bits::<N>(0)
    }
}

/// Bitwise inversion of a [Bits] vector
/// The `!` operator will invert each bit in a [Bits] vector.
/// ```
/// # use rust_hdl::core::prelude::*;
/// let x : Bits<16> = bits(0xAAAA);
/// let y = !x;
/// assert_eq!(y, bits(0x5555))
/// ```
impl<const N: usize> std::ops::Not for Bits<N> {
    type Output = Bits<N>;

    fn not(self) -> Self::Output {
        match self {
            Bits::Short(x) => Bits::Short(!x),
            Bits::Long(x) => Bits::Long(!x),
        }
    }
}

#[doc(hidden)]
impl<const N: usize> Ord for Bits<N> {
    fn cmp(&self, other: &Bits<N>) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[doc(hidden)]
impl<const N: usize> PartialOrd<Bits<N>> for LiteralType {
    fn partial_cmp(&self, other: &Bits<N>) -> Option<Ordering> {
        let self_as_bits: Bits<N> = (*self).into();
        self_as_bits.partial_cmp(other)
    }
}

#[doc(hidden)]
impl<const N: usize> PartialOrd<LiteralType> for Bits<N> {
    fn partial_cmp(&self, other: &LiteralType) -> Option<Ordering> {
        let other_as_bits: Bits<N> = (*other).into();
        self.partial_cmp(&other_as_bits)
    }
}

#[doc(hidden)]
impl<const N: usize> PartialOrd<Bits<N>> for Bits<N> {
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

#[doc(hidden)]
impl<const N: usize> PartialEq<Bits<N>> for Bits<N> {
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

#[doc(hidden)]
impl<const N: usize> PartialEq<LiteralType> for Bits<N> {
    fn eq(&self, other: &LiteralType) -> bool {
        let other_as_bits: Bits<N> = (*other).into();
        self.eq(&other_as_bits)
    }
}

#[doc(hidden)]
impl<const N: usize> PartialEq<Bits<N>> for LiteralType {
    fn eq(&self, other: &Bits<N>) -> bool {
        let self_as_bits: Bits<N> = (*self).into();
        self_as_bits.eq(other)
    }
}

#[doc(hidden)]
impl PartialEq<bool> for Bits<1> {
    #[inline(always)]
    fn eq(&self, other: &bool) -> bool {
        self.get_bit(0) == *other
    }
}

#[doc(hidden)]
impl PartialEq<Bits<1>> for bool {
    fn eq(&self, other: &Bits<1>) -> bool {
        *self == other.get_bit(0)
    }
}

#[doc(hidden)]
impl<const N: usize> Eq for Bits<N> {}

#[doc(hidden)]
impl<const N: usize> std::hash::Hash for Bits<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Bits::Short(t) => t.hash(state),
            Bits::Long(t) => t.hash(state),
        }
    }
}

#[doc(hidden)]
impl<const N: usize> std::ops::Add<bool> for Bits<N> {
    type Output = Bits<N>;

    fn add(self, rhs: bool) -> Self::Output {
        if rhs {
            self + Bits::<N>::from(1)
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{bit_cast, clog2, Bits};
    use crate::core::bits::random_bits;
    use crate::core::bits::{LiteralType, ToBits};
    use num_bigint::BigUint;
    use num_traits::One;
    use seq_macro::seq;
    use std::num::Wrapping;

    #[test]
    fn test_get_bits_section() {
        let x: Bits<40> = 0xD_ADBE_EFCA.into();
        let y = x.get_bits::<32>(8).to_u32();
        let answer = 0xDAD_BEEF;
        assert_eq!(y, answer);
    }

    #[test]
    fn test_short_from_u8() {
        let x: Bits<4> = 15.into();
        let y: LiteralType = x.into();
        assert_eq!(y, 15 & (0x0F));
    }

    #[test]
    fn test_short_from_u16() {
        let x: Bits<12> = 1432.into();
        let y: LiteralType = x.into();
        assert_eq!(y, 1432 & (0x0FFF));
    }

    #[test]
    fn test_short_from_u32() {
        let x: Bits<64> = 12434234.into();
        let y: LiteralType = x.into();
        assert_eq!(y, 12434234);
    }

    #[test]
    fn test_from_u32() {
        let x: Bits<64> = 0xFFFF_FFFF.into();
        let y: LiteralType = x.into();
        assert_eq!(y, 0xFFFF_FFFF);
    }

    #[test]
    fn or_test() {
        let a: Bits<32> = 45.into();
        let b: Bits<32> = 10395.into();
        let c = a | b;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, 45 | 10395)
    }
    #[test]
    fn and_test() {
        let a: Bits<32> = 45.into();
        let b: Bits<32> = 10395.into();
        let c = a & b;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, 45 & 10395)
    }
    #[test]
    fn xor_test() {
        let a: Bits<32> = 45.into();
        let b: Bits<32> = 10395.into();
        let c = a ^ b;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, 45 ^ 10395)
    }
    #[test]
    fn not_test() {
        let a: Bits<32> = 45.into();
        let c = !a;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, (!45_u32) as LiteralType);
    }
    #[test]
    fn shr_test() {
        let a: Bits<32> = 10395.into();
        let c: Bits<32> = a >> 4;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, 10395 >> 4);
    }
    #[test]
    fn shr_test_pair() {
        let a: Bits<32> = 10395.into();
        let b: Bits<32> = 4.into();
        let c = a >> b;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, 10395 >> 4);
    }
    #[test]
    fn shl_test() {
        let a: Bits<32> = 10395.into();
        let c = a << 24;
        let c_u32 = c.to_u32();
        assert_eq!(c_u32, 10395 << 24);
    }
    #[test]
    fn shl_test_pair() {
        let a: Bits<32> = 10395.into();
        let b: Bits<32> = 4.into();
        let c = a << b;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, 10395 << 4);
    }
    #[test]
    fn add_works() {
        let a: Bits<32> = 10234.into();
        let b: Bits<32> = 19423.into();
        let c = a + b;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, 10234 + 19423);
    }
    #[test]
    fn add_int_works() {
        let a: Bits<32> = 10234.into();
        let b = 19423;
        let c: Bits<32> = a + b;
        let c_u32: LiteralType = c.into();
        assert_eq!(c_u32, 10234 + 19423);
    }
    #[test]
    fn add_works_with_overflow() {
        let x = 2_042_102_334_u32;
        let y = 2_942_142_512_u32;
        let a: Bits<32> = x.to_bits();
        let b: Bits<32> = y.to_bits();
        let c = a + b;
        let c_u32 = c.to_u32();
        assert_eq!(Wrapping(c_u32), Wrapping(x) + Wrapping(y));
    }
    #[test]
    fn sub_works() {
        let x = 2_042_102_334_u32;
        let y = 2_942_142_512_u32;
        let a: Bits<32> = x.to_bits();
        let b: Bits<32> = y.to_bits();
        let c = a - b;
        let c_u32 = c.to_u32();
        assert_eq!(Wrapping(c_u32), Wrapping(x) - Wrapping(y));
    }
    #[test]
    fn sub_int_works() {
        let x = 2_042_102_334_u32;
        let y = 2_942_142_512;
        let a: Bits<32> = x.to_bits();
        let c = a - y;
        let c_u32 = c.to_u32();
        assert_eq!(Wrapping(c_u32), Wrapping(x) - Wrapping(y as u32));
    }
    #[test]
    fn eq_works() {
        let x = 2_032_142_351;
        let y = 2_942_142_512;
        let a: Bits<32> = x.into();
        let b: Bits<32> = x.into();
        let c: Bits<32> = y.into();
        assert_eq!(a, b);
        assert_ne!(a, c)
    }
    #[test]
    fn mask_works() {
        let a: Bits<48> = 0xFFFF_FFFF_FFFF.into();
        let b = Bits::<48>::mask();
        assert_eq!(a, b);
        let a: Bits<16> = 0xFFFF.into();
        let b = Bits::<16>::mask();
        assert_eq!(a, b)
    }
    #[test]
    fn get_bit_works() {
        // 0101 = 5
        let a: Bits<48> = 0xFFFF_FFFF_FFF5.into();
        assert!(a.get_bit(0));
        assert!(!a.get_bit(1));
        assert!(a.get_bit(2));
        assert!(!a.get_bit(3));
        let c: Bits<5> = 3.into();
        assert!(!a.get_bit(c.index()));
    }
    #[test]
    fn test_bit_cast_short() {
        let a: Bits<8> = 0xFF.into();
        let b: Bits<16> = bit_cast(a);
        assert_eq!(b, 0xFF);
        let c: Bits<4> = bit_cast(a);
        assert_eq!(c, 0xF);
    }
    #[test]
    fn test_bit_cast_long() {
        let a: Bits<48> = 0xdead_cafe_babe.into();
        let b: Bits<44> = bit_cast(a);
        assert_eq!(b, 0xead_cafe_babe);
        let b: Bits<32> = bit_cast(a);
        assert_eq!(b, 0xcafe_babe);
    }
    #[test]
    fn test_bit_extract_long() {
        let a: Bits<48> = 0xdead_cafe_babe.into();
        let b: Bits<44> = a.get_bits(4);
        assert_eq!(b, 0x0dea_dcaf_ebab);
        let b: Bits<32> = a.get_bits(16);
        assert_eq!(b, 0xdead_cafe);
    }
    #[test]
    fn test_set_bit() {
        let a: Bits<48> = 0xdead_cafe_babe.into();
        let mut b = a;
        for i in 4..8 {
            b = b.replace_bit(i, false)
        }
        assert_eq!(b, 0xdead_cafe_ba0e);
    }
    #[test]
    fn test_set_bits() {
        let a: Bits<16> = 0xdead.into();
        let b: Bits<4> = 0xf.into();
        let mut c = a;
        c.set_bits(4, b);
        assert_eq!(c, 0xdefd);
        let a: Bits<48> = 0xdead_cafe_babe.into();
        let b: Bits<8> = 0xde.into();
        let mut c = a;
        c.set_bits(16, b);
        assert_eq!(c, 0xdead_cade_babe);
    }

    #[test]
    fn test_constants_and_bits() {
        let a: Bits<16> = 0xdead.into();
        let b = a + 1;
        let c = 1 + a;
        println!("{:x}", b);
        assert_eq!(b, 0xdeae);
        assert_eq!(b, c);
    }
    #[test]
    fn test_clog2() {
        const A_WIDTH: usize = clog2(250);
        let a: Bits<{ A_WIDTH }> = 153.into();
        println!("{:x}", a);
        assert_eq!(a.len(), 8);
        assert_eq!(clog2(1024), 10);
    }

    #[test]
    fn test_clog2_inline() {
        const A_WIDTH: usize = clog2(1000);
        let a: Bits<A_WIDTH> = 1023.into();
        assert_eq!(a.len(), 10);
    }

    #[test]
    fn test_default() {
        const N: usize = 128;
        let a = Bits::<N>::default();
        assert_eq!(a, 0);
    }

    #[test]
    fn test_get_bits() {
        fn get_bits_test<const N: usize, const M: usize>() {
            for offset in 0_usize..N {
                let y: Bits<N> = random_bits();
                let z = y.get_bits::<M>(offset);
                let yb: BigUint = y.into();
                let yb = (yb >> offset) & ((BigUint::one() << M) - BigUint::one());
                let zb: BigUint = z.into();
                assert_eq!(zb, yb);
            }
        }
        seq!(N in 0..16 {
            get_bits_test::<8, N>();
        });
        seq!(N in 0..64 {
            get_bits_test::<32, N>();
        });
        seq!(N in 0..65 {
            get_bits_test::<64, N>();
        });
        seq!(N in 0..300 {
            get_bits_test::<256, N>();
        });
        seq!(N in 0..150 {
            get_bits_test::<125, N>();
        });
    }

    #[test]
    fn test_bitcast() {
        fn bitcast_test<const N: usize, const M: usize>() {
            let y: Bits<N> = random_bits();
            let z = bit_cast::<M, N>(y);
            let yb: BigUint = y.into();
            let zb = yb & ((BigUint::one() << M) - BigUint::one());
            let zc: BigUint = z.into();
            assert_eq!(zb, zc);
        }
        fn bitcast_test_set<const M: usize>() {
            bitcast_test::<M, 1>();
            bitcast_test::<M, 8>();
            bitcast_test::<M, 16>();
            bitcast_test::<M, 32>();
            bitcast_test::<M, 64>();
            bitcast_test::<M, 128>();
            bitcast_test::<M, 256>();
        }
        bitcast_test_set::<1>();
        bitcast_test_set::<8>();
        bitcast_test_set::<16>();
        bitcast_test_set::<32>();
        bitcast_test_set::<64>();
        bitcast_test_set::<128>();
        bitcast_test_set::<256>();
    }

    #[test]
    fn test_any() {
        seq!(N in 1..150 {
            for _rep in 0..10 {
                let y: Bits<N> = random_bits();
                let z : BigUint = y.into();
                let y_any = y.any();
                let z_any = z.count_ones() != 0;
                assert_eq!(y_any, z_any)
            }
            let y = Bits::<N>::default();
            assert!(!y.any());
        });
    }

    #[test]
    fn test_all() {
        seq!(N in 1..150 {
            for _rep in 0..10 {
                let y: Bits<N> = random_bits();
                let z : BigUint = y.into();
                let y_all = y.all();
                let z_all = z.count_ones() == N;
                assert_eq!(y_all, z_all)
            }
            let y = Bits::<N>::mask();
            assert!(y.all());
        });
    }

    #[test]
    fn test_shl() {
        seq!(N in 1..150 {
            for l in 0..N {
                let y: Bits<N> = random_bits();
                let z: Bits<N> = y << l;
                let y1 : BigUint = y.into();
                let mask : BigUint = (BigUint::one() << N) - BigUint::one();
                let z1 = (y1 << l) & mask;
                let convert : BigUint = z.into();
                assert_eq!(z1, convert)
            }
        });
    }

    #[test]
    fn test_shr() {
        seq!(N in 1..150 {
            for l in 0..N {
                let y: Bits<N> = random_bits();
                let z: Bits<N> = y >> l;
                let y1 : BigUint = y.into();
                let mask : BigUint = (BigUint::one() << N) - BigUint::one();
                let z1 = (y1 >> l) & mask;
                let convert : BigUint = z.into();
                assert_eq!(z1, convert)
            }
        });
    }

    macro_rules! test_op_with_values {
        ($func: ident) => {
            seq!(N in 1..150 {
                for _iters in 0..10 {
                    let y: Bits<N> = random_bits();
                    let z: Bits<N> = random_bits();
                    let v1_as_bint : BigUint = y.into();
                    let v2_as_bint : BigUint = z.into();
                    let mask : BigUint = (BigUint::one() << N) - BigUint::one();
                    let (lib_answer, biguint_answer) = $func(y, z, v1_as_bint, v2_as_bint, mask);
                    let convert : BigUint = lib_answer.into();
                    assert_eq!(biguint_answer, convert)
                }
            });
        }
    }

    #[test]
    fn test_add() {
        fn add<const N: usize>(
            y: Bits<N>,
            z: Bits<N>,
            y1: BigUint,
            z1: BigUint,
            mask: BigUint,
        ) -> (Bits<N>, BigUint) {
            (y + z, (y1 + z1) & mask)
        }
        test_op_with_values!(add);
    }

    #[test]
    fn test_sub() {
        fn sub<const N: usize>(
            y: Bits<N>,
            z: Bits<N>,
            y1: BigUint,
            z1: BigUint,
            mask: BigUint,
        ) -> (Bits<N>, BigUint) {
            if z1 <= y1 {
                (y - z, (y1 - z1))
            } else {
                (y - z, mask + BigUint::one() + y1 - z1)
            }
        }
        test_op_with_values!(sub);
    }

    #[test]
    fn test_bitor() {
        fn bor<const N: usize>(
            y: Bits<N>,
            z: Bits<N>,
            y1: BigUint,
            z1: BigUint,
            mask: BigUint,
        ) -> (Bits<N>, BigUint) {
            (y | z, (y1 | z1) & mask)
        }
        test_op_with_values!(bor);
    }

    #[test]
    fn test_bitand() {
        fn band<const N: usize>(
            y: Bits<N>,
            z: Bits<N>,
            y1: BigUint,
            z1: BigUint,
            mask: BigUint,
        ) -> (Bits<N>, BigUint) {
            (y & z, (y1 & z1) & mask)
        }
        test_op_with_values!(band);
    }

    #[test]
    fn test_bitxor() {
        fn bxor<const N: usize>(
            y: Bits<N>,
            z: Bits<N>,
            y1: BigUint,
            z1: BigUint,
            mask: BigUint,
        ) -> (Bits<N>, BigUint) {
            (y ^ z, (y1 ^ z1) & mask)
        }
        test_op_with_values!(bxor);
    }

    #[test]
    fn test_not() {
        fn not<const N: usize>(
            y: Bits<N>,
            _z: Bits<N>,
            y1: BigUint,
            _z1: BigUint,
            mask: BigUint,
        ) -> (Bits<N>, BigUint) {
            (!y, (y1 ^ mask))
        }
        test_op_with_values!(not);
    }

    macro_rules! test_cmp_with_values {
        ($func: ident) => {
            seq!(N in 1..256 {
                for _iters in 0..10 {
                    let y: Bits<N> = random_bits();
                    let z: Bits<N> = random_bits();
                    let v1_as_bint : BigUint = y.into();
                    let v2_as_bint : BigUint = z.into();
                    let (lib_answer, biguint_answer) = $func(y, z, v1_as_bint, v2_as_bint);
                    assert_eq!(lib_answer, biguint_answer)
                }
            });
        }
    }

    #[test]
    fn test_lt() {
        fn lt<const N: usize>(y: Bits<N>, z: Bits<N>, y1: BigUint, z1: BigUint) -> (bool, bool) {
            (y < z, y1 < z1)
        }
        test_cmp_with_values!(lt);
    }

    #[test]
    fn test_le() {
        fn le<const N: usize>(y: Bits<N>, z: Bits<N>, y1: BigUint, z1: BigUint) -> (bool, bool) {
            (y <= z, y1 <= z1)
        }
        test_cmp_with_values!(le);
    }

    #[test]
    fn test_eq() {
        fn eq<const N: usize>(y: Bits<N>, z: Bits<N>, y1: BigUint, z1: BigUint) -> (bool, bool) {
            (y == z, y1 == z1)
        }
        test_cmp_with_values!(eq);
    }

    #[test]
    fn test_neq() {
        fn neq<const N: usize>(y: Bits<N>, z: Bits<N>, y1: BigUint, z1: BigUint) -> (bool, bool) {
            (y != z, y1 != z1)
        }
        test_cmp_with_values!(neq);
    }

    #[test]
    fn test_ge() {
        fn ge<const N: usize>(y: Bits<N>, z: Bits<N>, y1: BigUint, z1: BigUint) -> (bool, bool) {
            (y >= z, y1 >= z1)
        }
        test_cmp_with_values!(ge);
    }

    #[test]
    fn test_gt() {
        fn gt<const N: usize>(y: Bits<N>, z: Bits<N>, y1: BigUint, z1: BigUint) -> (bool, bool) {
            (y > z, y1 > z1)
        }
        test_cmp_with_values!(gt);
    }
}

/// A type alias for a simple bool.  You can use them interchangeably.
pub type Bit = bool;

/// Multipliers are special, so we only implement multipliers that we think are
/// synthesizable.  In this case, we implement a 16 x 16 bit multiplier
/// which yields a 32 bit result.
impl std::ops::Mul<Bits<16>> for Bits<16> {
    type Output = Bits<32>;

    fn mul(self, rhs: Bits<16>) -> Self::Output {
        let x = match self {
            Bits::Short(x) => x.short(),
            Bits::Long(_) => {
                panic!("unreachable!")
            }
        };
        let y = match rhs {
            Bits::Short(x) => x.short(),
            Bits::Long(_) => {
                panic!("unreachable!")
            }
        };
        Bits::Short(ShortBitVec::from(x * y))
    }
}
