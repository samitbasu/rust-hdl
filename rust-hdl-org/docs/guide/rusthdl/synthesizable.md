# Synthesizable Subset of Rust

RustHDL uses procedural macros to define a subset of the Rust language that can be used to
describe actual hardware.  That subset is known as the synthesizable subset of Rust.  It is
quite limited because the end result is translated into Verilog and ultimately into hardware
configuration for the FPGA.

## Valid Rust

:::danger
The HDL kernel must be valid Rust!  If you remove the `#[hdl_gen]` attribute, the code
must still be accepted by `rustc`!  That means you must satisfy the type constraints, the
private nature of the struct fields, etc.  This is one of the major benefits of RustHDL.  It
takes code that is already been checked by `rustc` and then converts it into HDL.
:::

So this will _clearly_ fail to compile.

```rust
struct Foo {
 bar: Signal<Out, Bits<4>>
}

impl Logic for Foo {
   #[hdl_gen]
   fn update(&mut self) {
      self.bar.next = "Oy!"; // Type issue here...
   }
}
```

## HDL Kernel Signature

:::info
The `#[hdl_gen]` attribute can only be applied to a function (aka HDL Kernel) that
takes `&mut self` as an argument. 
:::

In almost all cases, you will write something like:

```rust
struct Foo {}

impl Logic for Foo {
  #[hdl_gen]
  fn update(&mut self) {
     // Put your synthesizable subset of Rust here...
  }
}
```

:::info
The body of the `update` function must be a single block, consisting of statements.
Local definitions and items are not allowed in HDL kernels.  The following, for example, will
fail.  
:::

This is an example of valid Rust that is not allowed in an HDL kernel.

```rust
struct Foo {}

impl Logic for Foo {
   #[hdl_gen]
   fn update (&mut self) {
     // Fails because local items are not allowed in HDL kernels.
     let x = 32;
   }
}
```

## Assignments

:::info
Assignments are allowed as long as you follow the rules about signals.  Types are
still enforced by Rust.
:::

:::warning
Indexed assignments are currently not supported
:::

:::warning
Signal assignments must be to either `.next` or `.next.field` if the signal is struct based.
:::

So valid assignments will be of the form `self.<signal>.next = <expr>`, or for structure-valued
signals.

- Expressions support accessing fields of a signal
- Binary operations supported are `+`, `-`, `*`, `&&`, `||`, `^`, `&`, `|`, `<<`, `>>`, `==`, `<`, `<=`, `!=`, `>`, `>=`
In general, binary operations require that both arguments are of the same type (e.g. bitwidth) or one of the
arguments will be a literal.
```rust
# use rust_hdl::prelude::*;

struct Foo {
   pub sig1: Signal<In, Bits<4>>,
   pub sig2: Signal<In, Bits<4>>,
   pub sig3: Signal<Out, Bits<4>>,
}

impl Logic for Foo {
   #[hdl_gen]
   fn update(&mut self) {
      self.sig3.next = self.sig1.val() + 4; // Example of binop with a literal
      self.sig3.next = self.sig1.val() ^ self.sig2.val(); // Example of a binop with two bitvecs
   }
}
```

## Unary operators

- Unary operations supported are `-` and `!`
The `-` operator is only supported for `Signed` types.  Otherwise, it makes no sense.  If
you want to compute the 2's complement of an unsigned value, you need to do so explicitly.
The `!` operator will flip all of the bits in the bitvector.

## Conditionals

Conditionals (`if`) are supported.

