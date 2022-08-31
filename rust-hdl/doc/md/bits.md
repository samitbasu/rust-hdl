 
  The [Bits](core::bits::Bits) type is a `Copy` enabled type that you can construct from integers,
  from the `Default` trait, or from other `Bits`.    Mostly, it is meant to stay out of your way
  and behave like a `u32`.
 
  ```
  # use rust_hdl::core::prelude::*;
  let x: Bits<50> = Default::default();
  ```
  This will construct a length 50 bit vector that is initialized to all `0`.
 
  You can also convert from literals into bit vectors using the [From] and [Into] traits,
  provided the literals are of the `u64` type.
 
  ```
  # use rust_hdl::core::prelude::*;
  let x: Bits<50> = 0xBEEF.into();
  ```
 
  In some cases, Rust complains about literals, and you may need to provide a suffix:
  ```
  # use rust_hdl::core::prelude::*;
  let x: Bits<50> = 0xDEAD_BEEF_u64.into();
  ```
  However, in most cases, you can leave literals suffix-free, and Rust will automatically
  determine the type from the context.
 
  You can construct a larger constant using the [bits] function.  If you have a literal of up to
  128 bits, it provides a functional form
  ```
  # use rust_hdl::core::prelude::*;
  let x: Bits<200> = bits(0xDEAD_BEEE);   // Works for up to 128 bit constants.
  ```
 
  There is also the [ToBits] trait, which is implemented on the basic unsigned integer types.
  This trait allows you to handily convert from different integer values
 
  ```
  # use rust_hdl::core::prelude::*;
  let x: Bits<10> = 32_u8.to_bits();
  ```
