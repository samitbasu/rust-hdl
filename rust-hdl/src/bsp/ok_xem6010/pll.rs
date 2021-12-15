use crate::core::prelude::*;

pub struct Spartan6PLLSettings {
    pub clkin_period_ns: f64,
    pub pll_mult: i32,
    pub pll_div: i32,
    pub output_divs: [u8; 6],
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PLLSettingsValidation {
    InvalidClockInPeriod,
    PLLMultiplierOutOfRange,
    PLLDividerOutOfRange,
    VCOOutOfRange(f64),
    OutputClockOutOfRange(u8),
    Ok,
}

impl Spartan6PLLSettings {
    pub fn validate(&self) -> PLLSettingsValidation {
        // The clock period must be valid
        if self.clkin_period_ns < 1.408 || self.clkin_period_ns > 52.630 {
            return PLLSettingsValidation::InvalidClockInPeriod;
        }
        if self.pll_mult < 1 || self.pll_mult > 64 {
            return PLLSettingsValidation::PLLMultiplierOutOfRange;
        }
        if self.pll_div < 1 || self.pll_div > 52 {
            return PLLSettingsValidation::PLLDividerOutOfRange;
        }
        // These limits are conservative.  Your specific part
        // may allow higher VCO frequencies.
        let freq_in = 1.0e9 / self.clkin_period_ns;
        let vco_freq = (freq_in * (self.pll_mult as f64) / (self.pll_div as f64)) / 1e6;
        if vco_freq < 400.0 || vco_freq > 1000.0 {
            return PLLSettingsValidation::VCOOutOfRange(vco_freq);
        }
        // Ensure each output is valid
        for factor in self.output_divs {
            let out_freq = vco_freq / factor as f64;
            if out_freq < 19.0 || out_freq > 400.0 {
                return PLLSettingsValidation::OutputClockOutOfRange(factor);
            }
        }
        PLLSettingsValidation::Ok
    }
}

#[derive(LogicBlock)]
pub struct PLLFreqSynthesis {
    pub clock_in: Signal<In, Clock>,
    pub clock_out0: Signal<Out, Clock>,
    pub clock_out1: Signal<Out, Clock>,
    pub clock_out2: Signal<Out, Clock>,
    pub clock_out3: Signal<Out, Clock>,
    pub clock_out4: Signal<Out, Clock>,
    pub clock_out5: Signal<Out, Clock>,
    pub locked: Signal<Out, Bit>,
    pub reset: Signal<In, Bit>,
    _settings: Spartan6PLLSettings,
}

impl PLLFreqSynthesis {
    pub fn new(settings: Spartan6PLLSettings) -> Self {
        let r = settings.validate();
        match r {
            PLLSettingsValidation::Ok => {}
            _ => panic!("PLL Settings are invalid {:?}", r),
        }
        Self {
            clock_in: Default::default(),
            clock_out0: Default::default(),
            clock_out1: Default::default(),
            clock_out2: Default::default(),
            clock_out3: Default::default(),
            clock_out4: Default::default(),
            clock_out5: Default::default(),
            locked: Default::default(),
            reset: Default::default(),
            _settings: settings,
        }
    }
}

impl Logic for PLLFreqSynthesis {
    fn update(&mut self) {}

