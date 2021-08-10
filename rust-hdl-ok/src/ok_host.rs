use rust_hdl_core::prelude::*;

use crate::ok_hi::OpalKellyHostInterface;
use crate::MHz48;

#[derive(Clone, Debug, LogicBlock)]
pub struct OpalKellyHost {
    pub hi: OpalKellyHostInterface,
    pub ok1: Signal<Out, Bits<31>, MHz48>,
    pub ok2: Signal<In, Bits<17>, MHz48>,
    pub ti_clk: Signal<Out, Clock, MHz48>,
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
        self.ti_clk.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(BlackBox {
            code: r#"
module OpalKellyHost
	(
	input  wire [7:0]  hi_sig_in,
	output wire [1:0]  hi_sig_out,
	inout  wire [15:0] hi_sig_inout,
	inout  wire        hi_sig_aa,
	output wire        ti_clk,
	output wire [30:0] ok1,
	input  wire [16:0] ok2
	);

	okHost host(.hi_in(hi_sig_in),
	            .hi_out(hi_sig_out),
	            .hi_inout(hi_sig_inout),
	            .hi_aa(hi_sig_aa),
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

impl Default for OpalKellyHost {
    fn default() -> Self {
        let hi = OpalKellyHostInterface::xem_6010();
        Self {
            hi,
            ok1: Signal::default(),
            ok2: Signal::default(),
            ti_clk: Signal::default(),
        }
    }
}
