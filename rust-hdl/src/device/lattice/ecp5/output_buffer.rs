use crate::core::prelude::*;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct OutputBuffer {
    pub i: Signal<In, Bit>,
    pub o: Signal<Out, Bit>,
}

impl Logic for OutputBuffer {
    fn update(&mut self) {
        self.o.next = self.i.val();
    }
    fn connect(&mut self) {
        self.o.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: r##"
OB inst_OB(.I(i), .O(o));
            "##.into(),
            cores: r##"
(* blackbox *)
module OB(input I, output O);
endmodule
            "##.into()
        })
    }
}

#[test]
fn test_output_buffer_synthesizes() {
    let mut uut = TopWrap::new(OutputBuffer::default());
    uut.uut.i.connect();
    uut.connect_all();
    yosys_validate("obuf", &generate_verilog(&uut)).unwrap();
}