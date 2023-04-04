# Generating Verilog

At some point, you will want to generate Verilog so you can send your design to other
tools.  This is pretty simple.  You call [generate_verilog] and pass it a reference
to the circuit in question.  The [generate_verilog] function will check your circuit,
and then return a string that contains the Verilog equivalent.  It's pretty simple.

Here is an example.  We will reuse the `MyAdder` circuit from the testbench section,
but this time, generate the Verilog for the circuit instead of simulating it.

```rust
use rust_hdl::prelude::*;   // <- shorthand to bring in all definitions

//        v--- Required by RustHDL
#[derive(LogicBlock, Default)]
struct MyAdder {
    pub sig_a: Signal<In, Bits<8>>,
    pub sig_b: Signal<In, Bits<8>>,
    pub sig_sum: Signal<Out, Bits<8>>,
    pub clock: Signal<In, Clock>,
    my_reg: DFF<Bits<8>>,
}

impl Logic for MyAdder {
  #[hdl_gen]  // <--- don't forget this
  fn update(&mut self) {
       // Setup the DFF.. this macro is handy to prevent latches
       dff_setup!(self, clock, my_reg);
       self.my_reg.d.next = self.sig_a.val() + self.sig_b.val();
       self.sig_sum.next = self.my_reg.q.val();
   }
}

let mut uut = MyAdder::default();
uut.connect_all();
println!("{}", generate_verilog(&uut));
```

You should get the following generated code in your console:
```verilog
module top(sig_a,sig_b,sig_sum,clock);

    // Module arguments
    input wire  [7:0] sig_a;
    input wire  [7:0] sig_b;
    output reg  [7:0] sig_sum;
    input wire  clock;

    // Stub signals
    reg  [7:0] my_reg$d;
    wire  [7:0] my_reg$q;
    reg  my_reg$clock;

    // Sub module instances
    top$my_reg my_reg(
        .d(my_reg$d),
        .q(my_reg$q),
        .clock(my_reg$clock)
    );

    // Update code
    always @(*) begin
        my_reg$clock = clock;
        my_reg$d = my_reg$q;
        my_reg$d = sig_a + sig_b;
        sig_sum = my_reg$q;
    end

endmodule // top


module top$my_reg(d,q,clock);

    // Module arguments
    input wire  [7:0] d;
    output reg  [7:0] q;
    input wire  clock;

    // Update code (custom)
    initial begin
       q = 8'h0;
    end

    always @(posedge clock) begin
       q <= d;
    end

endmodule // top$my_reg
```

A few things about the Verilog generated.
  - It is hierarchical (scoped) by design.  The scopes mimic the scopes inside the RustHDL circuit.
 That makes it easy to map the Verilog back to the RustHDL code if needed when debugging.
  - The code is readable and formatted.
  - The names correspond to the names in RustHDL, which makes it easy to see the details of the logic.
  - RustHDL (at least for this trivial example) is a pretty thin wrapper around Verilog.  That's
good for compatibility with tooling.

While most FPGAs will require you to use a proprietary and closed source toolchain to synthesize
your design, you can use the open source [Yosys] compiler (if you have it installed) to
check your designs.  For that, you can use the [yosys_validate] function, which runs the Verilog
through some checks and reports on potential errors.  At the moment, [Yosys] is far more
thorough in it's checking than RustHDL, so I highly recommend you install it and use the
[yosys_validate] function on your generated Verilog.
