#![allow(non_camel_case_types)]

pub mod ucf_gen;
pub mod synth;

use rust_hdl_core::prelude::*;
use rust_hdl_core::constraint::Timing::Periodic;
use rust_hdl_synth::yosys_validate;
use rust_hdl_core::ast::BlackBox;
use crate::ucf_gen::generate_ucf;
use rust_hdl_widgets::pulser::Pulser;
use std::time::Duration;

make_domain!(MHz48, 48_000_000);

pub fn xem_6010_leds() -> Signal<Out, Bits<8>, Async> {
    let mut x = Signal::default();
    for (ndx, name) in [
        "Y17", "AB17", "AA14", "AB14", "AA16", "AB16", "AA10", "AB10",
    ]
        .iter()
        .enumerate()
    {
        x.add_location(ndx, name);
        x.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
    }
    x
}


#[derive(Clone, Debug, LogicInterface)]
pub struct okHostInterface {
    pub sig_in: Signal<In, Bits<8>, MHz48>,
    pub sig_out: Signal<Out, Bits<2>, MHz48>,
    pub sig_inout: Signal<InOut, Bits<16>, MHz48>,
    pub sig_aa: Signal<InOut, Bit, MHz48>,
}

impl okHostInterface {
    pub fn xem_6010() -> okHostInterface {
        let mut hi_in = Signal::default();
        for (ndx, name) in ["Y12", "AB20", "AB7", "AB8", "AA4", "AB4", "Y3", "AB3"]
            .iter()
            .enumerate() {
            hi_in.add_location(ndx, name);
            hi_in.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
        }
        hi_in.add_constraint(PinConstraint {
            index: 0,
            constraint: Constraint::Timing(
                Periodic(PeriodicTiming {
                    net: "okHostClk".into(),
                    period_nanoseconds: 20.83,
                    duty_cycle: 50.0,
                })
            )
        });
        let mut hi_out = Signal::default();
        for (ndx, name) in ["Y19", "AA8"].iter().enumerate() {
            hi_out
                .add_location(ndx, name);
            hi_out
                .add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
        }
        let mut hi_inout = Signal::default();
        for (ndx, name) in [
            "AB12", "AA12", "Y13", "AB18", "AA18", "V15", "AB2", "AA2", "Y7", "Y4", "W4", "AB6",
            "AA6", "U13", "U14", "AA20",
        ]
            .iter()
            .enumerate()
        {
            hi_inout.add_location(ndx, name);
            hi_inout.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
        }
        let mut hi_aa = Signal::default();
        hi_aa.add_location(0, "W11");
        hi_aa.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        Self {
            sig_in: hi_in,
            sig_out: hi_out,
            sig_inout: hi_inout,
            sig_aa: hi_aa,
        }
    }
}

#[derive(Clone, Debug, LogicBlock)]
pub struct OpalKellyHost {
    pub hi: okHostInterface,
    pub ok1: Signal<Out, Bits<31>, MHz48>,
    pub ok2: Signal<In, Bits<17>, MHz48>,
    pub ti_clk: Signal<Out, Clock, MHz48>,
}

impl Logic for OpalKellyHost {
    fn update(&mut self) {
    }
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
        Verilog::Blackbox(
            BlackBox {
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
           "#.into(),
                name: "OpalKellyHost".into()
            }
        )
    }
}

impl Default for OpalKellyHost {
    fn default() -> Self {
        let hi = okHostInterface::xem_6010();
        Self {
            hi,
            ok1: Signal::default(),
            ok2: Signal::default(),
            ti_clk: Signal::default(),
        }
    }
}

#[derive(LogicBlock)]
pub struct OKTest1 {
    pub hi: okHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>, Async>,
    pub pulser: Pulser<MHz48>,
}

macro_rules! link {
    ($from: expr, $to: expr) => {
    }
}

impl OKTest1 {
    pub fn new() -> Self {
        Self {
            hi: okHostInterface::xem_6010(),
            ok_host: OpalKellyHost::default(),
            led: xem_6010_leds(),
            pulser: Pulser::new(1.0, Duration::from_millis(500))
        }
    }
}

impl Logic for OKTest1 {
    #[hdl_gen]
    fn update(&mut self) {
        link!(self.hi.sig_in, self.ok_host.hi.sig_in);
        link!(self.hi.sig_inout, self.ok_host.hi.sig_inout);
        link!(self.hi.sig_out, self.ok_host.hi.sig_out);
        link!(self.hi.sig_aa, self.ok_host.hi.sig_aa);
        self.pulser.clock.next = self.ok_host.ti_clk.val();
        self.pulser.enable.next = true.into();
        if self.pulser.pulse.val().any() {
            self.led.next = 0xFF_u8.into();
        } else {
            self.led.next = 0x00_u8.into();
        }
    }
}

#[test]
fn test_ok_host_synthesizable() {
    let mut uut = OKTest1::new();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_inout.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    let ucf = generate_ucf(&uut);
    println!("{}", ucf);
    yosys_validate("vlog", &vlog).unwrap();
}