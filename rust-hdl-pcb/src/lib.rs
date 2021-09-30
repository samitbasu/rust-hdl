use rust_hdl_pcb_core::capacitors::{CapacitorKind, CapacitorTolerance, DielectricCode};
use rust_hdl_pcb_core::circuit::{
    instance, Capacitor, CircuitNode, LogicFunction, LogicSignalStandard, PartInstance,
};
use rust_hdl_pcb_core::diode::DiodeKind;
use rust_hdl_pcb_core::epin::PinKind;
use rust_hdl_pcb_core::inductors::make_ty_brl_series;
use rust_hdl_pcb_core::resistors::{PowerWatt, ResistorKind};
use rust_hdl_pcb_core::smd::SizeCode;
use rust_hdl_pcb_svg::schematic::make_svgs;

use crate::adc::make_ads868x;
use crate::analog_devices::make_lt3092_current_source;
use crate::avx_caps::make_avx_capacitor;
use crate::connectors::{
    make_amphenol_10056845_header, make_molex_55935_connector, make_sullins_sbh11_header,
};
use crate::isolators::make_iso7741edwrq1;
use crate::kemet_ceramic_caps::make_kemet_ceramic_capacitor;
use crate::kemet_t491_series::make_kemet_t491_capacitor;
use crate::ldo::{
    make_mcp_1799_regulator, make_on_semi_ncv33375_regulator, make_ti_tps_7b84_regulator,
    make_zldo1117g_regulator,
};
use crate::lvc_one_gate::make_lvc_one_gate;
use crate::murata_mlcc_caps::make_murata_capacitor;
use crate::nippon_electrolytic_caps::make_nippon_hxd_capacitor;
use crate::panasonic_era_resistors::make_panasonic_resistor;
use crate::sn74_series_logic::make_sn74_series;
use crate::tdk_c_series::make_tdk_c_series_capacitor;
use crate::tdk_cga_series::make_tdk_cga_capacitor;
use crate::traco_power_tmr1_series::make_traco_tmr1_regulator;
use crate::wurth_led::make_wurth_led;
use crate::yageo_cc_caps::make_yageo_cc_series_cap;
use crate::yageo_resistor_series::make_yageo_series_resistor;

pub mod adc;
pub mod analog_devices;
pub mod avx_caps;
pub mod connectors;
pub mod digikey_table;
pub mod isolators;
pub mod kemet_ceramic_caps;
pub mod kemet_t491_series;
pub mod ldo;
pub mod lvc_one_gate;
pub mod murata_mlcc_caps;
pub mod nippon_electrolytic_caps;
pub mod panasonic_era_resistors;
pub mod schematic_manual_layout;
pub mod sn74_series_logic;
pub mod tdk_c_series;
pub mod tdk_cga_series;
pub mod traco_power_tmr1_series;
pub mod wurth_led;
pub mod yageo_cc_caps;
pub mod yageo_resistor_series;

#[test]
fn test_yageo_rc_68k() {
    let led_limit = match make_yageo_series_resistor("RC0603FR-0768KL") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    println!("{:?}", led_limit);
    assert_eq!(led_limit.value_ohms, 68e3);
    assert_eq!(led_limit.details.size, SizeCode::I0603);
    assert!(led_limit.power_watt >= PowerWatt::new(1, 10));
}

#[test]
fn test_yageo_rc_1k() {
    let current_limit = match make_yageo_series_resistor("RC1206FR-071KL") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    assert_eq!(current_limit.value_ohms, 1.0e3);
    assert_eq!(current_limit.details.size, SizeCode::I1206);
    assert!(current_limit.power_watt >= PowerWatt::new(1, 4));
    assert_eq!(current_limit.details.pins.len(), 2);
}

#[test]
fn test_tdk_cga_cap() {
    let filter_cap = as_cap(make_tdk_cga_capacitor("CGA4J2X7R2A104K125AA"));
    // 'TDK         CGA4J2X7R2A104K125AA             SMD Multilayer Ceramic Capacitor, 0805 [2012 Metric], 0.1 F, 100 V,  10%, X7R, CGA Series
    assert_eq!(filter_cap.details.manufacturer.name, "TDK");
    assert_eq!(
        filter_cap.kind,
        CapacitorKind::MultiLayerChip(DielectricCode::X7R)
    );
    assert_eq!(filter_cap.details.size, SizeCode::I0805);
    assert_eq!(filter_cap.value_pf, 0.1 * 1e6);
    assert_eq!(filter_cap.voltage, 100.);
    println!("{:?}", filter_cap);
}

