# Interfaces

One area you will encouter as your circuits become more complex is that the interfaces
to those circuits will become increasingly complicated.  To demonstrate, suppose you
have a circuit that consumes a sequence of 16-bit integers via a FIFO interface.  The
circuit has some flow control signals because it cannot consume them every clock
cycle (For Reasons).  Suppose also that you have a data producer circuit that will
produce 16-bit integers and you want to connect these two together.  A natural
FIFO interface would look like this

```rust
# use rust_hdl::prelude::*;
 struct MyFIFO {
     pub data_to_fifo: Signal<In, Bits<16>>,
     pub write: Signal<In, Bits<16>>,
     pub full: Signal<Out, Bit>,
     pub overflow: Signal<Out, Bit>,
 }

 struct DataWidget {
     pub data_to_fifo: Signal<Out, Bits<16>>,
     pub write: Signal<Out, Bits<16>>,
     pub full: Signal<In, Bit>,
     pub overflow: Signal<In, Bit>,
 }

 struct Foo {
    producer: DataWidget,
    consumer: MyFIFO,
 }
```

Now, we want to connect the output of the DataWidget (all 4 signals!) to the corresponding
signals on `MyFIFO`.  Keep in mind that the order of assignment is irrelevant, but which
signal appears on the LHS vs RHS _is_ important.  In the `impl Logic` block for `Foo`,
our HDL kernel will look like this:
```rust
impl Logic for Foo {
  #[hdl_gen]
  fn update(&mut self) {
     self.consumer.data_to_fifo.next = self.producer.data_to_fifo.val();
     self.consumer.write.next = self.producer.write.val();
     self.producer.full.next = self.consumer.full.val();
     self.producer.overflow.next = self.consumer.overflow.val();
  }
}
```
This is basically boilerplate at this point, and typing that in and getting it right
is error prone and tedious.  Fortunately, RustHDL can help!  RustHDL includes the
concept of an `Interface`, which is basically a bus.  An `Interface` is generally a
pair of structs that contain signals of complementary directions and a `#[derive]`
macro that autogenerates a bunch of boilerplate.  To continue on with our previous
example, we could define a pair of `struct`s for the write interface of the FIFO

```rust
#[derive(LogicInterface)]     // <- Note the LogicInterface, not LogicBlock
#[join = "MyFIFOWriteSender"] // <- Name of the "mating" interface
struct MyFIFOWriteReceiver {
    pub data_to_fifo: Signal<In, Bits<16>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub overflow: Signal<Out, Bit>,
}

#[derive(LogicInterface)]       // <- Also here
#[join = "MyFIFOWriteReceiver"] // <- Name of the "mating" interface
struct MyFIFOWriteSender {
   pub data_to_fifo: Signal<Out, Bits<16>>,
   pub write: Signal<Out, Bit>,
   pub full: Signal<In, Bit>,
   pub overflow: Signal<In, Bit>
}
```
The names of the fields must match, the types of the fields must also match, and the directions
of the signals must be complementary.  So in general:

- Each field in struct `A` must have a matching named field in struct `B`
- The types of those fields must match
- The direction of those signals must be opposite
- Order of the fields is immaterial
- The `join` attribute tells the compiler which interface to mate to this one.

So what can we do with our shiny new interfaces?  Plenty of stuff.  First, lets
rewrite our FIFO circuit and data producer to use our new interfaces.

```rust
struct MyFIFO {
    // The write interface to the FIFO - now only one line!
    pub write_bus: MyFIFOWriteReceiver,
}

struct DataWidget {
    // The output interface from the DataWidget!
    pub data_out: MyFIFOWriteSender,
}
```

That is significantly less verbose!  So what happens to our
`impl Logic for Foo`?  Well, RustHDL autogenerates 2 methods for each `LogicInterface`.  The first
one is called `join`.  And it, well, joins the interfaces.

```rust
impl Logic for Foo {
   #[hdl_gen]
   fn update(&mut self) {
      // Excess verbosity eliminated!!
      MyFIFOWriteSender::join(&mut self.producer.data_out, &mut self.consumer.write_bus);
   }
}
```

This is exactly equivalent to our previous 4 lines of hand crafted code, but is now automatically
generated _and_ synthesizable.  But wait!  There is more.  RustHDL also generates a `link`
method, which allows you to _forward_ a bus from one point to another.  If you think in terms
gendered cables, a `join` is a cable with a Male connector on one end and a Female connector
on the other.  A `link` is a cable that is either Male to Male or Female to Female.  Links
are useful when you want to forward an interface to an interior component of a circuit, but
hide that interior component from the outside world.  For example, lets suppose that
`DataWidget` doesn't actually produce the 16-bit samples.  Instead, some other FPGA component
or circuit generates the 16-bit samples, and `DataWidget` just wraps it along with some
other control logic.  So in fact, our `DataWidget` has an internal representation that looks
like this

```rust
struct DataWidget {
   pub data_out: MyFIFOWriteSender,
   secret_guy: CryptoGenerator,
   running: DFF<Bit>,
}

struct CryptoGenerator {
   pub data_out: MyFIFOWriteSender,
   // secret stuff!
}
```  

In this example, the `DataWidget` wants to present the outside world that it is a `MyFIFOWriteSender`
interface, and that it can produce 16-bit data values.  But the real work is being done internally
by the `secret_guy`.  The manual way to do this would be to connect up the signals manually.  Again,
paying attention to which signal is an input (for `DataWidget`), and which is an output.

```rust
impl Logic for DataWidget {
   #[hdl_gen]
    fn update(&mut self) {
       // Yawn...
       self.data_out.data_to_fifo.next = self.secret_guy.data_out.data_to_fifo.val();
       self.data_out.write.next = self.secret_guy.data_out.write.val();
       self.secret_guy.data_out.full.next = self.data_out.full.val();
       self.secret_guy.data_out.overflow.next = self.data_out.overflow.val();
    }
}
```

In these instances, you can use the `link` method instead.  The syntax is
`Interface::link(&mut self.outside, &mut self.inside)`, where `outside` is the
side of the interface going out of the circuit, and `inside` is the side of the interface
inside of the circuit.  Hence, our interface can be `forwarded` or `linked` with a single line
like so:
```rust
impl Logic for DataWidget {
   #[hdl_gen]
    fn update(&mut self) {
       // Tada!
       MyFIFOWriteSender::link(&mut self.data_out, &mut self.secret_guy.data_out);
    }
}
```

As a parting note, you can make interfaces generic across types.  Here, for example
is the FIFO interface used in the High Level Synthesis library in RustHDL:

```rust
#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "FIFOWriteResponder"]
pub struct FIFOWriteController<T: Synth> {
    pub data: Signal<Out, T>,
    pub write: Signal<Out, Bit>,
    pub full: Signal<In, Bit>,
    pub almost_full: Signal<In, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "FIFOWriteController"]
pub struct FIFOWriteResponder<T: Synth> {
    pub data: Signal<In, T>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub almost_full: Signal<Out, Bit>,
}
```

You can then use any synthesizable type for the data bus, and keep the control signals
as single bits!  Neat, eh? 🦑

