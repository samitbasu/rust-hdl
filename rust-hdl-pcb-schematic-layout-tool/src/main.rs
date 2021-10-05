use druid::{
    kurbo::Line, Affine, AppLauncher, BoxConstraints, Color, Data, Env, Event, EventCtx,
    FontDescriptor, FontFamily, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, RenderContext, Size,
    TextAlignment, TextLayout, UpdateCtx, Widget, WidgetId, WindowDesc,
};
use rust_hdl_pcb::adc::make_ads868x;
use rust_hdl_pcb_core::prelude::*;
use std::sync::Arc;

#[derive(Data, Clone)]
struct Schematic {
    circuit: Arc<Circuit>,
    layout: Arc<SchematicLayout>,
    center: (f64, f64),
    cursor: (f64, f64),
    size: Size,
    scale: f64,
}

struct SchematicViewer;

impl Widget<Schematic> for SchematicViewer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Schematic, _env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                ctx.set_active(true);
                data.cursor = (mouse.pos.x, mouse.pos.y);
            }
            Event::MouseUp(mouse) => {
                ctx.set_active(false);
            }
            Event::MouseMove(mouse) => {
                if ctx.is_active() {
                    data.center.0 += (mouse.pos.x - data.cursor.0);
                    data.center.1 += (mouse.pos.y - data.cursor.1);
                    data.cursor = (mouse.pos.x, mouse.pos.y);
                    ctx.request_paint();
                }
            }
            Event::Wheel(mouse) => {
                data.scale *= (1.0 - mouse.wheel_delta.y / 100.0).max(0.9).min(1.1);
                ctx.request_paint();
            }
            Event::Zoom(x) => {
                //dbg!(x);
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Schematic,
        env: &Env,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Schematic, data: &Schematic, env: &Env) {}

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Schematic,
        env: &Env,
    ) -> Size {
        if bc.is_width_bounded() && bc.is_height_bounded() {
            bc.max()
        } else {
            let size = Size::new(100.0, 100.0);
            bc.constrain(size)
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Schematic, env: &Env) {
        let size = ctx.size();
        let rect = size.to_rect();
        // Clear the canvas
        ctx.fill(rect, &Color::from_hex_str("FFFCF8").unwrap());
        //dbg!(data.cursor);
        let mc = (-data.center.0, -data.center.1);
        ctx.transform(Affine::translate(data.center));
        ctx.transform(Affine::scale(data.scale));
        ctx.transform(Affine::translate(mc));
        ctx.transform(Affine::scale_non_uniform(1.0, -1.0));
        for instance in &data.circuit.nodes {
            let part = get_details_from_instance(instance, &data.layout);
            let schematic_orientation = data.layout.part(&instance.id);
            ctx.with_save(|ctx| {
                if schematic_orientation.rotation == SchematicRotation::Vertical {
                    ctx.transform(Affine::translate((
                        schematic_orientation.center.0 as f64,
                        schematic_orientation.center.1 as f64,
                    )));
                    ctx.transform(Affine::rotate(std::f64::consts::PI / 2.0));
                } else {
                    ctx.transform(Affine::translate((
                        schematic_orientation.center.0 as f64,
                        schematic_orientation.center.1 as f64,
                    )));
                }
                for path in &part.outline {
                    render_glyph(ctx, path, part.hide_part_outline, env);
                }
                for (num, pin) in &part.pins {
                    render_pin(ctx, num, pin, &part.outline, part.hide_pin_designators, env);
                }
            });
        }
    }
}

fn map_point(p: &Point) -> druid::Point {
    druid::Point {
        x: p.x as f64,
        y: p.y as f64,
    }
}

const EM: i32 = 85;

