So.... I wonder if RustHDL would benefit from a more functional data-flow style,
as opposed to the oop style it currently uses.  In that case, I would separate
data (which is state) from functions (which are stateless).  This is similar to the
notion of logic being divided into states (memory) and functions (combinational logic).

The problem is one of composition.  As the same problem occurs in
the land of UIs.  

For example, let's look at the example of the strobe widget:

```rust
#[derive(Clone, Debug, LogicBlock)]
pub struct Strobe<const N: usize> {
    /// Set this to true to enable the pulse train.
    pub enable: Signal<In, Bit>,
    /// This is the strobing signal - it will fire for 1 clock cycle such that the strobe frequency is generated.
    pub strobe: Signal<Out, Bit>,
    /// The clock that drives the [Strobe].  All signals are synchronous to this clock.
    pub clock: Signal<In, Clock>,
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}
```

How do you do this in a purely functional form?  One way is to 
split the state out,  (in this case, the counter), and feed it
externally.

```rust
fn strobe<params>(enable, state_in) -> (strobe, state_out) {
   state_out = state_in;
   if enable {
      state_out.counter = state_in.counter + 1;
   }
   strobe = enable & (state_in.counter == params.threshold);
   if strobe {
     state_out.counter = 1;
   }
}
```

Note the absence of the clock input...
So far, this is fine.  What about composition?  Consider now
a pulser:

```rust
#[derive(LogicBlock)]
pub struct Pulser {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub pulse: Signal<Out, Bit>,
    strobe: Strobe<32>,
    shot: Shot<32>,
}
```

Here, we have internal state (the strobe and shot).  So the
function needs to look something like this:

```rust
fn pulser<params>(enable, state_in) -> (pulse, state_out) {
    state_out = state_in;
    let (strobe, state_out.strobe) = strobe<params.strobe>(enable, state_in.strobe);
    let (active, state_out.shot) = shot<params.shot>(strobe, state_in.shot);
    let pulse = active
}
```

The state is a simple composition of the input state

```rust
pub struct PulserState<N> {
    strobe: StrobeState<N>,
    shot: ShotState<N>,
}
```

Advantages: 
   - Signal directions are no longer needed
   - Priority is natural

Some examples...

An adder.

This is pretty simple:

fn add<const N: usize>(a: Bits<N>, b: Bits<N>) -> Bits<N> {
    a + b
}

What about a counter?  This has state.  The update function needs the current
state:

fn counter<const N: usize>(count: Bits<N>) -> Bits<N> {
    count + 1
}

In fairness, this isn't really a counter.  It's an incrementer function.  To 
build a counter, we need a bit more.

So we need a place to store the current count, and the counter update function.
In the current version of RustHDL, these two things coexist in a single struct.

But what if they are different?

/// The state of the counter
struct Counter<const N: usize> {
    count: Bits<N>
}

/// The update for the counter state
fn counter<const N: usize>(count: Bits<N>) -> Bits<N> {
    count + 1
}

/// The clock domain that ties them together.
fn clock<const N: usize>(mut state: Counter<N>, update: impl Fn(state) -> state) {
    loop {
        state = update(state);
    }
}

Simple enough, but there is no way to get inputs or outputs out of our counter.
Let's say, we want an enable signal.  This is not part of the state.  Just
an accessory input:

fn counter<const N: usize>(count: Bits<N>, enable: bool) -> Bits<N> {
    count + enable
}

The clock can now take the enable signal as an input:

fn clock(mut state: Counter<N>, enable, update) {
    loop {
        state = update(state, enable);
    }
}

What if there are outputs?  For example, suppose our counter sends
out a pulse whenever the counter rolls to zero.

fn counter<const N: usize>(count: Bits<N>, enable: bool) -> (Bits<N>, bool) {
    (count + enable, count == 0)
}

This will quickly get messy, though.  So better to have three type definitions:

fn update<S, I, O>(state: S, inputs: I) -> (S, O) {}

Then we can have the update written as is, or we can have multiple inputs and outputs
with names.  

The counter is a bit contrived.  Let's try a RAM.  Assume that it stores data
of type D (whatever that may be).  For now, let's assume a single clock domain.

struct RAMInputs<const N: usize, D> {
    read_address: Bits<N>,
    write_address: Bits<N>,
    write_data: D,
    write_enable: bool,
}

struct RAMOutputs<const N: usize, D> {
    read_data: D,
}

struct RAMState<const N: usize, D> {
    // This is just for simulation...
    // The actual state for hardware is stored in a BRAM or something equivalent
    _sim: Box<BTreeMap<Bits<N>, D>>
}

// The update function...
fn update(mut state: RAMState<N, D>, in: RAMInputs) -> (RAMState<N, D>, RAMOutputs) {
    // This depends on the exact behavior
    out = state._sim.get(in.read_address);
    if in.write_enable {
        state._sim.insert(in.write_address, in.write_data)
    }
    (state, out)
}

// That also looks reasonable.  A FIFO should also be reasonable.
struct FIFOInputs<D> {
    write: bool,
    data_in: D,
    read: bool,
}

struct FIFOOutputs<D> {
    data_out: D,
    empty: bool,
    almost_empty: bool,
    underflow: bool,
    full: bool,
    almost_full: bool,
    overflow: bool,
}

// The internal state of the FIFO has a bunch of stuff
struct FIFOState<D> {
    // RAM, counters, etc.
}

Hmmm... Too complicated.  Let's back up to something simpler.  Like the LFSR.

