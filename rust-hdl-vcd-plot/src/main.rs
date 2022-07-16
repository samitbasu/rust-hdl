use anyhow::Error;
use num_bigint::{BigInt, Sign};
use std::collections::HashMap;
use std::fmt::LowerHex;
use std::fs::File;
use std::io::ErrorKind::InvalidInput;
use substring::Substring;
use svg::node::element::path::Data;
use svg::node::element::{Element, Path, SVG, Text};
use svg::{Document, Node};
use vcd::{IdCode, Value};
use rust_hdl_vcd_plot::display_metrics::DisplayMetrics;
use rust_hdl_vcd_plot::trace_collection::TraceCollection;
use rust_hdl_vcd_plot::vcd_style::VCDStyle;

// We want major_tick_delt * 10 ~= max_time
// We also want major_tick_delt = [1, 2, 5] * 10^x
// So major_tick_delt = [1, 2, 5] * 10^x * 10 = max_time
//                      [1, 2, 5] * 10^{x+1} = max_time
//                    [0, log10(2), log10(5)] + (x+1) = log10(max_time)
//                    [0, log10(2), log10(5)] = log10(max_time) - x - 1
//  Let s = log10(max_time) - 1
//  Then we have
//   [0, 0.3, 0.7] + x = s, where x is an integer
//  If we take x = floor(s)
//  Then we have
//   [0, 0.3, 0.7] = s - floor(s)
//   We choose the closest one.

/*
TODO -
1. Fix the color scheme.  Use the GTKWave color scheme instead.  Or make it configurable.
2. Make all zero vector signals draw as ____ instead of ====.  Make all 1 vector signals draw as ~~~~.
3. Add major tick lines in the background.
4. Determine why the last vector signal label is being miscalculated.
5. Add cursor lines with annotations.
 */
fn main() -> anyhow::Result<()> {
    let vcd = std::fs::File::open("test0.vcd")?;
    let traces = TraceCollection::parse(&["uut.clock", "uut.state.q", "uut.active_col.d"], vcd)?;
    let mut metrics = DisplayMetrics::default();
    metrics.style = VCDStyle::gtkwave();
    metrics.max_time = 170_000;
    let document = traces.as_svg(&metrics)?;
    svg::save("image.svg", &document).unwrap();
    Ok(())
}
