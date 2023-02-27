use rust_hdl_lib_core::prelude::*;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct OutputDDR {
    pub d: Signal<In, Bits<2>>,
    pub clock: Signal<In, Clock>,
    pub q: Signal<Out, Bit>,
    pub reset: Signal<In, Bit>,
    _capture: Bits<2>,
}

impl Logic for OutputDDR {
    fn update(&mut self) {
        if self.clock.pos_edge() {
            self._capture = self.d.val();
            self.q.next = self._capture.get_bit(0);
        }
        if self.clock.neg_edge() {
            self.q.next = self._capture.get_bit(1);
        }
        if self.reset.val().into() {
            self._capture = 0.into();
            self.q.next = false;
        }
    }
    fn connect(&mut self) {
        self.q.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: r##"
ODDRX1F inst_ODDRX1F(.SCLK(clock), .RST(reset), .D0(d[0]), .D1(d[1]), .Q(q));
            "##
            .into(),
            cores: r##"
(* blackbox *)
module ODDRX1F(input D0, input D1, input SCLK, input RST, output Q);
endmodule
            "##
            .into(),
        })
    }
}

#[test]
fn test_oddr_synthesizes() {
    let mut uut = OutputDDR::default();
    uut.connect_all();
    yosys_validate("oddr", &generate_verilog(&uut)).unwrap();
}
