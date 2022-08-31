use crate::docs::vcd2svg::display_metrics::DisplayMetrics;
use crate::docs::vcd2svg::trace_collection::TraceCollection;
use crate::docs::vcd2svg::vcd_style::VCDStyle;

pub mod display_metrics;
mod interval;
mod renderable;
pub mod symbols;
pub mod text_frame;
mod time_view;
mod timed_value;
pub mod trace_collection;
mod utils;
pub mod vcd_style;

pub fn vcd_to_svg(
    vcd_filename: &str,
    svg_filename: &str,
    signal_names: &[&str],
    min_time_in_ps: u64,
    max_time_in_ps: u64,
) -> anyhow::Result<()> {
    let vcd = std::fs::File::open(vcd_filename)?;
    let traces = TraceCollection::parse(signal_names, vcd)?;
    let mut metrics = DisplayMetrics::default();
    metrics.style = VCDStyle::gtkwave();
    metrics.min_time = min_time_in_ps;
    metrics.max_time = max_time_in_ps;
    let document = traces.as_svg(&metrics)?;
    svg::save(svg_filename, &document)?;
    Ok(())
}

pub fn vcd_to_txt(
    vcd_filename: &str,
    signal_names: &[&str],
    min_time_in_ps: u64,
    max_time_in_ps: u64,
    max_columns: u64,
) -> anyhow::Result<String> {
    let vcd = std::fs::File::open(vcd_filename)?;
    let traces = TraceCollection::parse(signal_names, vcd)?;
    traces.as_string(min_time_in_ps, max_time_in_ps, max_columns as usize)
}

#[test]
fn test_txt() {
    let msg = vcd_to_txt(
        &format!("{}/sims/ad868x.vcd", env!("CARGO_MANIFEST_DIR")),
        &[
            "uut.clock",
            "uut.adc.spi_slave.state.q",
            "uut.master.clock",
            "uut.master.data_inbound",
            "uut.master.transfer_done",
            "uut.adc.spi_slave.bits_saved.d",
        ],
        0,
        250,
        140,
    )
    .unwrap();
    println!("{}", msg);
}
