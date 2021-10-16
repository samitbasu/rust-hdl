use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct WireOut {
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub datain: Signal<In, Bits<16>>,
    _n: u8,
}

impl WireOut {
    // TODO - add collision detection
    pub fn new(port: u8) -> Self {
        assert!(port >= 0x20 && port < 0x40);
        Self {
            ok1: Default::default(),
            ok2: Default::default(),
            datain: Default::default(),
            _n: port,
        }
    }
}

impl Logic for WireOut {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.ok2.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("WireOut_{:x}", self._n);
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
                VerilogLiteral::from(self._n)
            ),
            name,
        })
    }
}

#[test]
fn test_wire_out_synth() {
    let mut uut = rust_hdl_synth::TopWrap::new(WireOut::new(0x20));
    uut.uut.ok1.connect();
    uut.uut.datain.connect();
    uut.uut.connect_all();
    rust_hdl_synth::yosys_validate("wire_out", &generate_verilog(&uut)).unwrap();
}

#[derive(Clone, Debug, LogicBlock)]
pub struct WireIn {
    pub ok1: Signal<In, Bits<31>>,
    pub dataout: Signal<Out, Bits<16>>,
    _n: u8,
}

impl WireIn {
    pub fn new(n: u8) -> WireIn {
        assert!(n < 0x20);
        WireIn {
            ok1: Default::default(),
            dataout: Default::default(),
            _n: n,
        }
    }
}

impl Logic for WireIn {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.dataout.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("WireIn_{:x}", self._n);
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
                VerilogLiteral::from(self._n)
            ),
            name,
        })
    }
}

#[test]
fn test_wire_in_synth() {
    let mut uut = rust_hdl_synth::TopWrap::new(WireIn::new(0x02));
    uut.uut.ok1.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("wire_in", &generate_verilog(&uut)).unwrap();
}
