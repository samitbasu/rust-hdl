use crate::circuit::Circuit;
use crate::designator::Designator;
use crate::resistors::{PowerWatt, ResistanceValues, Tolerance};
use crate::smd::SizeCode;

mod bom;
mod capacitors;
mod circuit;
mod designator;
mod digikey_table;
mod epin;
mod inductors;
mod murata_grt_188r61h_series;
mod resistors;
mod smd;
mod tdk_cga_series;
mod traco_power_tmr1_series;
mod yageo_rc_l_series;

#[test]
fn make_circuit() {
    // LED current limiter
    let led_current_limiter = yageo_rc_l_series::make_yageo_rc_l_series_part(
        SizeCode::I0603,
        Tolerance::OnePercent,
        ResistanceValues::Ohm68K,
        Some(PowerWatt::Sixteenth),
    );
    let current_limit_resistor = yageo_rc_l_series::make_yageo_rc_l_series_part(
        SizeCode::I1206,
        Tolerance::OnePercent,
        ResistanceValues::Ohm1K,
        Some(PowerWatt::Quarter),
    );
    println!("{:?}", led_current_limiter);
}