#[test]
fn test_kemet_tantalum_cap() {
    let kemet = as_cap(make_kemet_t491_capacitor("T491A106K010AT"));
    assert_eq!(kemet.details.size, SizeCode::I1206);
    assert_eq!(kemet.kind, CapacitorKind::Tantalum);
    assert_eq!(kemet.value_pf, 10. * 1e6);
    assert_eq!(kemet.voltage, 10.);
    assert_eq!(kemet.tolerance, CapacitorTolerance::TenPercent);
    println!("{:#?}", kemet);
}

#[test]
fn test_yageo_precision() {
    let precise = match make_yageo_series_resistor("RL0603FR-070R47L") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    // 'Res Thick Film 0603 0.47 Ohm 1% 0.1W(1/10W) ±800ppm/C Molded SMD Paper T/R
    assert_eq!(precise.details.size, SizeCode::I0603);
    assert_eq!(precise.tolerance, 1.0);
    assert_eq!(precise.value_ohms, 0.47);
}

#[test]
fn test_yageo_bulk() {
    let bulk = match make_yageo_series_resistor("RC1206FR-071KL") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    // 'YAGEO - RC1206FR-071KL. - RES, THICK FILM, 1K, 1%, 0.25W, 1206, REEL
    assert_eq!(bulk.tolerance, 1.0);
    assert_eq!(bulk.power_watt, PowerWatt::new(1, 4));
    assert_eq!(bulk.details.size, SizeCode::I1206);
    assert_eq!(bulk.value_ohms, 1.0e3);
}

#[test]
fn test_yageo_tempco() {
    // T491A106K010AT
    let temp = match make_yageo_series_resistor("AT0603BRD0720KL") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    // '20K 0.1% precision
    assert_eq!(temp.tolerance, 0.1);
    assert_eq!(temp.value_ohms, 20e3);
    assert_eq!(temp.tempco, Some(25.0));
}

#[test]
fn test_avx() {
    let avx = match make_avx_capacitor("22201C475KAT2A") {
        CircuitNode::Capacitor(c) => c,
        _ => panic!(),
    };
    assert_eq!(avx.details.size, SizeCode::I2220);
    assert_eq!(avx.kind, CapacitorKind::MultiLayerChip(DielectricCode::X7R));
    assert_eq!(avx.value_pf, 47e5);
    assert_eq!(avx.tolerance, CapacitorTolerance::TenPercent);
}

#[test]
fn test_kemet() {
    let c = match make_kemet_ceramic_capacitor("C0603C104K5RACTU") {
        CircuitNode::Capacitor(c) => c,
        _ => panic!(),
    };
    assert_eq!(c.details.size, SizeCode::I0603);
    assert_eq!(c.kind, CapacitorKind::MultiLayerChip(DielectricCode::X7R));
    assert_eq!(c.value_pf, 10e4);
    assert_eq!(c.tolerance, CapacitorTolerance::TenPercent);
}

#[test]
fn test_tdk_c_series() {
    let c = as_cap(make_tdk_c_series_capacitor("C1608X7R1C105K080AC"));
    assert_eq!(c.details.size, SizeCode::I0603);
    assert_eq!(c.kind, CapacitorKind::MultiLayerChip(DielectricCode::X7R));
    assert_eq!(c.value_pf, 10e5);
    assert_eq!(c.tolerance, CapacitorTolerance::TenPercent);
    assert_eq!(c.voltage, 16.0);
}

#[test]
fn test_yageo_cc_series() {
    let c = as_cap(make_yageo_cc_series_cap("CC0805KKX5R8BB106"));
    // 'Cap Ceramic 10uF 25V X5R 10% Pad SMD 0805 85°C T/R
    assert_eq!(c.details.size, SizeCode::I0805);
    assert_eq!(c.tolerance, CapacitorTolerance::TenPercent);
    assert_eq!(c.voltage, 25.0);
    assert_eq!(c.kind, CapacitorKind::MultiLayerChip(DielectricCode::X5R));
    assert_eq!(c.value_pf, 10. * 1e6);
}

#[cfg(test)]
fn as_cap(c: CircuitNode) -> Capacitor {
    match c {
        CircuitNode::Capacitor(c) => c,
        _ => panic!(),
    }
}