    fn connect(&mut self) {
        self.clock_out0.connect();
        self.clock_out1.connect();
        self.clock_out2.connect();
        self.clock_out3.connect();
        self.clock_out4.connect();
        self.clock_out5.connect();
        self.locked.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: format!(
                r#"

wire clock_feedback;

PLL_ADV #(
      .BANDWIDTH		("OPTIMIZED"),  		// "high", "low" or "optimized"
      .CLKFBOUT_MULT		({PLLX}),       	// multiplication factor for all output clocks
      .CLKFBOUT_PHASE		(0.0),     			// phase shift (degrees) of all output clocks
      .CLKIN1_PERIOD		({CLKIN_PERIOD}),  	// clock period (ns) of input clock on clkin1
      .CLKIN2_PERIOD		({CLKIN_PERIOD}),  	// clock period (ns) of input clock on clkin2
      .CLKOUT0_DIVIDE		({CLK0_DIV}),       // division factor for clkout0 (1 to 128)
      .CLKOUT0_DUTY_CYCLE	(0.5), 				// duty cycle for clkout0 (0.01 to 0.99)
      .CLKOUT0_PHASE		(0.0), 				// phase shift (degrees) for clkout0 (0.0 to 360.0)
      .CLKOUT1_DIVIDE		({CLK1_DIV}),   	// division factor for clkout1 (1 to 128)
      .CLKOUT1_DUTY_CYCLE	(0.5), 				// duty cycle for clkout1 (0.01 to 0.99)
      .CLKOUT1_PHASE		(0.0), 				// phase shift (degrees) for clkout1 (0.0 to 360.0)
      .CLKOUT2_DIVIDE		({CLK2_DIV}),   	// division factor for clkout2 (1 to 128)
      .CLKOUT2_DUTY_CYCLE	(0.5), 				// duty cycle for clkout2 (0.01 to 0.99)
      .CLKOUT2_PHASE		(0.0), 				// phase shift (degrees) for clkout2 (0.0 to 360.0)
      .CLKOUT3_DIVIDE		({CLK3_DIV}),   	// division factor for clkout3 (1 to 128)
      .CLKOUT3_DUTY_CYCLE	(0.5), 				// duty cycle for clkout3 (0.01 to 0.99)
      .CLKOUT3_PHASE		(0.0), 				// phase shift (degrees) for clkout3 (0.0 to 360.0)
      .CLKOUT4_DIVIDE		({CLK4_DIV}),   	// division factor for clkout4 (1 to 128)
      .CLKOUT4_DUTY_CYCLE	(0.5), 				// duty cycle for clkout4 (0.01 to 0.99)
      .CLKOUT4_PHASE		(0.0),      		// phase shift (degrees) for clkout4 (0.0 to 360.0)
      .CLKOUT5_DIVIDE		({CLK5_DIV}),       // division factor for clkout5 (1 to 128)
      .CLKOUT5_DUTY_CYCLE	(0.5), 				// duty cycle for clkout5 (0.01 to 0.99)
      .CLKOUT5_PHASE		(0.0),      		// phase shift (degrees) for clkout5 (0.0 to 360.0)
      .COMPENSATION		("SYSTEM_SYNCHRONOUS"),	// "SYSTEM_SYNCHRONOUS", "SOURCE_SYNCHRONOUS", "INTERNAL", "EXTERNAL", "DCM2PLL", "PLL2DCM"
      .DIVCLK_DIVIDE		({PLLD}),        	// division factor for all clocks (1 to 52)
      .CLK_FEEDBACK		("CLKFBOUT"),       	//
      .REF_JITTER		(0.100))        		// input reference jitter (0.000 to 0.999 ui%)
pll_adv_inst (
      .CLKFBDCM			(),              		// output feedback signal used when pll feeds a dcm
      .CLKFBOUT			(clock_feedback), 		// general output feedback signal
      .CLKOUT0			(clock_out0),      		// one of six general clock output signals
      .CLKOUT1			(clock_out1),      		// one of six general clock output signals
      .CLKOUT2			(clock_out2), 		    // one of six general clock output signals
      .CLKOUT3			(clock_out3),           // one of six general clock output signals
      .CLKOUT4			(clock_out4),           // one of six general clock output signals
      .CLKOUT5			(clock_out5),           // one of six general clock output signals
      .CLKOUTDCM0		(),            			// one of six clock outputs to connect to the dcm
      .CLKOUTDCM1		(),            			// one of six clock outputs to connect to the dcm
      .CLKOUTDCM2		(),            			// one of six clock outputs to connect to the dcm
      .CLKOUTDCM3		(),            			// one of six clock outputs to connect to the dcm
      .CLKOUTDCM4		(),            			// one of six clock outputs to connect to the dcm
      .CLKOUTDCM5		(),            			// one of six clock outputs to connect to the dcm
      .DO			    (),                		// dynamic reconfig data output (16-bits)
      .DRDY			    (),                		// dynamic reconfig ready output
      .LOCKED			(locked),        		// active high pll lock signal
      .CLKFBIN			(clock_feedback),		// clock feedback input
      .CLKIN1			(clock_in),     		// primary clock input
      .CLKIN2			(1'b0),		     		// secondary clock input
      .CLKINSEL			(1'b1),             	// selects '1' = clkin1, '0' = clkin2
      .DADDR			(5'b00000),            	// dynamic reconfig address input (5-bits)
      .DCLK			(1'b0),               		// dynamic reconfig clock input
      .DEN			(1'b0),                		// dynamic reconfig enable input
      .DI			(16'h0000),        		    // dynamic reconfig data input (16-bits)
      .DWE			(1'b0),                		// dynamic reconfig write enable input
      .RST			(reset),               		// asynchronous pll reset
      .REL			(1'b0)) ;    			    // used to force the state of the PFD outputs (test only)"#,
                PLLX = self._settings.pll_mult,
                CLKIN_PERIOD = self._settings.clkin_period_ns,
                CLK0_DIV = self._settings.output_divs[0],
                CLK1_DIV = self._settings.output_divs[1],
                CLK2_DIV = self._settings.output_divs[2],
                CLK3_DIV = self._settings.output_divs[3],
                CLK4_DIV = self._settings.output_divs[4],
                CLK5_DIV = self._settings.output_divs[5],
                PLLD = self._settings.pll_div
            ),
            cores: r#"
(* blackbox *)
module PLL_ADV (
        CLKFBDCM,
        CLKFBOUT,
        CLKOUT0,
        CLKOUT1,
        CLKOUT2,
        CLKOUT3,
        CLKOUT4,
        CLKOUT5,
        CLKOUTDCM0,
        CLKOUTDCM1,
        CLKOUTDCM2,
        CLKOUTDCM3,
        CLKOUTDCM4,
        CLKOUTDCM5,
        DO,
        DRDY,
        LOCKED,
        CLKFBIN,
        CLKIN1,
        CLKIN2,
        CLKINSEL,
        DADDR,
        DCLK,
        DEN,
        DI,
        DWE,
        REL,
        RST
);

parameter BANDWIDTH = "OPTIMIZED";
parameter CLK_FEEDBACK = "CLKFBOUT";
parameter CLKFBOUT_DESKEW_ADJUST = "NONE";
parameter CLKOUT0_DESKEW_ADJUST = "NONE";
parameter CLKOUT1_DESKEW_ADJUST = "NONE";
parameter CLKOUT2_DESKEW_ADJUST = "NONE";
parameter CLKOUT3_DESKEW_ADJUST = "NONE";
parameter CLKOUT4_DESKEW_ADJUST = "NONE";
parameter CLKOUT5_DESKEW_ADJUST = "NONE";
parameter integer CLKFBOUT_MULT = 1;
parameter real CLKFBOUT_PHASE = 0.0;
parameter real CLKIN1_PERIOD = 0.000;
parameter real CLKIN2_PERIOD = 0.000;
parameter integer CLKOUT0_DIVIDE = 1;
parameter real CLKOUT0_DUTY_CYCLE = 0.5;
parameter real CLKOUT0_PHASE = 0.0;
parameter integer CLKOUT1_DIVIDE = 1;
parameter real CLKOUT1_DUTY_CYCLE = 0.5;
parameter real CLKOUT1_PHASE = 0.0;
parameter integer CLKOUT2_DIVIDE = 1;
parameter real CLKOUT2_DUTY_CYCLE = 0.5;
parameter real CLKOUT2_PHASE = 0.0;
parameter integer CLKOUT3_DIVIDE = 1;
parameter real CLKOUT3_DUTY_CYCLE = 0.5;
parameter real CLKOUT3_PHASE = 0.0;
parameter integer CLKOUT4_DIVIDE = 1;
parameter real CLKOUT4_DUTY_CYCLE = 0.5;
parameter real CLKOUT4_PHASE = 0.0;
parameter integer CLKOUT5_DIVIDE = 1;
parameter real CLKOUT5_DUTY_CYCLE = 0.5;
parameter real CLKOUT5_PHASE = 0.0;
parameter COMPENSATION = "SYSTEM_SYNCHRONOUS";
parameter integer DIVCLK_DIVIDE = 1;
parameter EN_REL = "FALSE";
parameter PLL_PMCD_MODE = "FALSE";
parameter real REF_JITTER = 0.100;
parameter RESET_ON_LOSS_OF_LOCK = "FALSE";
parameter RST_DEASSERT_CLK = "CLKIN1";
parameter SIM_DEVICE = "VIRTEX5";

localparam real VCOCLK_FREQ_MAX = 1440.0;
localparam real VCOCLK_FREQ_MIN = 400.0;
localparam real CLKIN_FREQ_MAX = 710.0;
localparam real CLKIN_FREQ_MIN = 19.0;
localparam real CLKPFD_FREQ_MAX = 550.0;
localparam real CLKPFD_FREQ_MIN = 19.0;

output CLKFBDCM;
output CLKFBOUT;
output CLKOUT0;
output CLKOUT1;
output CLKOUT2;
output CLKOUT3;
output CLKOUT4;
output CLKOUT5;
output CLKOUTDCM0;
output CLKOUTDCM1;
output CLKOUTDCM2;
output CLKOUTDCM3;
output CLKOUTDCM4;
output CLKOUTDCM5;
output DRDY;
output LOCKED;
output [15:0] DO;

input CLKFBIN;
input CLKIN1;
input CLKIN2;
input CLKINSEL;
input DCLK;
input DEN;
input DWE;
input REL;
input RST;
input [15:0] DI;
input [4:0] DADDR;

endmodule
                "#
            .into(),
        })
    }
}

#[test]
fn test_pll_gen() {
    let mut uut = TopWrap::new(PLLFreqSynthesis::new(Spartan6PLLSettings {
        clkin_period_ns: 10.0,
        pll_mult: 12,
        pll_div: 3,
        output_divs: [1, 7, 7, 7, 7, 7],
    }));
    uut.uut.clock_in.connect();
    uut.uut.reset.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("pll", &vlog).unwrap();
}
