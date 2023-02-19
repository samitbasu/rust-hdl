// Based on https://github.com/YosysHQ/icestorm/blob/master/icepll/icepll.cc
// Original license:
//
//  Copyright (C) 2015  Clifford Wolf <clifford@clifford.at>
//
//  Permission to use, copy, modify, and/or distribute this software for any
//  purpose with or without fee is hereby granted, provided that the above
//  copyright notice and this permission notice appear in all copies.
//
//  THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
//  WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
//  MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
//  ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
//  WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
//  ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
//  OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//

use rust_hdl__core::prelude::*;

#[derive(Clone, Default, Debug)]
struct ICE40PLLSettings {
    f_pllin: f64,
    fout: f64,
    divr: i32,
    divf: i32,
    divq: i32,
    simple: bool,
}

impl ICE40PLLSettings {
    fn filter_range(&self) -> usize {
        let f_pfd = self.f_pllin / (self.divr as f64 + 1.);
        let filter_range = if f_pfd < 17. {
            1
        } else if f_pfd < 26. {
            2
        } else if f_pfd < 44. {
            3
        } else if f_pfd < 66. {
            4
        } else if f_pfd < 101. {
            5
        } else {
            6
        };
        filter_range
    }
}

fn analyze(simple_feedback: bool, f_pllin: f64, f_pllout: f64) -> Option<ICE40PLLSettings> {
    let mut found_something = false;
    let mut best = ICE40PLLSettings::default();
    best.simple = simple_feedback;

    let divf_max = if simple_feedback { 127 } else { 63 };
    // The documentation in the iCE40 PLL Usage Guide incorrectly lists the
    // maximum value of DIVF as 63, when it is only limited to 63 when using
    // feedback modes other that SIMPLE.

    if f_pllin < 10. || f_pllin > 133. {
        panic!(
            "Error: PLL input frequency {} MHz is outside range 10 MHz - 133 MHz!\n",
            f_pllin
        );
    }

    if f_pllout < 16. || f_pllout > 275. {
        panic!(
            "Error: PLL output frequency {} MHz is outside range 16 MHz - 275 MHz!\n",
            f_pllout
        );
    }

    for divr in 0..=15 {
        let f_pfd = f_pllin / (divr as f64 + 1.);
        if f_pfd < 10. || f_pfd > 133. {
            continue;
        }
        for divf in 0..=divf_max {
            if simple_feedback {
                let f_vco = f_pfd * (divf as f64 + 1.);
                if f_vco < 533. || f_vco > 1066. {
                    continue;
                }
                for divq in 1..=6 {
                    let fout = f_vco * f64::exp2(-divq as f64);
                    if f64::abs(fout - f_pllout) < f64::abs(best.fout - f_pllout)
                        || !found_something
                    {
                        best.fout = fout;
                        best.divr = divr;
                        best.divf = divf;
                        best.divq = divq;
                        found_something = true;
                    }
                }
            } else {
                for divq in 1..=6 {
                    let f_vco = f_pfd * (divf as f64 + 1.) * f64::exp2(divq as f64);
                    if f_vco < 533. || f_vco > 1066. {
                        continue;
                    }
                    let fout = f_vco * f64::exp2(-divq as f64);
                    if f64::abs(fout - f_pllout) < f64::abs(best.fout - f_pllout)
                        || !found_something
                    {
                        best.fout = fout;
                        best.divr = divr;
                        best.divf = divf;
                        best.divq = divq;
                        found_something = true;
                    }
                }
            }
        }
    }
    if found_something {
        Some(best)
    } else {
        None
    }
}

#[test]
fn test_pll_gen() {
    let x = analyze(true, 100., 33.33333);
    println!("x: {:?}", x);
    assert!(x.is_some());
    let x = x.unwrap();
    assert!((x.fout - 33.3333).abs() < 1e-3);
}