#[test]
fn test_murata_grt_series() {
    let c = as_cap(make_murata_capacitor("GRT188R61H105KE13D"));
    // 'Multilayer Ceramic Capacitors MLCC - SMD/SMT 0603 50Vdc 1.0uF X5R 10%
    assert_eq!(c.details.size, SizeCode::I0603);
    assert_eq!(c.tolerance, CapacitorTolerance::TenPercent);
    assert_eq!(c.voltage, 50.0);
    assert_eq!(c.kind, CapacitorKind::MultiLayerChip(DielectricCode::X5R));
    assert_eq!(c.value_pf, 10. * 1e5);
}

#[test]
fn test_panasonic_era_series() {
    let r = match make_panasonic_resistor("ERA8AEB201V") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    // 'RES SMD 200 OHM 0.1% 1/4W 1206
    assert_eq!(r.details.size, SizeCode::I1206);
    assert_eq!(r.tolerance, 0.1);
    assert_eq!(r.value_ohms, 200.);
    assert_eq!(r.power_watt, PowerWatt::new(1, 4));
    assert_eq!(r.kind, ResistorKind::ThinFilmChip);
    assert_eq!(r.tempco, Some(25.));
}

#[test]
fn test_panasonic_erj_series() {
    let r = match make_panasonic_resistor("ERJ-3RQFR22V") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    // 'Res Thick Film 0603 0.22 Ohm 1% 1/10W ±300ppm/°C Molded SMD Punched Carrier T/R
    assert_eq!(r.details.size, SizeCode::I0603);
    assert_eq!(r.tolerance, 1.);
    assert_eq!(r.value_ohms, 0.22);
    assert_eq!(r.power_watt, PowerWatt::new(1, 10));
    assert_eq!(r.kind, ResistorKind::ThickFilmChip);
}

#[test]
fn test_murata_grm_series() {
    let c = as_cap(make_murata_capacitor("GRM21BR61C226ME44L"));
    // '0805 22 uF 16 V ±20% Tolerance X5R Multilayer Ceramic Chip Capacitor
    assert_eq!(c.details.size, SizeCode::I0805);
    assert_eq!(c.voltage, 16.);
    assert_eq!(c.tolerance, CapacitorTolerance::TwentyPercent);
    assert_eq!(c.kind, CapacitorKind::MultiLayerChip(DielectricCode::X5R));
    assert_eq!(c.value_pf, 22e6);
}

#[test]
fn test_chemi_con_hybrid_cap() {
    let c = as_cap(make_nippon_hxd_capacitor("HHXD500ARA101MJA0G"));
    // 100 uF, 50V Alum Poly 25 mR ESR, Hybrid
    assert_eq!(c.voltage, 50.);
    assert_eq!(c.kind, CapacitorKind::AluminumPolyLowESR(25));
    assert_eq!(c.value_pf, 100. * 1e6);
    assert_eq!(c.details.size, SizeCode::Custom("JA0".to_owned()))
}

#[test]
fn test_yageo_pth_resistors() {
    let r = match make_yageo_series_resistor("FMP100JR-52-15K") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    assert_eq!(r.tolerance, 5.);
    assert_eq!(r.power_watt, PowerWatt::new(1, 1));
    assert_eq!(r.value_ohms, 15e3);
    let r = match make_yageo_series_resistor("FMP100JR-52-10R") {
        CircuitNode::Resistor(r) => r,
        _ => panic!(),
    };
    assert_eq!(r.tolerance, 5.);
    assert_eq!(r.power_watt, PowerWatt::new(1, 1));
    assert_eq!(r.value_ohms, 10.0);
}

#[test]
fn test_green_led() {
    let d = match make_wurth_led("150060GS75000") {
        CircuitNode::Diode(d) => d,
        _ => panic!(),
    };
    assert_eq!(d.kind, DiodeKind::LED("Green".into()));
    assert_eq!(d.forward_drop_volts, 3.2);
    assert_eq!(d.details.pins.len(), 2);
    assert_eq!(d.details.pins[&1].kind, PinKind::PassiveNeg);
    assert_eq!(d.details.pins[&2].kind, PinKind::PassivePos);
}

#[test]
fn test_zldo_regulator() {
    let v = match make_zldo1117g_regulator("ZLDO1117G50TA") {
        CircuitNode::Regulator(r) => r,
        _ => panic!(),
    };
    assert_eq!(v.input_max_voltage, 18.0);
    assert_eq!(v.output_nominal_voltage, 5.0);
    assert_eq!(v.details.pins.len(), 4);
    assert_eq!(v.details.pins[&1].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&2].kind, PinKind::PowerSource);
    assert_eq!(v.details.pins[&3].kind, PinKind::PowerSink);
    assert_eq!(v.details.pins[&4].kind, PinKind::PowerSource);
    let v = match make_zldo1117g_regulator("ZLDO1117G33TA") {
        CircuitNode::Regulator(r) => r,
        _ => panic!(),
    };
    assert_eq!(v.input_max_voltage, 18.0);
    assert_eq!(v.output_nominal_voltage, 3.3);
    assert_eq!(v.details.pins.len(), 4);
    assert_eq!(v.details.pins[&1].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&2].kind, PinKind::PowerSource);
    assert_eq!(v.details.pins[&3].kind, PinKind::PowerSink);
    assert_eq!(v.details.pins[&4].kind, PinKind::PowerSource);
}