fn render_pin(
    ctx: &mut PaintCtx,
    num: &u64,
    pin: &EPin,
    outline: &[Glyph],
    hide_pin_designators: bool,
    env: &Env,
) {
    if outline.len() == 0 {
        return;
    }
    if let Glyph::OutlineRect(r) = &outline[0] {
        if r.is_empty() {
            return;
        }
        match pin.location.edge {
            EdgeLocation::North => {
                if !hide_pin_designators {
                    render_text(
                        ctx,
                        &format!("{}", num),
                        "000000",
                        TextJustification::BottomLeft,
                        80.0,
                        Point {
                            x: pin.location.offset - 15,
                            y: r.p0.y.max(r.p1.y) + 50,
                        },
                        env,
                        true,
                    );
                }
                render_text(
                    ctx,
                    &pin.name,
                    "000000",
                    TextJustification::MiddleRight,
                    80.0,
                    Point {
                        x: pin.location.offset,
                        y: r.p1.y - EM,
                    },
                    env,
                    true,
                );
                render_line(
                    ctx,
                    Point {
                        x: pin.location.offset,
                        y: r.p1.y,
                    },
                    Point {
                        x: pin.location.offset,
                        y: r.p1.y + PIN_LENGTH,
                    },
                    "000000",
                    10.0,
                );
            }
            EdgeLocation::West => {
                if !hide_pin_designators {
                    render_text(
                        ctx,
                        &format!("{}", num),
                        "000000",
                        TextJustification::BottomRight,
                        80.0,
                        Point {
                            x: r.p0.x - 85,
                            y: pin.location.offset + 15,
                        },
                        env,
                        false,
                    );
                }
                render_text(
                    ctx,
                    &pin.name,
                    "000000",
                    TextJustification::MiddleLeft,
                    80.0,
                    Point {
                        x: r.p0.x + EM,
                        y: pin.location.offset,
                    },
                    env,
                    false,
                );
                render_line(
                    ctx,
                    Point {
                        x: r.p0.x,
                        y: pin.location.offset,
                    },
                    Point {
                        x: r.p0.x - PIN_LENGTH,
                        y: pin.location.offset,
                    },
                    "000000",
                    10.0,
                );
            }
            EdgeLocation::South => {
                if !hide_pin_designators {
                    render_text(
                        ctx,
                        &format!("{}", num),
                        "000000",
                        TextJustification::BottomRight,
                        80.0,
                        Point {
                            x: pin.location.offset - 15,
                            y: r.p0.y.min(r.p1.y) - 50,
                        },
                        env,
                        true,
                    );
                }
                render_text(
                    ctx,
                    &pin.name,
                    "000000",
                    TextJustification::MiddleLeft,
                    80.0,
                    Point {
                        x: pin.location.offset,
                        y: r.p0.y + EM,
                    },
                    env,
                    true,
                );
                render_line(
                    ctx,
                    Point {
                        x: pin.location.offset,
                        y: r.p0.y,
                    },
                    Point {
                        x: pin.location.offset,
                        y: r.p0.y - PIN_LENGTH,
                    },
                    "000000",
                    10.0,
                );
            }
            EdgeLocation::East => {
                if !hide_pin_designators {
                    render_text(
                        ctx,
                        &format!("{}", num),
                        "000000",
                        TextJustification::BottomLeft,
                        80.0,
                        Point {
                            x: r.p1.x + 85,
                            y: pin.location.offset + 15,
                        },
                        env,
                        false,
                    );
                }
                render_text(
                    ctx,
                    &pin.name,
                    "000000",
                    TextJustification::MiddleRight,
                    80.0,
                    Point {
                        x: r.p1.x - EM,
                        y: pin.location.offset,
                    },
                    env,
                    false,
                );
                render_line(
                    ctx,
                    Point {
                        x: r.p1.x,
                        y: pin.location.offset,
                    },
                    Point {
                        x: r.p1.x + PIN_LENGTH,
                        y: pin.location.offset,
                    },
                    "000000",
                    10.0,
                );
            }
        }
    }
}

fn render_line(ctx: &mut PaintCtx, start: Point, end: Point, color: &str, width: f64) {
    let line = druid::kurbo::Line {
        p0: map_point(&start),
        p1: map_point(&end),
    };
    let stroke_color = Color::from_hex_str(color).unwrap();
    ctx.stroke(line, &stroke_color, width);
}