```rust
# use rust_hdl::prelude::*;

struct Foo {
    pub sig1: Signal<In, Bit>,
    pub sig2: Signal<Out, Bits<2>>,
    pub sig3: Signal<In, Bits<2>>,
    pub sig4: Signal<Out, Bits<2>>,
}

impl Logic for Foo {
    #[hdl_gen]
    fn update(&mut self) {
        self.sig2.next = 0.into(); // Latch prevention!
        // Straight `if`s are supported, but beware of latches!
        // This `if` statement would generate a latch if not for
        // the unconditional assign to `sig2`
        if self.sig1.val() {
           self.sig2.next = 1.into();
        }
        // You can use `else` clauses also
        if self.sig1.val() {
           self.sig2.next = 1.into();
        } else {
           self.sig2.next = 2.into();
        }
        // Nesting and chaining are also fine
        if self.sig3.val() == 0 {
           self.sig4.next = 3.into();
        } else if self.sig3.val() == 1 {
           self.sig4.next = 2.into();
        } else {
           self.sig4.next = 0.into();   // <- Fall through else prevents latch
        }
    }
}
```

## Literals and Function Calls
- Literals (provided they implement the `Synth` trait) are supported.  In most cases, you
can used un-suffixed literals (like `1` or `0xDEAD`) as add `.into()`.
- Function calls - RustHDL kernels support a very limited number of function calls, all of
   which are ignored in HDL at the moment (they are provided to make `rustc` happy)
    - `bit_cast`
    - `signed_bit_cast`
    - `unsigned_cast`
    - `bits`
    - `Bits`
    - `Type::join` and `Type::link` used to link and join logical interfaces...
- Method calls - Kernels support the following limited set of method calls
    - `get_bits` - extract a (fixed width) set of bits from a bit vector
    - `get_bit` - extract a single bit from a bit vector
    - `replace_bit` - replace a single bit in a bit vector
    - `all` - true if all the bits in the bit vector are true
    - `any` - true if any of the bits in the bit vector are true
    - `xor` - true if the number of ones in the bit vector is odd
    - `val`, `into`, `index`, `to_bits` - ignored in HDL kernels
```rust
# use rust_hdl::prelude::*;

struct Foo {
    pub sig1: Signal<In, Bits<8>>,
    pub sig_index: Signal<In, Bits<3>>,
    pub sig2: Signal<Out, Bit>,
    pub sig3: Signal<Out, Bits<3>>,
    pub sig4: Signal<Out, Bit>,
}

impl Logic for Foo {
    #[hdl_gen]
    fn update(&mut self) {
        self.sig2.next = self.sig1.val().get_bit(self.sig_index.val().index()); // <- Selects specified bit out of sig1
        self.sig3.next = self.sig1.val().get_bits::<3>(self.sig_index.val().index()); // Selects 3 bits starting at index `sig_index`
        // Notice that here we have an output on both the left and right side of the assignment
        // That is fine as long we we write to `.next` before we read from `.val`.
        self.sig4.next = self.sig3.val().all(); // True if sig3 is all true
    }
}
```

## Matches

- Matches - Kernels support matching with literals or identifiers
Matches are used for state machines and implementing ROMs.  
For now, `match` is a statement, not an expression!  Maybe that will be fixed in a future
version of RustHDL, but for now, the value of the `match` is ignored.
Here is an example of a `match` for a state machine:
```rust
# use rust_hdl::prelude::*;
# use rust_hdl::widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    Running,
    Paused,
}


struct Foo {
    pub start: Signal<In, Bit>,
    pub pause: Signal<In, Bit>,
    pub stop: Signal<In, Bit>,
    pub clock: Signal<In, Clock>,
    state: DFF<State>,
}

impl Logic for Foo {
    #[hdl_gen]
    fn update(&mut self) {
       dff_setup!(self, clock, state); // <- setup the DFF
       match self.state.q.val() {
           State::Idle =>
                  if self.start.val() {
                     self.state.d.next = State::Running;
                  }
           State::Running =>
                  if self.pause.val() {
                     self.state.d.next = State::Paused;
                  }
           State::Paused =>
                  if !self.pause.val() {
                     self.state.d.next = State::Running;
                  }
       }
       if self.stop.val() {
           self.state.d.next = State::Idle;
       }
    }
}
```

## Macros

- Macros - some macros are supported in kernels
    - `println` - this is converted into a comment in the generated HDL
    - `comment` - also a comment
    - `assert` - converted to a comment
    - `dff_setup` - setup a DFF - this macro is converted into the appropriate HDL
    - `clock` - clock a set of components - this macro is also converted into the appropriate HDL