#[test]
fn test_ti_tps_regulator() {
    let v = match make_ti_tps_7b84_regulator("TPS7B8450QDCYRQ1") {
        CircuitNode::Regulator(r) => r,
        _ => panic!(),
    };
    assert_eq!(v.input_max_voltage, 42.0);
    assert_eq!(v.output_nominal_voltage, 5.0);
    assert_eq!(v.output_max_current_ma, 150.0);
    assert_eq!(v.details.pins[&1].kind, PinKind::PowerSink);
    assert_eq!(v.details.pins[&1].name, "IN");
    assert_eq!(v.details.pins[&2].kind, PinKind::Input);
    assert_eq!(v.details.pins[&2].name, "EN");
    assert_eq!(v.details.pins[&3].kind, PinKind::PowerSource);
    assert_eq!(v.details.pins[&3].name, "OUT");
    assert_eq!(v.details.pins[&4].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&4].name, "GND");
    let v = match make_ti_tps_7b84_regulator("TPS7B8433QDCYRQ1") {
        CircuitNode::Regulator(r) => r,
        _ => panic!(),
    };
    assert_eq!(v.input_max_voltage, 42.0);
    assert_eq!(v.output_nominal_voltage, 3.3);
    assert_eq!(v.output_max_current_ma, 150.0);
    assert_eq!(v.details.pins[&1].kind, PinKind::PowerSink);
    assert_eq!(v.details.pins[&1].name, "IN");
    assert_eq!(v.details.pins[&2].kind, PinKind::Input);
    assert_eq!(v.details.pins[&2].name, "EN");
    assert_eq!(v.details.pins[&3].kind, PinKind::PowerSource);
    assert_eq!(v.details.pins[&3].name, "OUT");
    assert_eq!(v.details.pins[&4].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&4].name, "GND");
}

#[test]
fn test_on_semi_regulators() {
    let v = match make_on_semi_ncv33375_regulator("NCV33375ST3.3T3G") {
        CircuitNode::Regulator(v) => v,
        _ => panic!(),
    };
    assert_eq!(v.input_max_voltage, 13.0);
    assert_eq!(v.output_nominal_voltage, 3.3);
    assert_eq!(v.output_max_current_ma, 300.0);
    assert_eq!(v.details.pins[&1].kind, PinKind::PowerSink);
    assert_eq!(v.details.pins[&1].name, "VIN");
    assert_eq!(v.details.pins[&2].kind, PinKind::Input);
    assert_eq!(v.details.pins[&2].name, "ON/OFF");
    assert_eq!(v.details.pins[&3].kind, PinKind::PowerSource);
    assert_eq!(v.details.pins[&3].name, "VOUT");
    assert_eq!(v.details.pins[&4].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&4].name, "GND");
    let v = match make_on_semi_ncv33375_regulator("NCV33375ST1.8T3G") {
        CircuitNode::Regulator(v) => v,
        _ => panic!(),
    };
    assert_eq!(v.input_max_voltage, 13.0);
    assert_eq!(v.output_nominal_voltage, 1.8);
    assert_eq!(v.output_max_current_ma, 300.0);
    assert_eq!(v.details.pins[&1].kind, PinKind::PowerSink);
    assert_eq!(v.details.pins[&1].name, "VIN");
    assert_eq!(v.details.pins[&2].kind, PinKind::Input);
    assert_eq!(v.details.pins[&2].name, "ON/OFF");
    assert_eq!(v.details.pins[&3].kind, PinKind::PowerSource);
    assert_eq!(v.details.pins[&3].name, "VOUT");
    assert_eq!(v.details.pins[&4].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&4].name, "GND");
}

