use crate::circuit::{PartInstance, CircuitNode, SchematicRotation};
use crate::ldo::make_ti_tps_7b84_regulator;
use crate::murata_mlcc_caps::make_murata_capacitor;
use crate::schematic::write_circuit_to_svg;
use crate::schematic_flexbox_layout::PTS_PER_MIL;
use crate::yageo_cc_caps::make_yageo_cc_series_cap;
use crate::yageo_resistor_series::make_yageo_series_resistor;
use crate::circuit::SchematicRotation::{Horizontal, Vertical};

pub fn place(x: CircuitNode, xc: i32, yc: i32, orientation: SchematicRotation) -> PartInstance {
    let mut y: PartInstance = x.into();
    if rotated {
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
    let in_resistor = place(make_yageo_series_resistor("RC1206FR-071KL"), 0, 200, Horizontal);
    let input_cap = place(make_murata_capacitor("GRT188R61H105KE13D"), 700, -150, Vertical);
    let v_reg = place(make_ti_tps_7b84_regulator("TPS7B8433QDCYRQ1"), 1900, 0, Horizontal);
    let output_cap = place(make_yageo_cc_series_cap("CC0805KKX5R8BB106"), 3000, -150, Vertical);
    let circuit = vec![&in_resistor, &input_cap, &v_reg, &output_cap];
    write_circuit_to_svg(&circuit, "test_circuit_manual.svg");
}
