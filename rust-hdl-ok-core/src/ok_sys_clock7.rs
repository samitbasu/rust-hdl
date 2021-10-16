use rust_hdl_core::prelude::*;
use rust_hdl_synth::{yosys_validate, TopWrap};

#[derive(LogicBlock, Default)]
pub struct OpalKellySystemClock7 {
    pub clock_p: Signal<In, Clock>,
    pub clock_n: Signal<In, Clock>,
    pub sys_clock: Signal<Out, Clock>,
}

impl Logic for OpalKellySystemClock7 {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.sys_clock.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(BlackBox {
            code: r#"
module opal_kelly_system_clock_7 (
            clock_p,
            clock_n,
            sys_clock
            );

input wire clock_p;
input wire clock_n;
output wire sys_clock;

wire clock_single_ended;

// Buffer the input
IBUFDS clk_ibufgds
    (.O  (clock_single_ended),
     .I  (clock_p),
     .IB (clock_n));


  // Clocking PRIMITIVE
  //------------------------------------

  // Instantiation of the MMCM PRIMITIVE
  //    * Unused inputs are tied off
  //    * Unused outputs are labeled unused

  wire        clk_out1_cdiv;

  wire        locked_int;
  wire        clkfbout_cdiv;
  wire        clkfbout_buf_cdiv;

  MMCME2_BASE
  #(.CLKFBOUT_MULT_F      (5.000),
    .CLKOUT0_DIVIDE_F     (10.000),
    .CLKIN1_PERIOD        (5.000))
  mmcm_base_inst
    // Output clocks
   (
    .CLKFBOUT            (clkfbout_cdiv),
    .CLKOUT0             (clk_out1_cdiv),
     // Input clock control
    .CLKFBIN             (clkfbout_buf_cdiv),
    .CLKIN1              (clock_single_ended),
    // Other control and status signals
    .LOCKED              (locked_int),
    .PWRDWN              (1'b0),
    .RST                 (1'b0));

// Clock Monitor clock assigning
//--------------------------------------
 // Output buffering
  //-----------------------------------

  BUFG clkf_buf
   (.O (clkfbout_buf_cdiv),
    .I (clkfbout_cdiv));

  BUFG clkout1_buf
   (.O   (sys_clock),
    .I   (clk_out1_cdiv));
endmodule

(* blackbox *)
module BUFG(I, O);
  input wire I;
  output wire O;
endmodule

(* blackbox *)
module IBUFDS(I, IB, O);
  input wire I;
  input wire IB;
  output wire O;
endmodule

(* blackbox *)
module MMCME2_BASE (
  CLKFBOUT,
  CLKFBOUTB,
  CLKOUT0,
  CLKOUT0B,
  CLKOUT1,
  CLKOUT1B,
  CLKOUT2,
  CLKOUT2B,
  CLKOUT3,
  CLKOUT3B,
  CLKOUT4,
  CLKOUT5,
  CLKOUT6,
  LOCKED,
  CLKFBIN,
  CLKIN1,
  PWRDWN,
  RST
);
  parameter BANDWIDTH = "OPTIMIZED";
  parameter real CLKFBOUT_MULT_F = 5.000;
  parameter real CLKFBOUT_PHASE = 0.000;
  parameter real CLKIN1_PERIOD = 0.000;
  parameter real CLKOUT0_DIVIDE_F = 1.000;
  parameter real CLKOUT0_DUTY_CYCLE = 0.500;
  parameter real CLKOUT0_PHASE = 0.000;
  parameter integer CLKOUT1_DIVIDE = 1;
  parameter real CLKOUT1_DUTY_CYCLE = 0.500;
  parameter real CLKOUT1_PHASE = 0.000;
  parameter integer CLKOUT2_DIVIDE = 1;
  parameter real CLKOUT2_DUTY_CYCLE = 0.500;
  parameter real CLKOUT2_PHASE = 0.000;
  parameter integer CLKOUT3_DIVIDE = 1;
  parameter real CLKOUT3_DUTY_CYCLE = 0.500;
  parameter real CLKOUT3_PHASE = 0.000;
  parameter CLKOUT4_CASCADE = "FALSE";
  parameter integer CLKOUT4_DIVIDE = 1;
  parameter real CLKOUT4_DUTY_CYCLE = 0.500;
  parameter real CLKOUT4_PHASE = 0.000;
  parameter integer CLKOUT5_DIVIDE = 1;
  parameter real CLKOUT5_DUTY_CYCLE = 0.500;
  parameter real CLKOUT5_PHASE = 0.000;
  parameter integer CLKOUT6_DIVIDE = 1;
  parameter real CLKOUT6_DUTY_CYCLE = 0.500;
  parameter real CLKOUT6_PHASE = 0.000;
  parameter integer DIVCLK_DIVIDE = 1;
  parameter real REF_JITTER1 = 0.010;
  parameter STARTUP_WAIT = "FALSE";
   output CLKFBOUT;
   output CLKFBOUTB;
   output CLKOUT0;
   output CLKOUT0B;
   output CLKOUT1;
   output CLKOUT1B;
   output CLKOUT2;
   output CLKOUT2B;
   output CLKOUT3;
   output CLKOUT3B;
   output CLKOUT4;
   output CLKOUT5;
   output CLKOUT6;
   output LOCKED;
   input CLKFBIN;
   input CLKIN1;
   input PWRDWN;
   input RST;
endmodule

        "#
            .into(),
            name: "opal_kelly_system_clock_7".to_string(),
        })
    }
}

#[test]
fn test_synth() {
    let mut uut = TopWrap::new(OpalKellySystemClock7::default());
    uut.uut.clock_n.connect();
    uut.uut.clock_p.connect();
    uut.connect_all();
    println!("vlog: {}", generate_verilog(&uut));
    yosys_validate("ok_sys_clock7", &generate_verilog(&uut)).unwrap();
}
