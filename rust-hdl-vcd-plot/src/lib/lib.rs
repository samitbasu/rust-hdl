use svg::node::element::{Path, SVG, Text};
use svg::node::element::path::Data;
use num_bigint::{BigInt, Sign};
use std::fs::File;
use std::collections::HashMap;
use vcd::{IdCode, Value};
use substring::Substring;
use svg::Document;
use interval::Interval;
use time_view::TimeView;
use timed_value::TimedValue;
use crate::display_metrics::DisplayMetrics;
use crate::vcd_style::VCDStyle;

pub mod vcd_style;
pub mod display_metrics;
pub mod trace_collection;
mod timed_value;
mod interval;
mod time_view;
mod utils;
mod renderable;


fn make_clock(period: u64) -> Vec<TimedValue<bool>> {
    (0..1000)
        .map(|x| TimedValue {
            time: period * x,
            value: x % 2 == 0,
        })
        .collect()
}

fn make_linear_counter(period: u64) -> Vec<TimedValue<BigInt>> {
    (0..1000)
        .map(|x| TimedValue {
            time: period * x,
            value: x.into(),
        })
        .collect()
}

fn make_counter(period: u64) -> Vec<TimedValue<BigInt>> {
    (0..1000)
        .map(|x| TimedValue {
            time: period * x,
            value: (x * 10000 + x * x * 100000).into(),
        })
        .collect()
}

fn time_label(val: u64) -> String {
    if val < 1000 {
        format!("{}ps", val)
    } else if val < 1_000_000 {
        format!("{}ns", val / 1_000)
    } else {
        format!("{}ms", val / 1_000_000)
    }
}
