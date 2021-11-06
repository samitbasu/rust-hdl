use rust_hdl_core::prelude::*;
use rust_hdl_core::ast::Wrapper;
use rust_hdl_yosys_synth::TopWrap;

#[derive(LogicBlock, Default)]
pub struct ClockBuffer {
    pub clock_in: Signal<In, Clock>,
    pub clock_out: Signal<Out, Clock>,
}

impl Logic for ClockBuffer {
    fn update(&mut self) {
    }

    fn connect(&mut self) {
        self.clock_out.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(
            Wrapper {
                code: r#"
BUFG bufg_inst(.I(clock_in), .O(clock_out));
                "#.to_string(),
                cores: r#"
(* blackbox *)
module BUFG (O, I);
    output O;
    input  I;
endmodule
                "#.to_string()
            }
        )
    }
}

#[test]
fn test_clock_buffer() {
    let mut uut : TopWrap<ClockBuffer> = TopWrap::new(ClockBuffer::default());
    uut.uut.clock_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    rust_hdl_yosys_synth::yosys_validate("bufg", &vlog).unwrap();
}