## Loops 
- Loops - `for` loops are supported for code generation

:::info
In software parlance, all `for` loops are unrolled at compile time, so they must be of the form `for <ident> in <const>..<const>`.
:::

A simple example to consider is a parameterizable mux.

```rust
# use rust_hdl::prelude::*;

// Mux from N separate signals, using A address bits
// For fun, it's also generic over the width of the
// signals being muxed.  So there are 3 generics here:
//    - D - the type of those signals
//    - N - the number of signals being muxed
//    - A - the number of address bits (check that 2^A >= N)
struct Mux<D: Synth, const N: usize, const A: usize> {
   pub input_lines: [Signal<In, D>; N],
   pub select: Signal<In, Bits<A>>,
   pub outsig: Signal<Out, D>,
   fallback: Constant<D>,
}

// The impl for this requires a for loop
impl<D: Synth, const N: usize, const A: usize> Logic for Mux<D, N, A> {
  #[hdl_gen]
  fn update(&mut self) {
       self.outsig.next = self.fallback.val();
       for i in 0..N {
          if self.select.val().index() == i {
             self.outsig.next = self.input_lines[i].val();
          }
       }
   }
}
```
RustHDL is still pretty restrictive about arrays and loops.  You can still do great stuff though.

Since an example is instructive, here is the HDL kernel for a nontrivial circuit (the `SPIMaster`),
annotated to demonstrate the various valid bits of syntax.  It's been heavily redacted to make
it easier to read.

```rust
// Note - you can use const generics in HDL definitions and kernels!
#[derive(LogicBlock)]
struct SPIMaster<const N: usize> {
    // The `pub` members are the ones you can access from other circuits.
    // These form the official interface of the circuit
    pub clock: Signal<In, Clock>,
    pub bits_outbound: Signal<In, Bits<16>>,
    pub data_outbound: Signal<In, Bits<N>>,
    // snip...
    // These are private, so they can only be accessed by internal code
    register_out: DFF<Bits<N>>,
    register_in: DFF<Bits<N>>,
    state: DFF<SPIState>,
    strobe: Strobe<32>,
    pointer: DFF<Bits<16>>,
     // snip...
    // Computed constants need to be stored in a special Constant field member
    cs_off: Constant<Bit>,
    mosi_off: Constant<Bit>,
}

impl<const N: usize> Logic for SPIMaster<N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Setup the internals - for Latch avoidance, each digital flip flop
        // requires setup - it needs to be clocked, and it needs to connect
        // the output and input together, so that the input is driven.
        // This macro simply declutters the code a bit and makes it easier to read.
        dff_setup!(
            self,
            clock,
            //   | equivalent to `self.register_out.clock.next = self.clock.val();`
            // v--               `self.register_out.d.next = self.register_out.q.val();`
            register_out,
            register_in,
            state,
            pointer,
        );
        // This macro is shorthand for `self.strobe.next = self.clock.val();`
        clock!(self, clock, strobe);
        // These are just standard assignments... Nothing too special.
        // Note that `.next` is on the LHS, and `.val()` on the right...
        self.strobe.enable.next = true;
        self.wires.mclk.next = self.clock_state.q.val();
        self.wires.msel.next = self.msel_flop.q.val();
        self.data_inbound.next = self.register_in.q.val();
        self.pointerm1.next = self.pointer.q.val() - 1;
        // The `match` is used to model state machines
        match self.state.q.val() {
            SPIState::Idle => {
                self.busy.next = false;
                self.clock_state.d.next = self.cpol.val();
                if self.start_send.val() {
                    // Capture the outgoing data in our register
                    self.register_out.d.next = self.data_outbound.val();
                    self.state.d.next = SPIState::Dwell; // Transition to the DWELL state
                    self.pointer.d.next = self.bits_outbound.val(); // set bit pointer to number of bit to send (1 based)
                    self.register_in.d.next = 0.into(); // Clear out the input store register
                    self.msel_flop.d.next = !self.cs_off.val(); // Activate the chip select
                    self.continued_save.d.next = self.continued_transaction.val();
                } else {
                    if !self.continued_save.q.val() {
                        self.msel_flop.d.next = self.cs_off.val(); // Set the chip select signal to be "off"
                    }
                }
                self.mosi_flop.d.next = self.mosi_off.val(); // Set the mosi signal to be "off"
            }
            SPIState::Dwell => {
                if self.strobe.strobe.val() {
                    // Dwell timeout has reached zero
                    self.state.d.next = SPIState::LoadBit; // Transition to the loadbit state
                }
            }
            SPIState::LoadBit => {
                // Note in this statement that to use the pointer register as a bit index
                // into the `register_out` DFF, we need to convert it with `index()`.
                if self.pointer.q.val().any() {
                    // We have data to send
                    self.mosi_flop.d.next = self
                        .register_out
                        .q
                        .val()
                        .get_bit(self.pointerm1.val().index()); // Fetch the corresponding bit out of the register
                    self.state.d.next = SPIState::MActive; // Move to the hold mclock low state
                    self.clock_state.d.next = self.cpol.val() ^ self.cpha.val();
                } else {
                    self.mosi_flop.d.next = self.mosi_off.val(); // Set the mosi signal to be "off"
                    self.clock_state.d.next = self.cpol.val();
                    self.state.d.next = SPIState::Finish; // No data, go back to idle
                }
            }
            SPIState::MActive => {
                if self.strobe.strobe.val() {
                    self.state.d.next = SPIState::SampleMISO;
                }
            }
       }
    }
}
```