#[derive(LogicBlock)]
pub struct ICE40PLLBlock<const FIN_FREQ: u64, const FOUT_FREQ: u64> {
    pub clock_in: Signal<In, Clock>,
    pub clock_out: Signal<Out, Clock>,
    pub locked: Signal<Out, Bit>,
    core: ICEPLL40Core,
    _settings: ICE40PLLSettings,
}

impl<const FIN_FREQ: u64, const FOUT_FREQ: u64> Default for ICE40PLLBlock<FIN_FREQ, FOUT_FREQ> {
    fn default() -> Self {
        let freq_in_mhz = (FIN_FREQ as f64) / (1_000_000.0);
        let freq_out_mhz = (FOUT_FREQ as f64) / (1_000_000.0);
        Self {
            clock_in: Signal::default(),
            clock_out: Signal::new_with_default(Clock::default()),
            locked: Signal::new_with_default(false),
            core: ICEPLL40Core::new(),
            _settings: analyze(true, freq_in_mhz, freq_out_mhz).unwrap(),
        }
    }
}

impl<const FIN_FREQ: u64, const FOUT_FREQ: u64> Logic for ICE40PLLBlock<FIN_FREQ, FOUT_FREQ> {
    fn update(&mut self) {}

    fn connect(&mut self) {
        self.clock_out.connect();
        self.locked.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Custom(format!(
            "\
SB_PLL40_CORE #(
                .FEEDBACK_PATH(\"{feedback}\"),
                .DIVR({DIVR}),
                .DIVF({DIVF}),
                .DIVQ({DIVQ}),
                .FILTER_RANGE({FILTER_RANGE})
               ) uut (
                .LOCK(locked),
                .RESETB(1'b1),
                .BYPASS(1'b0),
                .REFERENCECLK(clock_in),
                .PLLOUTCORE(clock_out));
",
            feedback = if self._settings.simple {
                "SIMPLE"
            } else {
                "NON_SIMPLE"
            },
            DIVR = VerilogLiteral::from(self._settings.divr as u32),
            DIVF = VerilogLiteral::from(self._settings.divf as u32),
            DIVQ = VerilogLiteral::from(self._settings.divq as u32),
            FILTER_RANGE = VerilogLiteral::from(self._settings.filter_range())
        ))
    }
}

#[derive(LogicBlock)]
pub struct ICEPLL40Core {}

impl ICEPLL40Core {
    pub fn new() -> ICEPLL40Core {
        Self {}
    }
}

impl Logic for ICEPLL40Core {
    fn update(&mut self) {}

    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(BlackBox {
            code: r#"
(* blackbox *)
module SB_PLL40_CORE (
    input   REFERENCECLK,
    output  PLLOUTCORE,
    output  PLLOUTGLOBAL,
    input   EXTFEEDBACK,
    input   [7:0] DYNAMICDELAY,
    output  LOCK,
    input   BYPASS,
    input   RESETB,
    input   LATCHINPUTVALUE,
    output  SDO,
    input   SDI,
    input   SCLK
);
parameter FEEDBACK_PATH = "SIMPLE";
parameter DELAY_ADJUSTMENT_MODE_FEEDBACK = "FIXED";
parameter DELAY_ADJUSTMENT_MODE_RELATIVE = "FIXED";
parameter SHIFTREG_DIV_MODE = 1'b0;
parameter FDA_FEEDBACK = 4'b0000;
parameter FDA_RELATIVE = 4'b0000;
parameter PLLOUT_SELECT = "GENCLK";
parameter DIVR = 4'b0000;
parameter DIVF = 7'b0000000;
parameter DIVQ = 3'b000;
parameter FILTER_RANGE = 3'b000;
parameter ENABLE_ICEGATE = 1'b0;
parameter TEST_MODE = 1'b0;
parameter EXTERNAL_DIVIDE_FACTOR = 1;
endmodule
            "#
            .into(),
            name: "SB_PLL40_CORE".into(),
        })
    }
}
