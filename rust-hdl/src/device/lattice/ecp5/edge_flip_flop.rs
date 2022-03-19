use crate::core::prelude::*;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct EdgeFlipFlop {
    pub d: Signal<In, Bit>,
    pub q: Signal<Out, Bit>,
    pub clk: Signal<In, Clock>,
}

impl Logic for EdgeFlipFlop {
    fn update(&mut self) {
        if self.clk.pos_edge() {
            self.q.next = self.d.val()
        }
    }
    fn connect(&mut self) {
        self.q.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: r##"
OFS1P3DX inst_OFS1P3DX(.SCLK(clk), .SP(1), .D(d), .Q(q), .CD(0));
            "##.into(),
            cores: r##"
(* blackbox *)
module OFS1P3DX(input D, input SP, input SCLK, input CD, output Q);
endmodule
            "##.into(),
        })
    }
}

#[test]
fn test_eflop_synthesizes() {
    let mut uut = TopWrap::new(EdgeFlipFlop::default());
    uut.uut.d.connect();
    uut.uut.clk.connect();
    uut.connect_all();
    yosys_validate("eflop", &generate_verilog(&uut)).unwrap();
}