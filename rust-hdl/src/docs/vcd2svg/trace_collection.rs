use crate::docs::vcd2svg::display_metrics::DisplayMetrics;
use crate::docs::vcd2svg::symbols;
use crate::docs::vcd2svg::text_frame::TextFrame;
use crate::docs::vcd2svg::timed_value::{changes, SignalType, TimedValue};
use crate::docs::vcd2svg::utils::{time_label, value_to_bigint, value_to_bool};
use num_bigint::BigInt;
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::iter::Iterator;
use std::string::ToString;
use svg::Document;
use vcd::IdCode;

type StringTrace = Vec<TimedValue<String>>;
type VectorTrace = Vec<TimedValue<BigInt>>;
type BinaryTrace = Vec<TimedValue<bool>>;

// Lifted from dwfv/src/tui/waveform.rs
#[derive(Clone, PartialEq, Eq)]
enum WaveformElement {
    Low,
    High,
    Value(String),
    Transition,
    RisingEdge,
    FallingEdge,
    Invalid,
    LowDensity,
    MediumDensity,
    HighDensity,
}

impl WaveformElement {
    pub fn to_symbols(&self) -> (char, char, char) {
        match self {
            WaveformElement::Low => (symbols::BLANK, symbols::BLANK, symbols::HORIZONTAL),
            WaveformElement::High => (symbols::HORIZONTAL, symbols::BLANK, symbols::BLANK),
            WaveformElement::Value(_) => (symbols::HORIZONTAL, symbols::BLANK, symbols::HORIZONTAL),
            WaveformElement::RisingEdge => {
                (symbols::TOP_LEFT, symbols::VERTICAL, symbols::BOTTOM_RIGHT)
            }
            WaveformElement::FallingEdge => {
                (symbols::TOP_RIGHT, symbols::VERTICAL, symbols::BOTTOM_LEFT)
            }
            WaveformElement::Transition => (
                symbols::HORIZONTAL_DOWN,
                symbols::VERTICAL,
                symbols::HORIZONTAL_UP,
            ),
            WaveformElement::Invalid => (symbols::FULL_LOWER, symbols::FULL, symbols::FULL_UPPER),
            WaveformElement::LowDensity => {
                (symbols::LIGHT_LOWER, symbols::LIGHT, symbols::LIGHT_UPPER)
            }
            WaveformElement::MediumDensity => (
                symbols::MEDIUM_LOWER,
                symbols::MEDIUM,
                symbols::MEDIUM_UPPER,
            ),
            WaveformElement::HighDensity => {
                (symbols::FULL_LOWER, symbols::FULL, symbols::FULL_UPPER)
            }
        }
    }
}

#[derive(Clone, Default, Debug)]
struct SignalBin<T: SignalType> {
    start_time: u64,
    end_time: u64,
    values: Vec<TimedValue<T>>,
    before: Option<T>,
    after: Option<T>,
    changes: usize,
}

fn bin_edges(first_time: u64, last_time: u64, num_bins: usize) -> Vec<(u64, u64)> {
    let delta = (last_time - first_time) as f64 / (num_bins as f64);
    (0..num_bins)
        .map(|x| {
            (
                (first_time + ((x as f64) * delta) as u64),
                (first_time + ((x as f64 + 1.0) * delta) as u64),
            )
        })
        .collect()
}

fn bin_trace<T: SignalType>(trace: &Vec<TimedValue<T>>, timing: &BinTimes) -> Vec<SignalBin<T>> {
    let num_bins = timing.count();
    let mut bins: Vec<SignalBin<T>> = vec![Default::default(); num_bins];
    trace.iter().for_each(|val| {
        if let Some(bin) = timing.to_bin(val.time) {
            bins[bin].values.push(val.clone());
        }
    });
    // The before value for bin N is the last value in bin N - 1 - but bins might be empty
    let mut signal_value = None;
    for ndx in 0..num_bins {
        let mut bin = &mut bins[ndx];
        (bin.start_time, bin.end_time) = timing.bracket(ndx);
        bin.before = signal_value.clone();
        signal_value = if let Some(x) = bin.values.last() {
            Some(x.value.clone())
        } else {
            signal_value
        };
        bin.after = signal_value.clone();
        bin.changes = bin.values.len();
    }
    bins
}

pub struct TraceCollection {
    pub signal_names: Vec<(IdCode, String)>,
    pub string_valued: HashMap<IdCode, StringTrace>,
    pub vector_valued: HashMap<IdCode, VectorTrace>,
    pub scalar_valued: HashMap<IdCode, BinaryTrace>,
}