#[test]
fn test_microchip_regulators() {
    let v = match make_mcp_1799_regulator("MCP1799T-5002H/DB") {
        CircuitNode::Regulator(r) => r,
        _ => panic!(),
    };
    assert_eq!(v.input_max_voltage, 45.0);
    assert_eq!(v.output_nominal_voltage, 5.0);
    assert_eq!(v.output_max_current_ma, 80.0);
    assert_eq!(v.details.pins[&1].kind, PinKind::PowerSink);
    assert_eq!(v.details.pins[&1].name, "VIN");
    assert_eq!(v.details.pins[&2].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&2].name, "GND_1");
    assert_eq!(v.details.pins[&3].kind, PinKind::PowerSource);
    assert_eq!(v.details.pins[&3].name, "VOUT");
    assert_eq!(v.details.pins[&4].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&4].name, "GND_2");
    let v = match make_mcp_1799_regulator("MCP1799T-3302H/DB") {
        CircuitNode::Regulator(r) => r,
        _ => panic!(),
    };
    assert_eq!(v.input_max_voltage, 45.0);
    assert_eq!(v.output_nominal_voltage, 3.3);
    assert_eq!(v.output_max_current_ma, 80.0);
    assert_eq!(v.details.pins[&1].kind, PinKind::PowerSink);
    assert_eq!(v.details.pins[&1].name, "VIN");
    assert_eq!(v.details.pins[&2].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&2].name, "GND_1");
    assert_eq!(v.details.pins[&3].kind, PinKind::PowerSource);
    assert_eq!(v.details.pins[&3].name, "VOUT");
    assert_eq!(v.details.pins[&4].kind, PinKind::PowerReturn);
    assert_eq!(v.details.pins[&4].name, "GND_2");
}

#[test]
fn test_lt3092() {
    let u = match make_lt3092_current_source("LT3092EST#PBF") {
        CircuitNode::IntegratedCircuit(u) => u,
        _ => panic!(),
    };
    assert_eq!(u.pins.len(), 4);
    assert_eq!(u.pins[&1].kind, PinKind::Input);
    assert_eq!(u.pins[&1].name, "SET");
    assert_eq!(u.pins[&2].kind, PinKind::PowerSource);
    assert_eq!(u.pins[&2].name, "OUT_1");
    assert_eq!(u.pins[&3].kind, PinKind::PowerSink);
    assert_eq!(u.pins[&3].name, "IN");
    assert_eq!(u.pins[&4].kind, PinKind::PowerSource);
    assert_eq!(u.pins[&4].name, "OUT_2");
}

#[test]
fn test_brl() {
    let l = match make_ty_brl_series("BRL3225T101K") {
        CircuitNode::Inductor(i) => i,
        _ => panic!(),
    };
    assert_eq!(l.details.size, SizeCode::I1210);
    assert_eq!(l.details.pins.len(), 2);
    assert_eq!(l.max_current_milliamps, 250.0);
    assert_eq!(l.dc_resistance_ohms, 2.5);
    assert_eq!(l.value_microhenry, 100.0);
    assert_eq!(l.details.pins[&1].kind, PinKind::Passive);
    assert_eq!(l.details.pins[&2].kind, PinKind::Passive);
}

#[test]
fn test_xor() {
    let u = match make_lvc_one_gate("SN74LVC1G86DCK") {
        CircuitNode::Logic(l) => l,
        _ => panic!(),
    };
    assert_eq!(u.input_type, LogicSignalStandard::WideRange);
    assert_eq!(u.output_type, LogicSignalStandard::WideRange);
    assert_eq!(u.min_supply_voltage, 1.65);
    assert_eq!(u.max_supply_voltage, 5.5);
    assert_eq!(u.drive_current_ma, 32.0);
    assert_eq!(u.details.size, SizeCode::SC70);
    assert_eq!(u.function, LogicFunction::XOR);
    assert_eq!(u.details.pins[&1].kind, PinKind::Input);
    assert_eq!(u.details.pins[&1].name, "A");
    assert_eq!(u.details.pins[&2].kind, PinKind::Input);
    assert_eq!(u.details.pins[&2].name, "B");
    assert_eq!(u.details.pins[&3].kind, PinKind::PowerReturn);
    assert_eq!(u.details.pins[&3].name, "GND");
    assert_eq!(u.details.pins[&4].kind, PinKind::Output);
    assert_eq!(u.details.pins[&4].name, "Y");
    assert_eq!(u.details.pins[&5].kind, PinKind::PowerSink);
    assert_eq!(u.details.pins[&5].name, "VCC");
}

