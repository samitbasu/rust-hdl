use crate::ldo::make_ti_tps_7b84_regulator;
use crate::murata_mlcc_caps::make_murata_capacitor;
use crate::yageo_cc_caps::make_yageo_cc_series_cap;
use crate::yageo_resistor_series::make_yageo_series_resistor;
use rust_hdl_pcb_core::prelude::*;
use rust_hdl_pcb_core::schematic_layout::NetLayoutCmd::{
    Junction, LineToCoords, LineToPort, MoveToCoords, MoveToPort,
};
use rust_hdl_pcb_core::schematic_layout::SchematicRotation::{Horizontal, Vertical};
use rust_hdl_pcb_kicad::write_circuit_to_kicad6;
use rust_hdl_pcb_svg::schematic::write_circuit_to_svg;

pub fn test_ldo_circuit() -> (Circuit, SchematicLayout) {
    let in_power_port = make_port("+VIN", PinKind::PowerSink).instance("in_power_port");
    let out_power_port = make_port("+3V3_OUT", PinKind::PowerSource).instance("out_power_port");
    let gnd_port = make_port("GND", PinKind::PowerReturn).instance("gnd_port");
    let in_resistor = make_yageo_series_resistor("RC1206FR-071KL").instance("in_resistor");
    let input_cap = make_murata_capacitor("GRT188R61H105KE13D").instance("input_cap");
    let v_reg = make_ti_tps_7b84_regulator("TPS7B8433QDCYRQ1").instance("v_reg");
    let output_cap = make_yageo_cc_series_cap("CC0805KKX5R8BB106").instance("output_cap");
    let mut layout = SchematicLayout::default();
    layout.set_part("in_power_port", orient().center(-800, 200));
    layout.set_part("out_power_port", orient().center(4800, 200).flip_lr());
    layout.set_part("gnd_port", orient().center(2300, -1200).vert());
    layout.set_part("in_resistor", orient().center(0, 200).horiz());
    layout.set_part("input_cap", orient().center(900, -200).vert());
    layout.set_part("v_reg", orient().center(2300, 0));
    layout.set_part("output_cap", orient().center(3500, -200).vert());
    let vup_net = Net::new("+VIN").add(&in_power_port, 1).add(&in_resistor, 1);
    let vin_net = Net::new("vin1")
        .add(&in_resistor, 2)
        .add(&input_cap, 2)
        .add(&v_reg, 1)
        .add(&v_reg, 2);
    let gnd_net = Net::new("GND")
        .add(&input_cap, 1)
        .add(&v_reg, 4)
        .add(&gnd_port, 1)
        .add(&output_cap, 1);
    let vout_net = Net::new("+3v3")
        .add(&out_power_port, 1)
        .add(&v_reg, 3)
        .add(&output_cap, 2);
    layout.set_net("+VIN", vec![MoveToPort(1), LineToPort(2)]);
    layout.set_net(
        "vin1",
        vec![
            MoveToPort(1),
            LineToCoords(900, 200),
            Junction,
            LineToPort(2),
            MoveToCoords(900, 200),
            LineToPort(3),
            Junction,
            LineToPort(4),
        ],
    );
    layout.set_net(
        "GND",
        vec![
            MoveToPort(1),
            LineToCoords(900, -800),
            LineToCoords(2300, -800),
            Junction,
            LineToPort(2),
            LineToPort(3),
            MoveToCoords(2300, -800),
            LineToCoords(3500, -800),
            LineToPort(4),
        ],
    );
    layout.set_net(
        "+3v3",
        vec![
            MoveToPort(1),
            LineToCoords(3500, 200),
            Junction,
            LineToPort(2),
            MoveToCoords(3500, 200),
            LineToPort(3),
        ],
    );
    let circuit = Circuit {
        nodes: vec![
            in_resistor,
            input_cap,
            v_reg,
            output_cap,
            in_power_port,
            out_power_port,
            gnd_port,
        ],
        nets: vec![vup_net, vin_net, gnd_net, vout_net],
    };
    (circuit, layout)
}

#[test]
fn test_manual_layout() {
    let (circuit, layout) = test_ldo_circuit();
    write_circuit_to_svg(&circuit, &layout, "test_circuit_manual.svg");
    //write_circuit_to_kicad6(&circuit, &layout, "test_circuit_manual.sch");
    write_circuit_to_kicad6(
        &circuit,
        &layout,
        "/Users/cdsfsbasu/Devel/rust-hdl/junk/test1.kicad_sch",
    );
    let layout_yaml = serde_yaml::to_string(&layout).unwrap();
    println!("Layout: {}", layout_yaml);
    let circuit = serde_json::to_string(&circuit).unwrap();
    println!("Circuit: {}", circuit);
}
