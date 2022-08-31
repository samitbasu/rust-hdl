use crate::docs::vcd2svg::renderable::Renderable;
use crate::docs::vcd2svg::time_view::TimeView;
use crate::docs::vcd2svg::timed_value::{changes, SignalType, TimedValue};
use crate::docs::vcd2svg::utils::{line, rect, time_label};
use crate::docs::vcd2svg::vcd_style::VCDStyle;
use svg::node::element::path::Data;
use svg::node::element::{Path, Text, SVG};

pub struct DisplayMetrics {
    pub signal_width: u32,
    pub signal_height: u32,
    pub timescale_height: u32,
    pub tick_half_height: u32,
    pub timescale_midline: u32,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub shim: u32,
    pub label_size: u32,
    pub min_time: u64,
    pub max_time: u64,
    pub style: VCDStyle,
}

impl Default for DisplayMetrics {
    fn default() -> Self {
        Self {
            signal_width: 200,
            signal_height: 20,
            timescale_height: 45,
            tick_half_height: 6,
            timescale_midline: 20,
            canvas_width: 1000,
            canvas_height: 400,
            shim: 5,
            label_size: 10,
            min_time: 40,
            max_time: 102,
            style: VCDStyle::scansion(),
        }
    }
}

impl DisplayMetrics {
    fn compute_major_tick_delta_t(&self) -> u64 {
        let delta_t = (self.max_time - self.min_time) as f64;
        let s = delta_t.log10() - 1.0;
        let x = s.floor();
        let e = s - x;
        let d0 = (e - 0.0).abs();
        let d1 = (e - 2.0_f64.log10()).abs();
        let d2 = (e - 5.0_f64.log10()).abs();
        let value = if d0 <= d1 && d0 <= d2 {
            (10.0_f64.powf(x)) as u64
        } else if d1 <= d0 && d1 <= d2 {
            (2.0_f64 * 10.0_f64.powf(x)) as u64
        } else {
            (5.0_f64 * 10.0_f64.powf(x)) as u64
        };
        value
    }

    fn time_to_pixel(&self, time: u64) -> Option<u32> {
        if time < self.min_time || time > self.max_time {
            None
        } else {
            Some(
                (self.signal_width as f64 + self.pixel_scale() * ((time - self.min_time) as f64))
                    .round() as u32,
            )
        }
    }

    fn major_tick_distance(&self) -> u32 {
        (self.compute_major_tick_delta_t() as f64 * self.pixel_scale()).round() as u32
    }

    fn minor_tick_delta_t(&self) -> f64 {
        self.compute_major_tick_delta_t() as f64 / 5.0
    }

    fn pixel_scale(&self) -> f64 {
        ((self.canvas_width - self.signal_width + 1) as f64)
            / ((self.max_time - self.min_time) as f64)
    }

    fn time_view(&self) -> TimeView {
        TimeView {
            start_time: self.min_time,
            end_time: self.max_time,
            pixel_scale: self.pixel_scale(),
        }
    }

    pub fn major_x0(&self, major: u64) -> Option<u32> {
        let value = self.compute_major_tick_delta_t() * major;
        self.time_to_pixel(value)
    }

    pub fn minor_x0(&self, major: u64, minor: u32) -> Option<u32> {
        let value = self.compute_major_tick_delta_t() * major
            + (self.minor_tick_delta_t() * (minor + 1) as f64) as u64;
        self.time_to_pixel(value)
    }
    pub fn signal_baseline(&self, index: usize) -> u32 {
        self.timescale_height + ((index as u32 + 1) * self.signal_height)
    }
    pub(crate) fn signal_rect(&self) -> Path {
        rect(
            0,
            0,
            self.signal_width,
            self.canvas_height,
            &self.style.signal_label_background_color,
        )
    }

    pub(crate) fn background_rect(&self) -> Path {
        rect(
            0,
            0,
            self.canvas_width,
            self.canvas_height,
            &self.style.background_color,
        )
    }