#[test]
fn test_octal_buffer() {
    let u = match make_sn74_series("SN74HCT541PWR") {
        CircuitNode::Logic(l) => l,
        _ => panic!(),
    };
    assert_eq!(u.details.manufacturer.name, "TI");
    assert_eq!(u.details.manufacturer.part_number, "SN74HCT541PWR");
    assert_eq!(u.input_type, LogicSignalStandard::TTL);
    assert_eq!(u.output_type, LogicSignalStandard::TriState5v0);
    assert_eq!(u.drive_current_ma, 6.0);
    assert_eq!(u.min_supply_voltage, 4.5);
    assert_eq!(u.max_supply_voltage, 5.5);
    assert_eq!(u.details.size, SizeCode::TSSOP(20));
    assert_eq!(u.function, LogicFunction::Buffer);
    assert_eq!(u.details.pins[&1].name, "~OE1");
    assert_eq!(u.details.pins[&1].kind, PinKind::InputInverted);
    assert_eq!(u.details.pins[&19].name, "~OE2");
    assert_eq!(u.details.pins[&19].kind, PinKind::InputInverted);
    assert_eq!(u.details.pins[&10].name, "GND");
    assert_eq!(u.details.pins[&10].kind, PinKind::PowerReturn);
    assert_eq!(u.details.pins[&20].name, "VCC");
    assert_eq!(u.details.pins[&20].kind, PinKind::PowerSink);
    for i in 2..=9 {
        assert_eq!(u.details.pins[&i].name, format!("A{}", i - 1));
        assert_eq!(u.details.pins[&i].kind, PinKind::Input);
        assert_eq!(u.details.pins[&(19 - i + 1)].name, format!("Y{}", i - 1));
        assert_eq!(u.details.pins[&(19 - i + 1)].kind, PinKind::TriState);
    }
}

#[test]
fn test_decoder() {
    let u = match make_sn74_series("SN74HCT138PWR") {
        CircuitNode::Logic(l) => l,
        _ => panic!(),
    };
    assert_eq!(u.details.manufacturer.name, "TI");
    assert_eq!(u.details.manufacturer.part_number, "SN74HCT138PWR");
    assert_eq!(u.input_type, LogicSignalStandard::TTL);
    assert_eq!(u.output_type, LogicSignalStandard::TTL);
    assert_eq!(u.drive_current_ma, 4.0);
    assert_eq!(u.min_supply_voltage, 4.5);
    assert_eq!(u.max_supply_voltage, 5.5);
    assert_eq!(u.details.size, SizeCode::TSSOP(16));
    assert_eq!(u.function, LogicFunction::Decoder);
    assert_eq!(u.details.pins[&1].name, "A");
    assert_eq!(u.details.pins[&1].kind, PinKind::Input);
    assert_eq!(u.details.pins[&2].name, "B");
    assert_eq!(u.details.pins[&2].kind, PinKind::Input);
    assert_eq!(u.details.pins[&3].name, "C");
    assert_eq!(u.details.pins[&3].kind, PinKind::Input);
    assert_eq!(u.details.pins[&4].name, "~G2A");
    assert_eq!(u.details.pins[&4].kind, PinKind::InputInverted);
    assert_eq!(u.details.pins[&5].name, "~G2B");
    assert_eq!(u.details.pins[&5].kind, PinKind::InputInverted);
    assert_eq!(u.details.pins[&6].name, "G1");
    assert_eq!(u.details.pins[&6].kind, PinKind::Input);
    assert_eq!(u.details.pins[&7].name, "Y7");
    assert_eq!(u.details.pins[&7].kind, PinKind::Output);
    assert_eq!(u.details.pins[&8].name, "GND");
    assert_eq!(u.details.pins[&8].kind, PinKind::PowerReturn);
    assert_eq!(u.details.pins[&16].name, "VCC");
    assert_eq!(u.details.pins[&16].kind, PinKind::PowerSink);
    for i in 9..=15 {
        assert_eq!(u.details.pins[&i].name, format!("Y{}", 15 - i));
        assert_eq!(u.details.pins[&i].kind, PinKind::Output);
    }
}