struct LFSRState {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

struct LFSRInputs {
    strobe: bool,
}

struct LFSROutputs {
    num: u32,
}

fn lfsr(mut state: LFSRState, in: LFSRInputs) -> (LFSRState, LFSROutputs) {
    let out = LFSROutputs {num: state. w};
    let t = state.x ^ (state.q << 11);
    if in.strobe {
        state.x = state.y;
        state.y = state.z;
        state.z = state.w;
        state.w = state.w ^ (w ...)
    }
}

Something like a pipeline register is easy to do...
struct pipeline<T> {
    state: T
}

fn pipeline(mut state: pipeline<T>, in: T) -> (pipeline<T>, T) {
    (in, state)
}


Hmm... There are some problems here?  What about multiple dependent reads/writes?

fn test(mut state: u32, ()) -> (u32, ()) {
    state += 1;
    state += 1;   
}

This is ok too, since sequence assignments are simply unrolled as logic.

Why do we not need iteration and a solver?  It's not clear.  I guess it's because
in this case, the data flow dependencies are explicit, not implicit.


There are a couple of open questions.

- How to handle multiple clock domains?  A clock domain should own it's state.


What about busses?  What happens when we have many inputs and outputs coupled
together logically?  We can no longer have inputs and outputs in the same "struct".

So, suppose we have a component that wants to write to a FIFO when it is not
full.  It will look something like this

struct FIFOWriteIn {
    full: bool,
    enable: bool,
}

struct FIFOWriteOut {
    write: bool,
    data: T,
}

struct FIFOWriteState {
    lsfr: LSFRState,
}

fn fifo_writer(my_state, fifo_in) -> (my_state, fifo_out) {
    let will_write = fifo_in.enable && !fifo_in.full;
    let (my_state, lsfr_out) = lsfr(my_state.lsfr, will_write);
    let mut fifo_out : FIFOWriteOut = {write: false, data: my_state.lsfr.}
    if fifo_in.enable && !fifo_in.full {
        fifo_out.write = true;
    }
}

Composition is an interesting issue.  In React, there is often a concern
that you must lift state to the parent component so that other components
can communicate.  For example, if you want A and B to communicate
then they must share state in some component C that contains both A and B.

We could do this...

trait logic {
    fn update<I: Synth, O: Synth>(self, I) -> (Self, O) {}
}

And then impl the trait on the state.

struct FIFOWriteState {
    lsfr: LSFRState,
}

impl logic for FIFOWriteState {
    fn update(self, input: FIFOWriteIn) -> (FIFOWriteState, FIFOWriteOut) {

    }
}

Joins

I don't think we need Joins anymore?  They come naturally from the assumption
that Synth: Copy.  So we can have:

fn update(BusControllerState, BusInputs) -> (BusControllerState, BusOutputs) {
    // The inputs are "consumed" by the controller, since they are passed
    // in by value.  
}

Let's take this a bit further.  Suppose we have a simple bus with an address line
a write enable, and a read enable.  Something like

struct BusInputs {
    address: u8,
    write_enable: bool,
    write_data: D,
    read_enable: bool,
}

struct BusOutputs {
    read_data: D,
}

We want to connect to this bus with a device that looks for a given address
and then responds to reads and writes (like a register)

struct BusRegister {
    my_value: D,
    my_address: u8
}

impl logic for BusRegister {
    fn update(mut self, input: BusInputs) -> (Self, BusOutputs) {
        let mut outputs = BusOutputs::default();
        if input.write_enable && input.address == self.my_address {
            self.my_value = input.write_data;
        }
        if input.read_enable && input.address == self.my_address {
            BusOutputs.write_data = self.my_value
        }
        (self, outputs)
    }
}

This brings up the point of how to combine values.  If we assume Synth -> Or, then
we can simply Or all of the outputs together.  We have to be careful, as this won't
work for enums, for example.


For now, we can have a bus controller that simply broadcasts and ors.

struct BusFoo {
    reg_1: BusRegister,
    reg_2: BusRegister,
}

impl logic for BusFoo {
    fn update(mut self, input: BusInputs) -> (Self, BusOutputs) {
        let (reg_1_next, reg_1_out) = reg_1.update(input);
        let (reg_2_next, reg_2_out) = reg_2.update(input);
        (Self {reg_1, reg_2}, reg_1_out | reg_2_out)
    }
}

In general, if we have a trait for this

trait BusThing {
    fn update(mut self, input: BusInputs) -> (Self, BusOutputs);
}


What about passing signals deep into the hierarchy.  For example, suppose
we have a reset signal or an enable that needs to make it down..
Each component along the way will need to copy it.

struct FooInputs {
    reset: bool,
    stuff: junk
}

struct FooOutputs {
    reset: bool,
    stuff: crap
}


State machines should be super easy...

struct StateMachine {
    state: MyState (enum!),
}

impl logic for StateMachine {
    fn update(mut self, input: enable) -> (Self, bool) {
        output = self.state == Running;
        match self.state {
            Idle => { 
                if input {
                    self.state = Running;
                }
            }
            Running => {
                if !input {
                    self.state = Idle;
                }
            }
        }
        (self, output)
    }
}



Early returns and control flow?  This seems more like a Verilog implementation 
detail.  First pass would be to prohibit early returns.

What about clock crossings?  

We have 2 clocks, and some state in clock domain 1 and some state in clock domain 2.

struct Domain1 {
    stuff
}

struct Domain2 {
    stuff
}

We need something like...

A synchronous machine is a tuple...

fn synchronous_thing(clock, state, update, inputs) -> outputs {
    loop {
        on_clock_edge {
            inputs = get_inputs_from_outside();
            let (state, outputs) = state.update(inputs);
            push_outputs_to_outside(outpus);
        }
    }
}


A synchronizer is simply:

struct synchronizer {
    d1: bool,
    d2: bool,
}

fn update(self, input: bool) -> (Self, bool) {
    (Self {
        d1: input,
        d2: self.d1
    }, self.d2)
}

