use rust_hdl__core::prelude::*;

#[derive(LogicBlock, Default)]
pub struct ClockBuffer {
    pub clock_in: Signal<In, Clock>,
    pub clock_out: Signal<Out, Clock>,
}

impl Logic for ClockBuffer {
    fn update(&mut self) {}

    fn connect(&mut self) {
        self.clock_out.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: r#"
BUFG bufg_inst(.I(clock_in), .O(clock_out));
                "#
            .to_string(),
            cores: r#"
(* blackbox *)
module BUFG (O, I);
    output O;
    input  I;
endmodule
                "#
            .to_string(),
        })
    }
}

#[test]
fn test_clock_buffer() {
    let mut uut = ClockBuffer::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("bufg", &vlog).unwrap();
}