#[test]
fn test_multiplexer() {
    let u = match make_sn74_series("SN74HC151QDRQ1") {
        CircuitNode::Logic(l) => l,
        _ => panic!(),
    };
    assert_eq!(u.details.manufacturer.name, "TI");
    assert_eq!(u.details.manufacturer.part_number, "SN74HC151QDRQ1");
    assert_eq!(u.input_type, LogicSignalStandard::TTL);
    assert_eq!(u.output_type, LogicSignalStandard::TTL);
    assert_eq!(u.drive_current_ma, 6.0);
    assert_eq!(u.min_supply_voltage, 2.0);
    assert_eq!(u.max_supply_voltage, 6.0);
    assert_eq!(u.details.size, SizeCode::SOIC(16));
    assert_eq!(u.function, LogicFunction::Multiplexer);
    for i in 0..=3 {
        assert_eq!(u.details.pins[&(i + 1)].name, format!("D{}", 3 - i));
        assert_eq!(u.details.pins[&(i + 1)].kind, PinKind::Input);
        assert_eq!(u.details.pins[&(i + 12)].name, format!("D{}", 7 - i));
        assert_eq!(u.details.pins[&(i + 12)].kind, PinKind::Input);
    }
    assert_eq!(u.details.pins[&5].name, "Y");
    assert_eq!(u.details.pins[&5].kind, PinKind::Output);
    assert_eq!(u.details.pins[&6].name, "W");
    assert_eq!(u.details.pins[&6].kind, PinKind::Output);
    assert_eq!(u.details.pins[&7].name, "~G");
    assert_eq!(u.details.pins[&7].kind, PinKind::InputInverted);
    assert_eq!(u.details.pins[&8].name, "GND");
    assert_eq!(u.details.pins[&8].kind, PinKind::PowerReturn);
    assert_eq!(u.details.pins[&9].name, "C");
    assert_eq!(u.details.pins[&9].kind, PinKind::Input);
    assert_eq!(u.details.pins[&10].name, "B");
    assert_eq!(u.details.pins[&10].kind, PinKind::Input);
    assert_eq!(u.details.pins[&11].name, "A");
    assert_eq!(u.details.pins[&11].kind, PinKind::Input);
    assert_eq!(u.details.pins[&16].name, "VCC");
    assert_eq!(u.details.pins[&16].kind, PinKind::PowerSink);
}

#[test]
fn test_buffer() {
    let u = match make_lvc_one_gate("74LVC1G125SE-7") {
        CircuitNode::Logic(l) => l,
        _ => panic!(),
    };
    assert_eq!(u.input_type, LogicSignalStandard::WideRange);
    assert_eq!(u.output_type, LogicSignalStandard::TriState);
    assert_eq!(u.min_supply_voltage, 1.65);
    assert_eq!(u.max_supply_voltage, 5.5);
    assert_eq!(u.drive_current_ma, 24.0);
    assert_eq!(u.details.size, SizeCode::SOT353);
    assert_eq!(u.function, LogicFunction::Buffer);
    assert_eq!(u.details.pins[&1].kind, PinKind::InputInverted);
    assert_eq!(u.details.pins[&1].name, "~OE");
    assert_eq!(u.details.pins[&2].kind, PinKind::Input);
    assert_eq!(u.details.pins[&2].name, "A");
    assert_eq!(u.details.pins[&3].kind, PinKind::PowerReturn);
    assert_eq!(u.details.pins[&3].name, "GND");
    assert_eq!(u.details.pins[&4].kind, PinKind::TriState);
    assert_eq!(u.details.pins[&4].name, "Y");
    assert_eq!(u.details.pins[&5].kind, PinKind::PowerSink);
    assert_eq!(u.details.pins[&5].name, "VCC");
}

#[test]
fn test_isolator() {
    let u = match make_iso7741edwrq1("ISO7741EDWRQ1") {
        CircuitNode::IntegratedCircuit(u) => u,
        _ => panic!(),
    };
    for i in [2, 8, 9, 15] {
        assert_eq!(u.pins[&i].kind, PinKind::PowerReturn);
        assert!(u.pins[&i].name.starts_with("GND"));
    }
    for i in [3, 4, 5, 11] {
        assert_eq!(u.pins[&i].kind, PinKind::Input);
    }
    for i in [6, 12, 13, 14] {
        assert_eq!(u.pins[&i].kind, PinKind::Output);
    }
    assert_eq!(
        u.pins.iter().map(|x| x.1.name.clone()).collect::<Vec<_>>(),
        vec![
            "VCC1", "GND1_1", "INA", "INB", "INC", "OUTD", "EN1", "GND1_2", "GND2_2", "EN2", "IND",
            "OUTC", "OUTB", "OUTA", "GND2_1", "VCC2"
        ]
    );
    assert_eq!(u.size, SizeCode::SOIC(16));
}

