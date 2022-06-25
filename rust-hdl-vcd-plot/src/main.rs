use itertools::Itertools;
use plotters::prelude::*;
use plotters::style::RelativeSize;
use rand::random;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let root = BitMapBackend::new("demo.png", (640, 480)).into_drawing_area();
    let root = SVGBackend::new("demo.svg", (640, 480)).into_drawing_area();
    let signals = [
        "time.clock.d",
        "foo.bar.input.size.q",
        "blah.blah.d.next",
    ];
    let font = ("monospace", 16).into_font();
    let style = TextStyle::from(font);
    let mut signal_height = 0;
    for signal in signals {
        let signal_width =
            root.estimate_text_size(signal, &style)?;
        println!("estimated text size: {:?}", signal_width);
        signal_height = signal_width.1;

    }
    let row_height = signal_height * 2;
    let xsplit = [100];
    let ysplit = (0..30).map(|x| x * row_height).collect::<Vec<_>>();

    let parts = root.split_by_breakpoints(&xsplit, &ysplit);
    for (part_label, part_trace) in
    parts.iter().tuples::<(&DrawingArea<_, _>, &DrawingArea<_, _>)>() {
        part_trace.fill(&RGBColor(rand::random(), rand::random(), rand::random()))?;
        part_label.fill(&RGBColor(233, 233, 233))?;
        part_label.draw_text("foo.bar.input.size.q", &style, (0, (signal_height/2) as i32))?;
    }

    //root.fill(&WHITE)?;
    //129,223,197
    if false {
        let (labels, root) = root.split_horizontally(RelativeSize::Width(0.3));
        root.fill(&RGBColor(40, 40, 40))?;
        let mut chart = ChartBuilder::on(&root)
            .caption("y=x^2", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
                &RGBColor(129, 223, 197),
            ))?
            .label("y = x^2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
    }

    Ok(())
}