    pub(crate) fn timescale_header_rect(&self) -> Path {
        rect(
            self.signal_width,
            0,
            self.canvas_width,
            self.timescale_height,
            &self.style.timeline_background_color,
        )
    }

    pub(crate) fn timescale_midline(&self) -> Path {
        line(
            self.signal_width,
            self.timescale_midline,
            self.canvas_width,
            self.timescale_midline,
            &self.style.timeline_line_color,
        )
    }

    fn timescale_major_tick(&self, major: u64) -> Option<Path> {
        dbg!(self.major_x0(major));
        if let Some(x0) = self.major_x0(major) {
            Some(line(
                x0,
                self.timescale_midline - self.tick_half_height,
                x0,
                self.timescale_midline + self.tick_half_height,
                &self.style.timeline_tick_color,
            ))
        } else {
            None
        }
    }

    fn timescale_major_gridline(&self, major: u64) -> Option<Path> {
        if let Some(x0) = self.major_x0(major) {
            if let Some(color) = self.style.grid_lines.as_ref() {
                Some(line(
                    x0,
                    self.timescale_midline,
                    x0,
                    self.canvas_height,
                    color,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn timescale_minor_tick(&self, major: u64, minor: u32) -> Option<Path> {
        if let Some(x1) = self.minor_x0(major, minor) {
            Some(line(
                x1,
                self.timescale_midline,
                x1,
                self.timescale_midline + self.tick_half_height,
                &self.style.timeline_tick_color,
            ))
        } else {
            None
        }
    }

    fn timescale_major_label(&self, major: u64, value: &str) -> Option<Text> {
        if let Some(x0) = self.major_x0(major) {
            let label_width = value.len() as u32 * self.label_size;
            if (x0 - label_width / 2) >= self.signal_width
                && (x0 + label_width / 2) <= self.canvas_width
            {
                let txt = Text::new()
                    .add(svg::node::Text::new(value))
                    .set("x", x0)
                    .set(
                        "y",
                        self.timescale_midline + self.tick_half_height + self.shim,
                    )
                    .set("text-anchor", "middle")
                    .set("font-family", "sans-serif")
                    .set("alignment-baseline", "hanging")
                    .set("fill", self.style.timeline_tick_color.clone())
                    .set("font-size", self.label_size);
                Some(txt)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn signal_label(&self, index: usize, signal: &str) -> Text {
        Text::new()
            .add(svg::node::Text::new(signal))
            .set("x", self.shim)
            .set("y", self.signal_baseline(index) - self.shim)
            .set("text-anchor", "start")
            .set("font-family", "sans-serif")
            .set("alignment-baseline", "bottom")
            .set("font-size", self.label_size)
    }

    pub(crate) fn signal_line(&self, index: usize) -> Path {
        let y0 = self.signal_baseline(index);
        line(
            0,
            y0,
            self.signal_width,
            y0,
            &self.style.timeline_line_color,
        )
    }

    pub(crate) fn timescale(&self, mut document: SVG) -> SVG {
        let first_major_tick =
            (self.min_time as f64 / self.compute_major_tick_delta_t() as f64).floor() as u64;
        dbg!(first_major_tick);
        let last_major_tick =
            (self.max_time as f64 / self.compute_major_tick_delta_t() as f64).ceil() as u64;
        dbg!(last_major_tick);
        let delt = self.compute_major_tick_delta_t();
        for major in first_major_tick..=last_major_tick {
            if let Some(major_line) = self.timescale_major_gridline(major) {
                document = document.add(major_line);
            }
            if let Some(major_tick) = self.timescale_major_tick(major) {
                document = document.add(major_tick);
            }
            if let Some(label) = self.timescale_major_label(major, &time_label(delt * major)) {
                document = document.add(label);
            }
            for minor in 0..4 {
                if let Some(minor_tick) = self.timescale_minor_tick(major, minor) {
                    document = document.add(minor_tick);
                }
            }
        }
        document
    }

    pub(crate) fn horiz_grid_line(&self, index: usize, doc: SVG) -> SVG {
        if self.style.grid_lines.is_none() {
            return doc;
        }
        doc.add(line(
            self.signal_width,
            self.signal_baseline(index),
            self.canvas_width,
            self.signal_baseline(index),
            self.style.grid_lines.as_ref().unwrap(),
        ))
    }

    pub(crate) fn vector_signal_plot<T: SignalType>(
        &self,
        index: usize,
        values: &[TimedValue<T>],
        mut doc: SVG,
    ) -> SVG {
        let values = changes(values);
        let time_view = self.time_view();
        let y0 = self.signal_baseline(index);
        let y_lo = (y0 - self.signal_height + self.shim / 2) as f64;
        let y_hi = (y0 - self.shim / 2) as f64;
        let flip = |x| if x == y_lo { y_hi } else { y_lo };
        let shim = 1.0;
        let x0 = self.signal_width as f64;
        let mut data_low = Data::new().move_to((x0, y_lo));
        let mut data_high = Data::new().move_to((x0, y_hi));
        let mut last_y1 = y_lo as f64;
        for value in time_view
            .intervals(&values)
            .iter()
            .filter(|x| !x.is_empty())
        {
            let x1 = x0 + value.start_x;
            let y1 = flip(last_y1);
            data_low = data_low.line_to((x1 - shim, last_y1));
            data_high = data_high.line_to((x1 - shim, flip(last_y1)));
            last_y1 = y1;
            data_low = data_low.line_to((x1 + shim, y1));
            data_high = data_high.line_to((x1 + shim, flip(y1)));
            doc = doc.add(
                Text::new()
                    .add(svg::node::Text::new(value.label.to_string()))
                    .set("x", x1 + 2.0 * shim)
                    .set(
                        "y",
                        self.signal_baseline(index) - self.signal_height / 2 + 1,
                    )
                    .set("text-anchor", "start")
                    .set("font-family", "monospace")
                    //                    .set("font-family", "sans-serif")
                    .set("alignment-baseline", "middle")
                    .set("font-size", self.label_size - 2)
                    .set("fill", "white"),
            );
        }
        if let Some(x) = self.time_to_pixel(self.max_time) {
            data_low = data_low.line_to((x, last_y1));
            data_high = data_high.line_to((x, flip(last_y1)));
        }
        let doc = doc
            .add(
                Path::new()
                    .set("fill", "none")
                    .set("stroke", self.style.trace_color.clone())
                    .set("stroke-width", 0.75)
                    .set("d", data_low),
            )
            .add(
                Path::new()
                    .set("fill", "none")
                    .set("stroke", self.style.trace_color.clone())
                    .set("stroke-width", 0.75)
                    .set("d", data_high),
            );
        doc
    }
    pub(crate) fn bit_signal_plot(&self, index: usize, values: &[TimedValue<bool>]) -> Path {
        let values = changes(values);
        let y0 = self.signal_baseline(index) - self.shim / 2;
        let y1 = y0 - self.signal_height + self.shim;
        let x0 = self.signal_width;
        let mut data = Data::new().move_to((x0, y0));
        let mut last_y1 = y0;
        for value in values {
            if let Some(x1) = self.time_to_pixel(value.time) {
                let y = if value.value { y1 } else { y0 };
                data = data.line_to((x1, last_y1));
                last_y1 = y;
                data = data.line_to((x1, y));
            }
        }
        if let Some(x) = self.time_to_pixel(self.max_time) {
            data = data.line_to((x, last_y1));
        }
        Path::new()
            .set("fill", "none")
            .set("stroke", self.style.trace_color.clone())
            .set("stroke-width", 0.75)
            .set("d", data)
    }
}
