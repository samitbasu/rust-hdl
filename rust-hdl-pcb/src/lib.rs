use crate::circuit::Circuit;
use crate::designator::Designator;
use crate::resistors::{PowerMilliWatt, ResistorTolerance};
use crate::smd::SizeCode;
use crate::yageo_rc_l_series::make_yageo_rc_l_resistor;
use crate::tdk_cga_series::make_tdk_cga_capacitor;
use crate::capacitors::{CapacitorKind, WorkingVoltage, DielectricCode};

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
mod kemet_t491_series;

#[test]
fn make_circuit() {
    // LED current limiter
    let led_limit = make_yageo_rc_l_resistor("RC0603FR-0768KL");
    println!("{:?}", led_limit);
    assert_eq!(led_limit.value_ohms, 68e3);
    assert_eq!(led_limit.details.size, SizeCode::I0603);
    assert!(led_limit.power.as_value() >= 100.0);
    let current_limit = make_yageo_rc_l_resistor("RC1206FR-071KL");
    assert_eq!(current_limit.value_ohms, 1.0e3);
    assert_eq!(current_limit.details.size, SizeCode::I1206);
    assert!(current_limit.power.as_value() >= 250.0);
    assert_eq!(current_limit.details.pins.len(), 2);
    let filter_cap = make_tdk_cga_capacitor("CGA4J2X7R2A104K125AA");
    // 'TDK         CGA4J2X7R2A104K125AA             SMD Multilayer Ceramic Capacitor, 0805 [2012 Metric], 0.1 F, 100 V,  10%, X7R, CGA Series
    assert_eq!(filter_cap.details.manufacturer.name, "TDK");
    assert_eq!(filter_cap.kind, CapacitorKind::MultiLayerChip);
    assert_eq!(filter_cap.details.size, SizeCode::I0805);
    assert_eq!(filter_cap.value_pf, 0.1 * 1e6);
    assert_eq!(filter_cap.voltage, WorkingVoltage::V100);
    assert_eq!(filter_cap.dielectric, DielectricCode::X7R);
    println!("{:?}", filter_cap);
}
