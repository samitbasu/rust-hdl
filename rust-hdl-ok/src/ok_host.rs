use rust_hdl_core::prelude::*;

use crate::bsp::XEM6010;
use crate::ok_hi::OpalKellyHostInterface;
use rust_hdl_synth::TopWrap;

#[derive(Clone, Debug, LogicBlock)]
pub struct OpalKellyHost {
    pub hi: OpalKellyHostInterface,
    pub ok1: Signal<Out, Bits<31>>,
    pub ok2: Signal<In, Bits<17>>,
    pub ti_clk: Signal<Out, Clock>,
}

impl Logic for OpalKellyHost {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.ok1.connect();
        self.ok2.connect();
        self.hi.sig_in.connect();
        self.hi.sig_out.connect();
        self.hi.sig_inout.connect();
        self.hi.sig_aa.connect();
        self.hi.sig_mux.connect();
        self.ti_clk.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(BlackBox {
            code: r#"
module OpalKellyHost
	(
	input  wire [7:0]  hi$sig_in,
	output wire [1:0]  hi$sig_out,
	inout  wire [15:0] hi$sig_inout,
	inout  wire        hi$sig_aa,
	output wire        hi$sig_mux,
	output wire        ti_clk,
	output wire [30:0] ok1,
	input  wire [16:0] ok2
	);

    assign hi$sig_mux = 1'b0;

	okHost host(.hi_in(hi$sig_in),
	            .hi_out(hi$sig_out),
	            .hi_inout(hi$sig_inout),
	            .hi_aa(hi$sig_aa),
	            .ti_clk(ti_clk),
	            .ok1(ok1),
	            .ok2(ok2));
endmodule

(* blackbox *)
module okHost(
    input  wire [7:0]  hi_in,
	output wire [1:0]  hi_out,
	inout  wire [15:0] hi_inout,
	inout  wire        hi_aa,
	output wire        ti_clk,
	output wire [30:0] ok1,
	input  wire [16:0] ok2);
endmodule
           "#
            .into(),
            name: "OpalKellyHost".into(),
        })
    }
}

impl OpalKellyHost {
    pub fn xem_6010() -> Self {
        let hi = OpalKellyHostInterface::xem_6010();
        Self {
            hi,
            ok1: Signal::default(),
            ok2: Signal::default(),
            ti_clk: Signal::default(),
        }
    }
    pub fn xem_7010() -> Self {
        let hi = OpalKellyHostInterface::xem_7010();
        Self {
            hi,
            ok1: Signal::default(),
            ok2: Signal::default(),
            ti_clk: Signal::default(),
        }
    }
}

#[test]
fn test_host_interface_synthesizes() {
    let mut uut = TopWrap::new(OpalKellyHost::xem_6010());
    uut.uut.ok2.connect();
    uut.uut.hi.sig_in.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("okhi", &generate_verilog(&uut)).unwrap();
}
