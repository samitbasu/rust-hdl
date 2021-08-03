use stretch::geometry::{Point, Size};
use stretch::node::Node;
use stretch::style::{Dimension, FlexDirection, JustifyContent, Style};
use stretch::Stretch;

use crate::circuit::PartInstance;
use crate::ldo::make_ti_tps_7b84_regulator;
use crate::murata_mlcc_caps::make_murata_capacitor;
use crate::schematic::{estimate_instance_bounding_box, write_circuit_to_svg};
use crate::yageo_cc_caps::make_yageo_cc_series_cap;
use crate::yageo_resistor_series::make_yageo_series_resistor;

#[derive(Clone, Copy, Debug)]
pub struct LayoutTag {
    node: Node,
    part: usize,
}

pub const PTS_PER_MIL: f32 = 72.0 / 1000.0;

pub fn make_node(stretch: &mut Stretch, part: &PartInstance) -> LayoutTag {
    let bbox = estimate_instance_bounding_box(part);
    let width = bbox.width() as f32 * PTS_PER_MIL;
    dbg!(width);
    let node = stretch
        .new_node(
            Style {
                size: Size {
                    width: Dimension::Points(bbox.width() as f32 * PTS_PER_MIL),
                    height: Dimension::Points(bbox.height() as f32 * PTS_PER_MIL),
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    LayoutTag {
        node,
        part: part.id.0,
    }
}

pub fn make_box(
    stretch: &mut Stretch,
    direction: FlexDirection,
    justify: JustifyContent,
    children: &[LayoutTag],
) -> LayoutTag {
    let node = stretch
        .new_node(
            Style {
                size: Size {
                    width: Dimension::Auto,
                    height: Dimension::Auto,
                },
                flex_direction: direction,
                justify_content: justify,
                ..Default::default()
            },
            children.iter().map(|x| x.node).collect(),
        )
        .unwrap();
    LayoutTag { node, part: 0 }
}

fn get_layout_position(
    stretch: &Stretch,
    children: &[Node],
    target: Node,
    offset: Point<f32>,
) -> Option<Point<f32>> {
    for child in children {
        let mut child_location = stretch.layout(*child).unwrap().location;
        child_location.x += offset.x;
        child_location.y += offset.y;
        if child.eq(&target) {
            return Some(child_location);
        }
        if let Some(p) = get_layout_position(
            stretch,
            &stretch.children(*child).unwrap(),
            target,
            child_location,
        ) {
            return Some(p);
        }
    }
    None
}

#[test]
fn test_flex_layout() {
    let mut in_resistor: PartInstance = make_yageo_series_resistor("RC1206FR-071KL").into();
    let mut input_cap: PartInstance = make_murata_capacitor("GRT188R61H105KE13D").into();
    let mut input_cap = input_cap.rot90();
    let mut output_cap: PartInstance = make_yageo_cc_series_cap("CC0805KKX5R8BB106").into();
    let mut output_cap = output_cap.rot90();
    let mut v_reg: PartInstance = make_ti_tps_7b84_regulator("TPS7B8433QDCYRQ1").into();
    let mut stretch = Stretch::new();
    let r27 = make_node(&mut stretch, &in_resistor);
    let c43 = make_node(&mut stretch, &input_cap);
    let v8 = make_node(&mut stretch, &v_reg);
    let c39 = make_node(&mut stretch, &output_cap);
    let b1 = make_box(
        &mut stretch,
        FlexDirection::Column,
        JustifyContent::FlexEnd,
        &[r27],
    );
    let b2 = make_box(
        &mut stretch,
        FlexDirection::Column,
        JustifyContent::Center,
        &[c43],
    );
    let b3 = make_box(
        &mut stretch,
        FlexDirection::Column,
        JustifyContent::FlexEnd,
        &[v8],
    );
    let b4 = make_box(
        &mut stretch,
        FlexDirection::Column,
        JustifyContent::Center,
        &[c39],
    );
    let c = make_box(
        &mut stretch,
        FlexDirection::Row,
        JustifyContent::SpaceAround,
        &[b1, b2, b3, b4],
    );
    stretch.compute_layout(c.node, Size::undefined()).unwrap();
    let offset = Point { x: 0.0, y: 0.0 };
    let r27_pos = get_layout_position(&stretch, &[c.node], r27.node, offset).unwrap();
    let c43_pos = get_layout_position(&stretch, &[c.node], c43.node, offset).unwrap();
    let v8_pos = get_layout_position(&stretch, &[c.node], v8.node, offset).unwrap();
    let c39_pos = get_layout_position(&stretch, &[c.node], c39.node, offset).unwrap();
    dbg!(r27_pos);
    dbg!(c43_pos);
    dbg!(v8_pos);
    dbg!(c39_pos);
    in_resistor.schematic_orientation.center = crate::glyph::Point {
        x: (r27_pos.x / PTS_PER_MIL).floor() as i32,
        y: (r27_pos.y / PTS_PER_MIL).floor() as i32,
    };
    input_cap.schematic_orientation.center = crate::glyph::Point {
        x: (c43_pos.x / PTS_PER_MIL).floor() as i32,
        y: (c43_pos.y / PTS_PER_MIL).floor() as i32,
    };
    v_reg.schematic_orientation.center = crate::glyph::Point {
        x: (v8_pos.x / PTS_PER_MIL).floor() as i32,
        y: (v8_pos.y / PTS_PER_MIL).floor() as i32,
    };
    output_cap.schematic_orientation.center = crate::glyph::Point {
        x: (c39_pos.x / PTS_PER_MIL).floor() as i32,
        y: (c39_pos.y / PTS_PER_MIL).floor() as i32,
    };
    let circuit = vec![&in_resistor, &input_cap, &v_reg, &output_cap];
    //write_circuit_to_svg(&circuit, "test_circuit.svg");
}
