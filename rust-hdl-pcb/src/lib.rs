use crate::circuit::Circuit;
use crate::designator::Designator;
use crate::resistors::{PowerWatt, ResistorTolerance, ResistorTempco};
use crate::smd::SizeCode;
use crate::yageo_rc_rl_at_series::make_yageo_series_resistor;
use crate::tdk_cga_series::make_tdk_cga_capacitor;
use crate::capacitors::{CapacitorKind, DielectricCode, CapacitorTolerance};
use crate::kemet_t491_series::make_kemet_t491_capacitor;
use crate::avx_caps::make_avx_capacitor;

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
mod yageo_rc_rl_at_series;
mod kemet_t491_series;
mod avx_caps;

#[test]
fn test_yageo_rc_68k() {
    let led_limit = make_yageo_series_resistor("RC0603FR-0768KL");
    println!("{:?}", led_limit);
    assert_eq!(led_limit.value_ohms, 68e3);
    assert_eq!(led_limit.details.size, SizeCode::I0603);
    assert!(led_limit.power_watt >= PowerWatt::new(1, 10));
}

#[test]
fn test_yageo_rc_1k() {
    let current_limit = make_yageo_series_resistor("RC1206FR-071KL");
    assert_eq!(current_limit.value_ohms, 1.0e3);
    assert_eq!(current_limit.details.size, SizeCode::I1206);
    assert!(current_limit.power_watt >= PowerWatt::new(1, 4));
    assert_eq!(current_limit.details.pins.len(), 2);
}

#[test]
fn test_tdk_cga_cap() {
    let filter_cap = make_tdk_cga_capacitor("CGA4J2X7R2A104K125AA");
    // 'TDK         CGA4J2X7R2A104K125AA             SMD Multilayer Ceramic Capacitor, 0805 [2012 Metric], 0.1 F, 100 V,  10%, X7R, CGA Series
    assert_eq!(filter_cap.details.manufacturer.name, "TDK");
    assert_eq!(filter_cap.kind, CapacitorKind::MultiLayerChip(DielectricCode::X7R));
    assert_eq!(filter_cap.details.size, SizeCode::I0805);
    assert_eq!(filter_cap.value_pf, 0.1 * 1e6);
    assert_eq!(filter_cap.voltage, 100.);
    println!("{:?}", filter_cap);
}

#[test]
fn test_kemet_tantalum_cap() {
    let kemet = make_kemet_t491_capacitor("T491A106K010AT");
    assert_eq!(kemet.details.size, SizeCode::I1206);
    assert_eq!(kemet.kind, CapacitorKind::Tantalum);
    assert_eq!(kemet.value_pf, 10. * 1e6);
    assert_eq!(kemet.voltage, 10.);
    assert_eq!(kemet.tolerance, CapacitorTolerance::TenPercent);
    println!("{:#?}", kemet);
}

#[test]
fn test_yageo_precision() {
    let precise = make_yageo_series_resistor("RL0603FR-070R47L");
    // 'Res Thick Film 0603 0.47 Ohm 1% 0.1W(1/10W) ±800ppm/C Molded SMD Paper T/R
    assert_eq!(precise.details.size, SizeCode::I0603);
    assert_eq!(precise.tolerance, ResistorTolerance::OnePercent);
    assert_eq!(precise.value_ohms, 0.47);
}

#[test]
fn test_yageo_bulk() {
    let bulk = make_yageo_series_resistor("RC1206FR-071KL");
    // 'YAGEO - RC1206FR-071KL. - RES, THICK FILM, 1K, 1%, 0.25W, 1206, REEL
    assert_eq!(bulk.tolerance, ResistorTolerance::OnePercent);
    assert_eq!(bulk.power_watt, PowerWatt::new(1, 4));
    assert_eq!(bulk.details.size, SizeCode::I1206);
    assert_eq!(bulk.value_ohms, 1.0e3);
}

#[test]
fn test_yageo_tempco() {
    // T491A106K010AT
    let temp = make_yageo_series_resistor("AT0603BRD0720KL");
    // '20K 0.1% precision
    assert_eq!(temp.tolerance, ResistorTolerance::TenthPercent);
    assert_eq!(temp.value_ohms, 20e3);
    assert_eq!(temp.tempco, Some(ResistorTempco::Ppm25degC));
}

#[test]
fn test_avx() {
    let avx = make_avx_capacitor("22201C475KAT2A");
    assert_eq!(avx.details.size, SizeCode::I2220);
    assert_eq!(avx.kind, CapacitorKind::MultiLayerChip(DielectricCode::X7R));
    assert_eq!(avx.value_pf, 47e5);
    assert_eq!(avx.tolerance, CapacitorTolerance::TenPercent);
}