fn render_text(
    ctx: &mut PaintCtx,
    t: &str,
    color: &str,
    justify: TextJustification,
    size: f64,
    at: Point,
    env: &Env,
    is_vert: bool,
) {
    let mut layout = TextLayout::<String>::from_text(t);
    match justify {
        TextJustification::BottomLeft
        | TextJustification::TopLeft
        | TextJustification::MiddleLeft => layout.set_text_alignment(TextAlignment::Start),
        TextJustification::BottomRight
        | TextJustification::TopRight
        | TextJustification::MiddleRight => layout.set_text_alignment(TextAlignment::End),
    }
    let stroke_color = Color::from_hex_str(color).unwrap();
    layout.set_text_color(stroke_color);
    layout.set_font(FontDescriptor::new(FontFamily::MONOSPACE).with_size(size));
    layout.rebuild_if_needed(ctx.text(), env);
    let baseline = layout.layout_metrics().size.height;
    let width = layout.layout_metrics().size.width;
    ctx.with_save(|ctx| {
        ctx.transform(Affine::scale_non_uniform(1.0, -1.0));
        if is_vert {
            ctx.transform(Affine::translate((at.x as f64, -at.y as f64)));
            ctx.transform(Affine::rotate(-std::f64::consts::PI / 2.0));
            ctx.transform(Affine::translate((-at.x as f64, at.y as f64)));
        }
        match justify {
            TextJustification::TopRight => ctx.transform(Affine::translate((-width, 0.0))),
            TextJustification::TopLeft => {}
            TextJustification::MiddleRight => {
                ctx.transform(Affine::translate((-width, 0.0)));
                ctx.transform(Affine::translate((0.0, -baseline / 2.0)))
            }
            TextJustification::MiddleLeft => {
                ctx.transform(Affine::translate((0.0, -baseline / 2.0)))
            }
            TextJustification::BottomLeft => ctx.transform(Affine::translate((0.0, -baseline))),
            TextJustification::BottomRight => {
                ctx.transform(Affine::translate((-width, 0.0)));
                ctx.transform(Affine::translate((0.0, -baseline)))
            }
        }
        layout.draw(
            ctx,
            druid::Point {
                x: at.x as f64,
                y: -at.y as f64,
            },
        );
    });
}

fn render_glyph(ctx: &mut PaintCtx, g: &Glyph, hide_outline: bool, env: &Env) {
    match g {
        Glyph::OutlineRect(r) => {
            if !hide_outline {
                let rect =
                    druid::Rect::new(r.p0.x as f64, r.p0.y as f64, r.p1.x as f64, r.p1.y as f64);
                let stroke_color = Color::from_hex_str("AE5E46").unwrap();
                let fill_color = Color::from_hex_str("FFFDB0").unwrap();
                ctx.stroke(rect, &stroke_color, 5.0);
                ctx.fill(rect, &fill_color);
            }
        }
        Glyph::Line(l) => {
            render_line(ctx, l.p0, l.p1, "0433FF", 10.0);
        }
        Glyph::Text(t) => {
            render_text(ctx, &t.text, "0433FF", t.justify, 80.0, t.p0, env, false);
        }
        Glyph::Arc(_) => {}
        Glyph::Circle(_) => {}
    }
}

fn make_root() -> impl Widget<Schematic> {
    SchematicViewer {}
}

fn main() {
    let window = WindowDesc::new(make_root())
        .window_size(Size {
            width: 800.0,
            height: 800.0,
        })
        .resizable(true)
        .title("Schematic Viewer");
    let (mut circuit, mut layout) = rust_hdl_pcb::schematic_manual_layout::test_ldo_circuit();
    circuit
        .nodes
        .push(make_ads868x("ADS8681IPW").instance("adc"));
    layout.set_part("adc", orient().center(4000, 4000));
    AppLauncher::with_window(window)
        .log_to_console()
        .launch(Schematic {
            circuit: Arc::new(circuit),
            layout: Arc::new(layout),
            center: (0.0, 0.0),
            cursor: (0.0, 0.0),
            size: Size {
                width: 800.0,
                height: 800.0,
            },
            scale: 0.2,
        })
        .expect("launch failed");
}