#[test]
fn test_ads8689() {
    let u = match make_ads868x("ADS8689IPW") {
        CircuitNode::IntegratedCircuit(u) => u,
        _ => panic!("Wrong type returned"),
    };
    assert_eq!(
        u.pins.iter().map(|x| x.1.name.clone()).collect::<Vec<_>>(),
        vec![
            "DGND",
            "AVDD",
            "AGND",
            "REFIO",
            "REFGND",
            "REFCAP",
            "AIN_P",
            "AIN_GND",
            "~RST",
            "SDI",
            "CONVST/~CS",
            "SCLK",
            "SDO-0",
            "ALARM/SDO-1/GPO",
            "RVS",
            "DVDD"
        ]
    );
    assert_eq!(u.size, SizeCode::TSSOP(16));
    assert_eq!(u.manufacturer.name, "TI");
}

#[test]
fn test_sullins_connector() {
    let j = match make_sullins_sbh11_header("SBH11-PBPC-D13-RA-BK") {
        CircuitNode::Connector(j) => j,
        _ => panic!(),
    };
    assert_eq!(j.manufacturer.name, "Sullins Connector Solutions");
    assert_eq!(j.pins.len(), 26);
    for pin in &j.pins {
        assert_eq!(pin.1.name, format!("{}", pin.0));
        assert_eq!(pin.1.kind, PinKind::Passive);
    }
}

#[test]
fn test_molex_connector() {
    let j = match make_molex_55935_connector("0559350810") {
        CircuitNode::Connector(j) => j,
        _ => panic!(),
    };
    assert_eq!(j.manufacturer.name, "Molex");
    assert_eq!(j.pins.len(), 8);
    for pin in &j.pins {
        assert_eq!(pin.1.name, format!("{}", pin.0));
        assert_eq!(pin.1.kind, PinKind::Passive);
    }
}

#[test]
fn test_amphenol_connector() {
    let j = match make_amphenol_10056845_header("10056845-108LF") {
        CircuitNode::Connector(j) => j,
        _ => panic!(),
    };
    assert_eq!(j.manufacturer.name, "Amphenol");
    assert_eq!(j.pins.len(), 8);
    for pin in &j.pins {
        assert_eq!(pin.1.name, format!("{}", pin.0));
        assert_eq!(pin.1.kind, PinKind::Passive);
    }
}

#[cfg(test)]
fn make_sample_library() -> Vec<CircuitNode> {
    vec![
        make_ads868x("ADS8689IPW"),
        make_lt3092_current_source("LT3092EST#PBF"),
        make_molex_55935_connector("0559350810"),
        make_amphenol_10056845_header("10056845-108LF"),
        make_on_semi_ncv33375_regulator("NCV33375ST3.3T3G"),
        make_zldo1117g_regulator("ZLDO1117G50TA"),
        make_ti_tps_7b84_regulator("TPS7B8450QDCYRQ1"),
        make_mcp_1799_regulator("MCP1799T-5002H/DB"),
        make_lt3092_current_source("LT3092EST#PBF"),
        make_lvc_one_gate("SN74LVC1G86DCK"),
        make_sn74_series("SN74HCT541PWR"),
        make_sn74_series("SN74HCT138PWR"),
        make_sn74_series("SN74HC151QDRQ1"),
        make_lvc_one_gate("74LVC1G125SE-7"),
        make_iso7741edwrq1("ISO7741EDWRQ1"),
        make_ads868x("ADS8689IPW"),
        make_yageo_series_resistor("RC0603FR-0768KL"),
        make_kemet_ceramic_capacitor("C0603C104K5RACTU"),
        make_nippon_hxd_capacitor("HHXD500ARA101MJA0G"),
        make_ty_brl_series("BRL3225T101K"),
        make_wurth_led("150060GS75000"),
        make_traco_tmr1_regulator("TMR1-2415"),
    ]
}

#[test]
fn test_schematics() {
    for p in make_sample_library() {
        println!("SVG Generation for {:?}", p);
        let mut i = instance(p, "p1");
        make_svgs(&mut i);
    }
}

#[test]
fn test_composite_circuit() {

    /*
    let layout = LayoutEngine::new();
    let in_resistor = make_yageo_series_resistor("RC1206FR-071KL");
    let input_cap = make_murata_capacitor("GRT188R61H105KE13D").rot90();
    let output_cap = make_yageo_cc_series_cap("CC0805KKX5R8BB106").rot90();
    let v_reg = make_ti_tps_7b84_regulator("TPS7B8433QDCYRQ1");
    let x = pin!("foo", PowerSink, 0, West);
    let pins = vec![
        pin!("+24V", PowerSink, 0, West),
        pin!("GND", PowerReturn, 0, South),
        pin!("+3V3", PowerSource, 0, East),
    ];

     */
    /*
    V - D - ! - L - ! - ! -
        !   C       C   C
        R - ! - L - ! - ! -
     */
}
