use crate::core::prelude::*;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct EdgeTristateBuffer {
    pub d: Signal<InOut, Bit>,
    pub q: Signal<InOut, Bit>,
    pub clk: Signal<In, Clock>,
}

impl Logic for EdgeTristateBuffer {
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



            "##.into(),
            cores: r##"
(* blackbox *)
module IFS1P3DX(input D, input SP, input SCLK, input CD, output Q);
endmodule

(* blackbox *)
module OFS1P3DX(input D, input SP, input SCLK, input CD, output Q);
endmodule
            "##.into(),
        })
    }
}