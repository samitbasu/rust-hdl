use crate::circuit::{PartInstance, CircuitNode, SchematicRotation, Net, Circuit};
use crate::ldo::make_ti_tps_7b84_regulator;
use crate::murata_mlcc_caps::make_murata_capacitor;
use crate::schematic::write_circuit_to_svg;
use crate::schematic_flexbox_layout::PTS_PER_MIL;
use crate::yageo_cc_caps::make_yageo_cc_series_cap;
use crate::yageo_resistor_series::make_yageo_series_resistor;
use crate::circuit::SchematicRotation::{Horizontal, Vertical};
use crate::epin::{EPin, PinKind, PinLocation, EdgeLocation};
use crate::glyph::make_ic_body;

pub fn place(x: CircuitNode, xc: i32, yc: i32, orientation: SchematicRotation) -> PartInstance {
    let mut y: PartInstance = x.into();
    if orientation == SchematicRotation::Vertical {
        y = y.rot90();
    }
    y.schematic_orientation.center = crate::glyph::Point {
        x: xc,
        y: yc,
    };
    y
}

#[test]
fn test_manual_layout() {
    use crate::pin;
    use crate::utils::pin_list;

    let in_power = pin!("+VIN", PowerSink, 0, West);
    let out_power = pin!("+3V3_OUT", PowerSource, 0, East);
    let gnd_pin = pin!("GND", PowerReturn, 2000, South);
    let in_resistor = place(make_yageo_series_resistor("RC1206FR-071KL"), 0, 200, Horizontal);
    let input_cap = place(make_murata_capacitor("GRT188R61H105KE13D"), 900, -200, Vertical);
    let v_reg = place(make_ti_tps_7b84_regulator("TPS7B8433QDCYRQ1"), 2300, 0, Horizontal);
    let output_cap = place(make_yageo_cc_series_cap("CC0805KKX5R8BB106"), 3500, -200, Vertical);
    let vup_net = Net::new(Some("+VIN"))
        .add_port(1)
        .add(&in_resistor, 1);
    let vin_net = Net::new(None)
        .add(&in_resistor,2)
        .add(&input_cap, 2)
        .add(&v_reg, 1)
        .add(&v_reg, 2);
    let gnd_net = Net::new(Some("GND"))
        .add(&input_cap, 1)
        .add(&v_reg, 4)
        .add_port(3)
        .add(&output_cap, 1);
    let vout_net = Net::new(Some("+3v3"))
        .add_port(2)
        .add(&v_reg, 3)
        .add(&output_cap, 2);
    let circuit = Circuit {
        pins: pin_list(vec![in_power, out_power, gnd_pin]),
        nodes: vec![in_resistor, input_cap, v_reg, output_cap],
        nets: vec![vup_net, vin_net, gnd_net, vout_net],
        outline: vec![make_ic_body(-1000, -1000, 5000, 3000)],
    };
    write_circuit_to_svg(&circuit, "test_circuit_manual.svg");
}