impl TraceCollection {
    pub fn parse(signals: &[&str], mut file: File) -> anyhow::Result<Self> {
        let mut parser = vcd::Parser::new(&mut file);
        let header = parser.parse_header()?;
        let mut string_valued = HashMap::new();
        let mut vector_valued = HashMap::new();
        let mut scalar_valued = HashMap::new();
        let mut signal_names = Vec::new();
        for signal in signals {
            let path = signal.split(".").collect::<Vec<_>>();
            let sig = header
                .find_var(&path)
                .ok_or_else(|| anyhow::Error::msg(format!("cannot resolve signal {}", signal)))?;
            if sig.size == 0 {
                string_valued.insert(sig.code, StringTrace::new());
            } else if sig.size == 1 {
                scalar_valued.insert(sig.code, BinaryTrace::new());
            } else {
                vector_valued.insert(sig.code, VectorTrace::new());
            }
            signal_names.push((sig.code, signal.to_string()));
        }
        let mut timestamp = 0_u64;
        for command_result in parser {
            let command = command_result?;
            match command {
                vcd::Command::Timestamp(x) => {
                    timestamp = x;
                }
                vcd::Command::ChangeScalar(i, v) => {
                    if let Some(s) = scalar_valued.get_mut(&i) {
                        s.push(TimedValue {
                            time: timestamp,
                            value: value_to_bool(&v)?,
                        })
                    }
                }
                vcd::Command::ChangeVector(i, v) => {
                    if let Some(s) = vector_valued.get_mut(&i) {
                        s.push(TimedValue {
                            time: timestamp,
                            value: value_to_bigint(&v)?,
                        })
                    }
                }
                vcd::Command::ChangeString(i, v) => {
                    if let Some(s) = string_valued.get_mut(&i) {
                        s.push(TimedValue {
                            time: timestamp,
                            value: v.clone(),
                        })
                    }
                }
                _ => {}
            }
        }
        Ok(Self {
            signal_names,
            string_valued,
            vector_valued,
            scalar_valued,
        })
    }

    pub fn as_svg(&self, metrics: &DisplayMetrics) -> anyhow::Result<Document> {
        let document = Document::new()
            .set(
                "viewBox",
                (0, 0, metrics.canvas_width, metrics.canvas_height),
            )
            .add(metrics.background_rect());

        // Paint the timescale rectangle
        let mut document = document
            .add(metrics.signal_rect())
            .add(metrics.timescale_header_rect())
            .add(metrics.timescale_midline());

        document = metrics.timescale(document);

        for (index, details) in self.signal_names.iter().enumerate() {
            document = document
                .add(metrics.signal_label(index, &details.1))
                .add(metrics.signal_line(index));
            document = metrics.horiz_grid_line(index, document);
            if let Some(s) = self.scalar_valued.get(&details.0) {
                document = document.add(metrics.bit_signal_plot(index, s));
            } else if let Some(s) = self.vector_valued.get(&details.0) {
                document = metrics.vector_signal_plot(index, s, document);
            } else if let Some(s) = self.string_valued.get(&details.0) {
                document = metrics.vector_signal_plot(index, s, document);
            } else {
                anyhow::bail!("Unable to find signal {} in the trace...", details.1)
            }
        }
        Ok(document)
    }
    pub fn as_string(
        &self,
        first_time: u64,
        last_time: u64,
        max_columns: usize,
    ) -> anyhow::Result<String> {
        let mut frame = TextFrame::new(max_columns);
        let sig_columns = self
            .signal_names
            .iter()
            .map(|(_, x)| x.len())
            .max()
            .unwrap_or(0)
            .max(4);
        let num_bins = max_columns - sig_columns - 1;
        let timing = BinTimes::new(first_time, last_time, num_bins);
        draw_symbols(
            &render_multibit(&clock_trace(&timing)),
            &mut frame,
            0,
            sig_columns + 1,
        );
        frame.write(1, sig_columns - 4, "time");
        for (index, details) in self.signal_names.iter().enumerate() {
            let index = index + 1;
            frame.write(index * 3 + 1, sig_columns - details.1.len(), &details.1);
            if let Some(s) = self.scalar_valued.get(&details.0) {
                let bins = bin_trace(&changes(s), &timing);
                draw_symbols(&render_bool(&bins), &mut frame, index * 3, sig_columns + 1);
            } else if let Some(s) = self.vector_valued.get(&details.0) {
                let bins = bin_trace(&changes(s), &timing);
                draw_symbols(
                    &render_multibit(&bins),
                    &mut frame,
                    index * 3,
                    sig_columns + 1,
                );
            } else if let Some(s) = self.string_valued.get(&details.0) {
                let bins = bin_trace(&changes(s), &timing);
                draw_symbols(
                    &render_multibit(&bins),
                    &mut frame,
                    index * 3,
                    sig_columns + 1,
                );
            } else {
                anyhow::bail!("Unable to find signal {} in the trace...", details.1)
            }
        }
        Ok(frame.to_string())
    }
}

struct BinTimes {
    first_time: u64,
    last_time: u64,
    num_bins: usize,
    delta: f64,
}

