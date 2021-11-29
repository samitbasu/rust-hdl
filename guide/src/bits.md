# BitVectors

The base of the RustHDL classes is the notion of a bitvector.
A bitvector is a way to have a statically sized vector of boolean digits
that can be used much like an atomic, built in integer class in RustHDL.
The bitvector is important to understand because it forms the basic
type that underlies RustHDL.  Generally, bitvectors are designed
to be intuitive.  

## Constructing Bits from a Constant

If you hae a value that you know is small enough to fit into Rust's
base type (a `u128` at the moment), then you can use the `bits`
function to construct it:

```rust
#fn main() {
    let t = bits::<13>(5_u128); // t is 13 bits wide, and represents 5
#}
```

You can also cast from one bits to another.  Bit casting is very common
in HDL designs.  RustHDL makes the cast explicit by requiring you
to satisfy the type system.  Here is the declaration of the
`bit_cast` function:

```rust
pub fn bit_cast<const M: usize, const N: usize>(x: Bits<N>) -> Bits<M> {
    // Snip...
}
```

If you aren't familiar with const generics, the syntax may look a little
odd, but it's basically parameterized by the input and output widths.  
In some cases, you won't need to provide these, since Rust will infer
them from context.  For example:

```rust
#fn main() {
   let x : Bits<4> = bit_cast(bits::<6>(5_u128)); // x is 5 represented with 4 bits  
#}
```

You can also use the turbofish notation `::<>` and indicate how many
bits you want after the cast:

```rust
#fn main() {
   let x = bit_cast::<4, 6>(bits::<6>(5_u128)); // Same result - x is 5 represented with 4 bits.   
#}
```

