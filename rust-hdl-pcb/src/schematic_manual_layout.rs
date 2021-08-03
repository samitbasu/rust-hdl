use crate::circuit::SchematicRotation::{Horizontal, Vertical};
use crate::circuit::{Circuit, CircuitNode, Net, PartInstance, SchematicRotation};
use crate::glyph::make_ic_body;
use crate::ldo::make_ti_tps_7b84_regulator;
use crate::murata_mlcc_caps::make_murata_capacitor;
use crate::port::make_port;
use crate::schematic::write_circuit_to_svg;
use crate::yageo_cc_caps::make_yageo_cc_series_cap;
use crate::yageo_resistor_series::make_yageo_series_resistor;
use crate::junction::make_junction;

pub fn place(x: CircuitNode, xc: i32, yc: i32, orientation: SchematicRotation) -> PartInstance {
    let mut y: PartInstance = x.into();
    if orientation == SchematicRotation::Vertical {
        y = y.rot90();
    }
    y.schematic_orientation.center = crate::glyph::Point { x: xc, y: yc };
    y
}

#[test]
fn test_manual_layout() {
    use crate::epin::PinKind;

    let in_power_port = place(
        make_port("+VIN", PinKind::PowerSink),
        -800,
        200,
        Horizontal,
    );
    let out_power_port = place(
        make_port("+3V3_OUT", PinKind::PowerSource),
        4800,
        200,
        Horizontal,
    )
    .flip_lr();
    let gnd_port = place(make_port("GND", PinKind::PowerReturn),
                         2300, -1200, Vertical);
    let in_resistor = place(
        make_yageo_series_resistor("RC1206FR-071KL"),
        0,
        200,
        Horizontal,
    );
    let input_cap = place(
        make_murata_capacitor("GRT188R61H105KE13D"),
        900,
        -200,
        Vertical,
    );
    let v_reg = place(
        make_ti_tps_7b84_regulator("TPS7B8433QDCYRQ1"),
        2300,
        0,
        Horizontal,
    );
    let output_cap = place(
        make_yageo_cc_series_cap("CC0805KKX5R8BB106"),
        3500,
        -200,
        Vertical,
    );
    let star_1 = place(make_junction(), 900, 200, Horizontal);
    let star_2 = place(make_junction(), 3500, 200, Horizontal);
    let star_3 = place(make_junction(), 2300, -800, Horizontal);
    let vup_net = Net::new(Some("+VIN"))
        .add(&in_power_port, 1, &in_resistor, 1);
    let vin_net = Net::new(None)
        .add(&in_resistor, 2, &star_1, 1)
        .add(&input_cap, 2, &star_1, 1)
        .add(&v_reg, 1, &star_1, 1)
        .add(&v_reg, 2, &v_reg, 1);
    let gnd_net = Net::new(Some("GND"))
        .add_via(&input_cap, 1, &star_3, 1, vec![(900,-800)])
        .add(&v_reg, 4, &star_3, 1)
        .add(&gnd_port, 1, &star_3, 1)
        .add_via(&output_cap, 1, &star_3, 1, vec![(3500,-800)]);
    let vout_net = Net::new(Some("+3v3"))
        .add(&out_power_port, 1, &star_2, 1)
        .add(&v_reg, 3, &star_2, 1)
        .add(&output_cap, 2, &star_2, 1);
    let circuit = Circuit {
        nodes: vec![
            in_resistor,
            input_cap,
            v_reg,
            output_cap,
            in_power_port,
            out_power_port,
            gnd_port,
            star_1,
            star_2,
            star_3,
        ],
        nets: vec![vup_net, vin_net, gnd_net, vout_net],
    };
    write_circuit_to_svg(&circuit, "test_circuit_manual.svg");
}
