The [Bits](core::bits::Bits) type supports a subset of operations that can be synthesized in
hardware.  You can perform
 
* Addition between `Bits` of the same size using the `+` operator
* Subtraction between `Bits` of the same size using the `-` operator
* Bitwise logical `AND` between `Bits` of the same size using the `&` operator
* Bitwise logical `OR` between `Bits` of the same size using the `|` operator
* Bitwise logical `XOR` (Exclusive Or) between `Bits` of the same size using the `^` operator
* Bitwise comparisons for equality between `Bits` of the same size using `==` and `!=` operators
* Unsigned comparisons (e.g., `>,>=,<,<=`) between `Bits` of the same size - these are
always treated as unsigned values for comparison purposes.
* Shift left using the `<<` operator
* Shift right (no sign extension!) using the '>>' operator
* Bitwise logical `NOT` using the `!` prefix operator
 
These should feel natural when using RustHDL, as expressions follow Rust's rules (and not Verilog's).
For example:
```rust
# use rust_hdl::core::prelude::*;
let x: Bits<32> = 0xDEAD_0000_u32.to_bits();
let y: Bits<32> = 0x0000_BEEF_u32.to_bits();
let z = x + y;
assert_eq!(z, 0xDEAD_BEEF_u32.to_bits());
```

You can, of course, construct expressions of arbitrary complexity using parenthesis, etc.
The only real surprise may be at synthesis time, when you try to fit the expression onto hardware.
 
