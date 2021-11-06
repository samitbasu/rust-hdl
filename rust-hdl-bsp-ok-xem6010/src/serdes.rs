use rust_hdl_core::prelude::*;
use rust_hdl_core::ast::Wrapper;
use rust_hdl_yosys_synth::TopWrap;

#[derive(LogicBlock)]
pub struct ClockSplitter<const DIVIDE: usize, const DOUBLED: bool> {
    pub clock_in: Signal<In, Clock>,
    pub serdes_strobe: Signal<Out, Bit>,
    pub clock_out: Signal<Out, Clock>,
    pub div_clock_out: Signal<Out, Clock>,
}

impl<const DIVIDE: usize, const DOUBLED: bool> Default for ClockSplitter<DIVIDE, DOUBLED> {
    fn default() -> Self {
        assert!([1, 3, 4, 5, 6, 7, 8].contains(&DIVIDE));
        Self {
            clock_in: Default::default(),
            serdes_strobe: Default::default(),
            clock_out: Default::default(),
            div_clock_out: Default::default()
        }
    }
}


impl<const DIVIDE: usize, const DOUBLED: bool> Logic for ClockSplitter<DIVIDE, DOUBLED> {
    fn update(&mut self) {}

    fn connect(&mut self) {
        self.serdes_strobe.connect();
        self.div_clock_out.connect();
        self.clock_out.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(
            BlackBox {
                code: format!(r#"
module clock_splitter(clock_in, serdes_strobe, clock_out, div_clock_out);

input wire clock_in;
output wire serdes_strobe;
output wire clock_out;
output wire div_clock_out;

wire div_clock;
wire buf_clock_in;

/*IBUFG ibufg_inst(.I(clock_in), .O(buf_clock_in));*/

BUFIO2 #(
          .DIVIDE({divide}),
          .I_INVERT("FALSE"),
          .DIVIDE_BYPASS("FALSE"),
          .USE_DOUBLER("{double}"))
bufio2_inst (
          .I(clock_in),
          .IOCLK(clock_out),
          .SERDESSTROBE(serdes_strobe),
          .DIVCLK(div_clock));

BUFG bufg_inst(.I(div_clock), .O(div_clock_out));

endmodule

(* blackbox *)
module BUFG(input I, output O);
endmodule

(* blackbox *)
module IBUFG(input I, output O);
endmodule

(* blackbox *)
module BUFIO2 (
   input I,
   output IOCLK,
   output DIVCLK,
   output SERDESSTROBE);
parameter DIVIDE = 1;
parameter DIVIDE_BYPASS = "TRUE";
parameter I_INVERT = "FALSE";
parameter USE_DOUBLER = "FALSE";
endmodule
"#,
                    divide = DIVIDE,
                    double = if DOUBLED {
                        "TRUE"
                    } else {
                        "FALSE"
                    }
                ),
                name: "clock_splitter".to_string()
            })
    }
}

#[test]
fn test_iobuf2_gen() {
    let mut uut : TopWrap<ClockSplitter<4, false>> = TopWrap::new(ClockSplitter::default());
    uut.uut.clock_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    rust_hdl_yosys_synth::yosys_validate("iobuf2", &vlog).unwrap();
}


#[derive(LogicBlock)]
pub struct DCMClockDoubler {
    pub clock_in: Signal<In, Clock>,
    pub clock_out: Signal<Out, Clock>,
    pub locked: Signal<Out, Bit>,
    _period_in_nsec: f64,
}

impl DCMClockDoubler {
    pub fn new(period_in_nsec: f64) -> Self {
        Self {
            clock_in: Default::default(),
            clock_out: Default::default(),
            locked: Default::default(),
            _period_in_nsec: period_in_nsec,
        }
    }
}


impl Logic for DCMClockDoubler {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.clock_out.connect();
        self.locked.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(
            BlackBox{
                code: format!(r#"
module clock_doubler(clock_in, clock_out, locked);

input wire clock_in;
output wire clock_out;
output wire locked;

wire clock_2x;

assign clock_out = clock_2x;

DCM_SP #(
         .STARTUP_WAIT("TRUE"),
         .CLK_FEEDBACK("2X"),
         .CLKIN_PERIOD({clock_in_period})
         )
dcm_sp_inst (.CLKIN(clock_in),
             .CLK2X(clock_2x),
             .CLKFB(clock_2x),
             .RST(1'b0),
             .PSEN(1'b0),
             .LOCKED(locked));
endmodule

(* blackbox *)
module DCM_SP (
	CLK0, CLK180, CLK270, CLK2X, CLK2X180, CLK90,
	CLKDV, CLKFX, CLKFX180, LOCKED, PSDONE, STATUS,
	CLKFB, CLKIN, DSSEN, PSCLK, PSEN, PSINCDEC, RST);

input CLKFB, CLKIN, DSSEN;
input PSCLK, PSEN, PSINCDEC, RST;

output CLK0, CLK180, CLK270, CLK2X, CLK2X180, CLK90;
output CLKDV, CLKFX, CLKFX180, LOCKED, PSDONE;
output [7:0] STATUS;

parameter real CLKDV_DIVIDE = 2.0;
parameter integer CLKFX_DIVIDE = 1;
parameter integer CLKFX_MULTIPLY = 4;
parameter CLKIN_DIVIDE_BY_2 = "FALSE";
parameter real CLKIN_PERIOD = 10.0;			// non-simulatable
parameter CLKOUT_PHASE_SHIFT = "NONE";
parameter CLK_FEEDBACK = "1X";
parameter DESKEW_ADJUST = "SYSTEM_SYNCHRONOUS";	// non-simulatable
parameter DFS_FREQUENCY_MODE = "LOW";
parameter DLL_FREQUENCY_MODE = "LOW";
parameter DSS_MODE = "NONE";			// non-simulatable
parameter DUTY_CYCLE_CORRECTION = "TRUE";
parameter FACTORY_JF = 16'hC080;		// non-simulatable
parameter integer PHASE_SHIFT = 0;
parameter STARTUP_WAIT = "FALSE";		// non-simulatable
endmodule

                "#, clock_in_period = self._period_in_nsec),
                name: "clock_doubler".into()
            }
        )
    }
}

#[test]
fn test_doubler_gen() {
    let mut uut = TopWrap::new(DCMClockDoubler::new(10.0));
    uut.uut.clock_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    rust_hdl_yosys_synth::yosys_validate("doubler", &vlog).unwrap();
}




/*

#[derive(LogicBlock)]
pub struct OutputSerDes4 {
    pub data_in: Signal<In, Bits<4>>,
    pub data_enable: Signal<In, Bit>,
    pub data_clock: Signal<In, Clock>,
    pub io_clock:
}

 */