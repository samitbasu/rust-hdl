use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, Default, LogicBlock)]
pub struct WireOut<const N: u8> {
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub datain: Signal<In, Bits<16>>,
}

impl<const N: u8> Logic for WireOut<N> {
    fn update(&mut self) {}
    fn connect(&mut self) {
        assert!(N >= 0x20 && N < 0x40);
        self.ok2.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("WireOut_{:x}", N);
        Verilog::Blackbox(BlackBox {
            code: format!(
                r#"
module {}
    (
    input wire [30:0] ok1,
    output wire [16:0] ok2,
    input wire [15:0] datain
    );

    okWireOut mod_wire(.ok1(ok1),
                       .ok2(ok2),
                  .ep_addr({:x}),
                  .ep_datain(datain));
endmodule

(* blackbox *)
module okWireOut(
    input wire [30:0] ok1,
    output wire [16:0] ok2,
    input wire [7:0] ep_addr,
    input wire [15:0] ep_datain
);
endmodule  "#,
                name,
                VerilogLiteral::from(N)
            ),
            name,
        })
    }
}

#[test]
fn test_wire_out_synth() {
    use rust_hdl_synth::top_wrap;

    top_wrap!(WireOut<0x20>, Wrapper);
    let mut uut: Wrapper = Default::default();
    uut.uut.ok1.connect();
    uut.uut.datain.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("wire_out", &generate_verilog(&uut)).unwrap();
}

#[derive(Clone, Debug, Default, LogicBlock)]
pub struct WireIn<const N: u8> {
    pub ok1: Signal<In, Bits<31>>,
    pub dataout: Signal<Out, Bits<16>>,
}

impl<const N: u8> Logic for WireIn<N> {
    fn update(&mut self) {}
    fn connect(&mut self) {
        assert!(N < 0x20);
        self.dataout.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("WireIn_{:x}", N);
        Verilog::Blackbox(BlackBox {
            code: format!(
                r#"
module {}
    (
    input wire [30:0] ok1,
    output wire [15:0] dataout
    );

    okWireIn mod_wire(.ok1(ok1),
                  .ep_addr({:x}),
                  .ep_dataout(dataout));
endmodule

(* blackbox *)
module okWireIn(
    input wire [30:0] ok1,
    input wire [7:0] ep_addr,
    output wire [15:0] ep_dataout
);
endmodule  "#,
                name,
                VerilogLiteral::from(N)
            ),
            name,
        })
    }
}

#[test]
fn test_wire_in_synth() {
    use rust_hdl_synth::top_wrap;

    top_wrap!(WireIn<0x02>, Wrapper);
    let mut uut: Wrapper = Default::default();
    uut.uut.ok1.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("wire_in", &generate_verilog(&uut)).unwrap();
}
