use num_bigint::BigInt;
use std::clone::Clone;
use std::collections::HashMap;
use std::fs::File;
use std::iter::Iterator;
use std::string::ToString;
use svg::Document;
use vcd::IdCode;
use crate::{DisplayMetrics, TimedValue};
use crate::utils::{value_to_bigint, value_to_bool};

type StringTrace = Vec<TimedValue<String>>;
type VectorTrace = Vec<TimedValue<BigInt>>;
type BinaryTrace = Vec<TimedValue<bool>>;

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
        let mut document = Document::new()
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
                return anyhow::bail!("Unable to find signal {} in the trace...", details.1);
            }
        }
        Ok(document)
    }
}