impl BinTimes {
    fn new(first_time: u64, last_time: u64, num_bins: usize) -> Self {
        Self {
            first_time,
            last_time,
            num_bins,
            delta: (last_time - first_time) as f64 / (num_bins as f64),
        }
    }
    fn bracket(&self, ndx: usize) -> (u64, u64) {
        (
            (self.first_time as f64 + self.delta * (ndx as f64)).floor() as u64,
            (self.first_time as f64 + self.delta * (ndx as f64 + 1.0)).floor() as u64,
        )
    }
    fn to_bin(&self, time: u64) -> Option<usize> {
        let bin = ((time as f64 - self.first_time as f64) / self.delta).floor() as i32;
        if bin >= 0 && bin < self.num_bins as i32 {
            Some(bin as usize)
        } else {
            None
        }
    }
    fn count(&self) -> usize {
        self.num_bins
    }
    fn first(&self) -> u64 {
        self.first_time
    }
    fn last(&self) -> u64 {
        self.last_time
    }
    fn span(&self) -> u64 {
        self.last_time - self.first_time
    }
}

#[test]
fn test_bin_times() {
    let times = BinTimes::new(0, 250, 80);
    (0..=times.count()).for_each(|x| println!("{:?}", times.bracket(x)));
}

fn clock_trace(timing: &BinTimes) -> Vec<SignalBin<String>> {
    let major_tick_delta = compute_major_tick_delta_t(timing.span());
    let first_index = (timing.first() as f64 / major_tick_delta as f64).floor() as u64;
    let last_index = (timing.last() as f64 / major_tick_delta as f64).ceil() as u64;
    let clock = (first_index..=last_index)
        .map(|ndx| ndx * major_tick_delta)
        .map(|x| TimedValue {
            time: x,
            value: time_label(x),
        })
        .collect::<Vec<_>>();
    bin_trace(&changes(&clock), timing)
}

fn compute_major_tick_delta_t(span: u64) -> u64 {
    let delta_t = span as f64;
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

fn render_multibit<T: SignalType>(bins: &[SignalBin<T>]) -> Vec<WaveformElement> {
    bins.iter()
        .map(|bin| {
            if bin.after.is_none() {
                WaveformElement::Invalid
            } else if bin.changes == 0 || (bin.changes == 1 && bin.before.is_none()) {
                if let Some(b) = bin.before.as_ref() {
                    WaveformElement::Value(b.render())
                } else {
                    WaveformElement::Value("?".to_string())
                }
            } else if bin.changes == 1 {
                WaveformElement::Transition
            } else if bin.changes <= 3 {
                WaveformElement::LowDensity
            } else if bin.changes <= 10 {
                WaveformElement::MediumDensity
            } else {
                WaveformElement::HighDensity
            }
        })
        .collect()
}

fn render_bool(bins: &[SignalBin<bool>]) -> Vec<WaveformElement> {
    bins.iter()
        .map(|bin| {
            if bin.after.is_none() {
                WaveformElement::Invalid
            } else if bin.changes == 0 || (bin.changes == 1 && bin.before.is_none()) {
                if !bin.after.unwrap() {
                    WaveformElement::Low
                } else {
                    WaveformElement::High
                }
            } else if bin.changes == 1 {
                if let Some(x) = bin.before {
                    if !x {
                        WaveformElement::RisingEdge
                    } else {
                        WaveformElement::FallingEdge
                    }
                } else {
                    WaveformElement::Invalid
                }
            } else if bin.changes <= 3 {
                WaveformElement::LowDensity
            } else if bin.changes <= 10 {
                WaveformElement::MediumDensity
            } else {
                WaveformElement::HighDensity
            }
        })
        .collect()
}

fn draw_symbols(symbols: &[WaveformElement], frame: &mut TextFrame, row: usize, start_col: usize) {
    symbols.iter().enumerate().for_each(|(ndx, element)| {
        let symbol = element.to_symbols();
        frame.put(row, start_col + ndx, symbol.0);
        frame.put(row + 1, start_col + ndx, symbol.1);
        frame.put(row + 2, start_col + ndx, symbol.2);
    });
    // From dwfv/src/tui/waveform.rs
    let mut elmts = symbols[1..].iter().enumerate();
    loop {
        let mut free_space = 0;
        let mut value = "";
        let mut elmt = elmts.next();
        let offset = if let Some((i, _)) = elmt { i } else { break };

        while let Some((_, WaveformElement::Value(v))) = elmt {
            free_space += 1;
            value = v;
            elmt = elmts.next();
        }

        for (i, c) in value.chars().enumerate() {
            if i >= free_space {
                break;
            }

            let r = &c.to_string();
            let symbol = if i >= free_space - 1 && i + 1 < value.len() {
                "…"
            } else {
                r
            };

            frame.write(row + 1, start_col + (offset + i + 1), symbol);
        }
    }
}