## Enums

In keeping with Rust's strongly typed model, you can use enums (not sum types) in your HDL,
provided you derive the `LogicState` trait for them.  This makes your code much easier to
read and debug, and `rustc` will make sure you don't do anything illegal with your
enums.
 
```rust
# use rust_hdl::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    Running,
    Paused,
}
```

Using enums for storing things like state has several advantages:
- RustHDL will automatically calculate the minimum number of bits needed to store the
enum in e.g., a register.

For example, we can create a Digital Flip Flop (register) of value `State` from the next
example, and RustHDL will convert this into a 2 bit binary register.  

```rust
# use rust_hdl::prelude::*;
# use rust_hdl::widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    Sending,
    Receiving,
    Done,
}

struct Foo {
    dff: DFF<State>,  // <-- This is a 2 bit DFF
}
```

Now imagine we add another state in the future to our state machine - say `Pending`:

```rust
# use rust_hdl::prelude::*;
# use rust_hdl::widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    Sending,
    Receiving,
    Pending,
    Done,
}

struct Foo {
    dff: DFF<State>,  // <-- This is now a 3 bit DFF!
}
```
RustHDL will _automatically_ choose a 3-bit representation.  

- RustHDL will ensure that assignments to `enum`-valued signals are valid at all times

The strong type guarantees ensure you cannot assign arbitrary values to `enum` valued
signals, and the namespaces ensure that there is no ambiguity in assignment.  This example
won't compile, since `On` without the name of the `enum` means nothing, and `State1` and
`State2` are separate types.  They cannot be assigned to one another.

:::danger
This example won't compile.
:::

```rust
# use rust_hdl::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State1 {
     On,
     Off,
}

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State2 {
     Off,
     On,
}

struct Foo {
    pub sig_in: Signal<In, State1>,
    pub sig_out: Signal<Out, State2>,
}

impl Logic for Foo {
    #[hdl_gen]
    fn update(&mut self) {
        self.sig_out.next = On; // << This won't work either.
        self.sig_out.next = self.sig_in.val(); // << Won't compile
    }
}
```

If for some reason, you needed to translate between enums, use a `match`:

```rust
impl Logic for Foo {
   #[hdl_gen]
   fn update(&mut self) {
      match self.sig_in.val() {
          State1::On => self.sig_out.next = State2::On,
          State1::Off => self.sig_out.next = State2::Off,
      }
   }
}
```

