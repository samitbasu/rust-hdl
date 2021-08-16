use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, Default, LogicBlock)]
pub struct TriggerOut<const N: u8> {
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub clk: Signal<In, Clock>,
    pub trigger: Signal<In, Bits<16>>,
}

impl<const N: u8> Logic for TriggerOut<N> {
    fn update(&mut self) {}
    fn connect(&mut self) {
        assert!(N >= 60 && N < 0x80);
        self.ok2.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("TriggerOut_{:x}", N);
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
                VerilogLiteral::from(N)
            ),
            name,
        })
    }
}

#[test]
fn test_trigger_out() {
    use rust_hdl_synth::top_wrap;

    top_wrap!(TriggerOut<0x60>, Wrapper);
    let mut uut: Wrapper = Default::default();
    uut.uut.ok1.connect();
    uut.uut.clk.connect();
    uut.uut.trigger.connect();
    uut.connect_all();
    yosys_validate("trigout", &generate_verilog(&uut)).unwrap();
}

#[derive(Clone, Debug, Default, LogicBlock)]
pub struct TriggerIn<const N: u8> {
    pub ok1: Signal<In, Bits<31>>,
    pub clk: Signal<In, Clock>,
    pub trigger: Signal<Out, Bits<16>>,
}

impl<const N: u8> Logic for TriggerIn<N> {
    fn update(&mut self) {}
    fn connect(&mut self) {
        assert!(N >= 0x40 && N < 0x60);
        self.trigger.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("TriggerIn_{:x}", N);
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
                VerilogLiteral::from(N)
            ),
            name,
        })
    }
}

#[test]
fn test_trigger_in() {
    use rust_hdl_synth::top_wrap;

    top_wrap!(TriggerIn<0x40>, Wrapper);
    let mut uut: Wrapper = Default::default();
    uut.uut.ok1.connect();
    uut.uut.clk.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("trigin", &generate_verilog(&uut)).unwrap();
}