use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct TriggerOut {
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub clk: Signal<In, Clock>,
    pub trigger: Signal<In, Bits<16>>,
    _n: u8,
}

impl TriggerOut {
    pub fn new(n: u8) -> Self {
        assert!(n >= 60 && n < 0x80);
        Self {
            ok1: Default::default(),
            ok2: Default::default(),
            clk: Default::default(),
            trigger: Default::default(),
            _n: n,
        }
    }
}

impl Logic for TriggerOut {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.ok2.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("TriggerOut_{:x}", self._n);
        Verilog::Blackbox(BlackBox {
            code: format!(
                r#"
module {}
    (
    input wire [30:0] ok1,
    output wire [16:0] ok2,
    input wire        clk,
    input wire [15:0] trigger
    );

    okTriggerOut mod_trigger(.ok1(ok1),
                             .ok2(ok2),
                             .ep_addr({:x}),
                             .ep_clk(clk),
                             .ep_trigger(trigger));
endmodule

(* blackbox *)
module okTriggerOut(
    input wire [30:0]  ok1,
    output wire [16:0] ok2,
    input wire [7:0]   ep_addr,
    input wire         ep_clk,
    input wire [15:0]  ep_trigger
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
fn test_trigger_out() {
    let mut uut = rust_hdl_synth::TopWrap::new(TriggerOut::new(0x60));
    uut.uut.ok1.connect();
    uut.uut.clk.connect();
    uut.uut.trigger.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("trigout", &generate_verilog(&uut)).unwrap();
}

#[derive(Clone, Debug, LogicBlock)]
pub struct TriggerIn {
    pub ok1: Signal<In, Bits<31>>,
    pub clk: Signal<In, Clock>,
    pub trigger: Signal<Out, Bits<16>>,
    _n: u8,
}

impl TriggerIn {
    pub fn new(n: u8) -> Self {
        assert!(n >= 0x40 && n < 0x60);
        Self {
            ok1: Default::default(),
            clk: Default::default(),
            trigger: Default::default(),
            _n: n,
        }
    }
}

impl Logic for TriggerIn {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.trigger.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("TriggerIn_{:x}", self._n);
        Verilog::Blackbox(BlackBox {
            code: format!(
                r#"
module {}
    (
    input wire [30:0] ok1,
    input wire        clk,
    output wire [15:0] trigger
    );

    okTriggerIn mod_trigger(.ok1(ok1),
                            .ep_addr({:x}),
                            .ep_clk(clk),
                            .ep_trigger(trigger));
endmodule

(* blackbox *)
module okTriggerIn(
    input wire [30:0]  ok1,
    input wire [7:0]   ep_addr,
    input wire         ep_clk,
    output wire [15:0] ep_trigger
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
fn test_trigger_in() {
    let mut uut = rust_hdl_synth::TopWrap::new(TriggerIn::new(0x40));
    uut.uut.ok1.connect();
    uut.uut.clk.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("trigin", &generate_verilog(&uut)).unwrap();
}
