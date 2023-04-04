# High Level Synthesis

RustHDL supports it's own version of High Level Synthesis (HLS).  Normally, this is some kind
of strange drag-and-drop based entry or visual programming paradigm.  Worse, still, it could
be some kind of macro meta-language that you build complex designs out of using a graphical
editor.  

But that is not the case here!  RustHDL's HLS approach is much simpler.  Essentially,
even though [Interfaces] are so great, you may not want to use them.  So the core widgets,
like the [AsynchronousFIFO] do not use Interfaces.  That leads to some pretty gnarly
circuit definitions.  Here is the [AsynchronousFIFO] for example:
```rust
# use rust_hdl::prelude::*;
pub struct AsynchronousFIFO<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    // Read interface
    pub read: Signal<In, Bit>,
    pub data_out: Signal<Out, D>,
    pub empty: Signal<Out, Bit>,
    pub almost_empty: Signal<Out, Bit>,
    pub underflow: Signal<Out, Bit>,
    pub read_clock: Signal<In, Clock>,
    pub read_fill: Signal<Out, Bits<NP1>>,
    // Write interface
    pub write: Signal<In, Bit>,
    pub data_in: Signal<In, D>,
    pub full: Signal<Out, Bit>,
    pub almost_full: Signal<Out, Bit>,
    pub overflow: Signal<Out, Bit>,
    pub write_clock: Signal<In, Clock>,
    pub write_fill: Signal<Out, Bits<NP1>>,
    // Internals ommitted...
}
```
Using an [AsynchronousFIFO] requires up to 14 separate signals!  With mixed directions and types!
Fortunately, there is an HLS wrapper type you can use instead.  It's _much_ simpler
```rust
# use rust_hdl::prelude::*;
pub struct AsyncFIFO<T: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    pub bus_write: FIFOWriteResponder<T>,
    pub write_clock: Signal<In, Clock>,
    pub bus_read: FIFOReadResponder<T>,
    pub read_clock: Signal<In, Clock>,
    fifo: AsynchronousFIFO<T, N, NP1, BLOCK_SIZE>,
}
```

In this case, it has only 4 signals, and using it is also much easier.  You can use the
[FIFOWriteResponder] and [FIFOWriteController] busses to send and receive data from the
asynchronous fifo.  Even better is the fact that this HLS construct is just a thin wrapper
around the [AsynchronousFIFO], so when you synthesize it, or plot signals, there is nothing
extra under the hood.

The HLS library also includes a sort of System-on-chip model in case you want to use it.
It allows you to connect a set of widgets to a single controller, and route data to them
over a fixed bus using a very simple protocol.  It won't replace AXI or WishBone, but it
can be used to build some pretty complicated designs and remain readable.  The test cases
are a good place to look for some runnable examples of the different SoC widgets.
