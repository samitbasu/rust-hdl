# Traits

There is only one trait that you typically need to implement to get things to work in RustHDL
with the simulation and synthesis frameworks.  That is the [Logic](core::logic::Logic) trait.
Although you will rarely (if ever) need to implement the methods themselves, here is the
full definition of the trait:
```rust

pub trait Logic {
    fn update(&mut self);
    fn connect(&mut self) {}
    fn hdl(&self) -> Verilog {
        Verilog::Empty
    }
    fn timing(&self) -> Vec<TimingInfo> {
        vec![]
    }
}
```

The methods are quite simple:

* `update` - this updates the state of the logical block based on the inputs and internal state.
In general, this is where the action of the logical block takes place.
* `connect` - this is where we claim whatever signals we drive, by calling `connect` on them.
* `hdl` - this method returns the Verilog description for our logical block in the form of
an [Verilog](core::ast::Verilog) enum.
* `timing` - this is where specific timing exceptions or requirements are expressed for the
logical block.

:::info
In almost all cases, you will use the `#[derive(LogicBlock)]` macro to derive all of the traits from your own `update` method, written in Rust.  
:::
 
If we revisit the `Blinky` example, note that
we only provided the `update` method, with an attribute of `#[hdl_gen]`, which in turn
generated the remaining trait implementations:

```rust
 #[derive(LogicBlock)]
 struct Blinky {
    pub clock: Signal<In, Clock>,
    pulser: Pulser,
    pub led: Signal<Out, Bit>,
 }

 impl Logic for Blinky {
    #[hdl_gen]
    fn update(&mut self) {
       self.pulser.clock.next = self.clock.val();
       self.pulser.enable.next = true.into();
       self.led.next = self.pulser.pulse.val();
    }
}  
```

 There are a couple of other traits that RustHDL uses that you should be aware of.

 - `Synth` - this trait is provided on types that can be represented in hardware, i.e. as
 a set of bits.  You will probably not need to implement this trait yourself, but if you
 need some special type representation `Foo`, and `impl Synth for Foo`, then RustHDL will
 be able to generate Verilog code for it.
 - `Block` - this trait is needed on any `struct` that is a composition of circuit elements
 (pretty much every struct used to model a circuit).  This should be auto-derived.
 - `Logic` - Sometimes, you will need to override the default implementations of the `Logic`
trait.  In those cases, (when you are providing a custom simulation model, or wrapping a
black box Verilog routine), you will need to `impl` the other methods.

:::info
The main need for implementing a trait is when you want to represent some logic block
that has "magic" internals.  Like a RAM block or some special SerDes circuitry.
:::
