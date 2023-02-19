// Covers the ECP5 via nextpnr, not via Diamond
use rust_hdl_private_core::prelude::*;
use std::collections::HashMap;

use super::map_signal_type_to_lattice_string;

#[derive(Default)]
struct LPFGenerator {
    path: NamedPath,
    namespace: NamedPath,
    lpf: Vec<String>,
    names: HashMap<usize, String>,
}

impl Probe for LPFGenerator {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        let _top_level = self.path.to_string();
        self.path.push(name);
        self.namespace.reset();
    }
    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.namespace.push(name);
    }
    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        let namespace = self.namespace.flat("$");
        let name = if namespace.is_empty() {
            name.to_owned()
        } else {
            format!("{}${}", namespace, name)
        };
        for pin in &signal.constraints() {
            self.names.insert(signal.id(), name.clone());
            let prefix = if signal.bits() == 1 {
                format!("{}", name)
            } else {
                format!("{}[{}]", name, pin.index)
            };
            match &pin.constraint {
                Constraint::Location(l) => {
                    self.lpf
                        .push(format!("LOCATE COMP \"{}\" SITE \"{}\"", prefix, l));
                }
                Constraint::Kind(k) => {
                    let name = map_signal_type_to_lattice_string(k);
                    self.lpf
                        .push(format!("IOBUF PORT \"{}\" IO_TYPE={}", prefix, name))
                }
                Constraint::Timing(t) => {
                    let timing = match t {
                        Timing::Periodic(p) => {
                            format!(
                                "FREQUENCY PORT \"{prefix}\" {freq} MHz",
                                prefix = prefix,
                                freq =
                                    ((1000.0 / p.period_nanoseconds) * 10000.0).round() / 10000.0
                            )
                        }
                        Timing::Custom(c) => c.to_string(),
                        _ => unimplemented!("Unknown timing constraint for ECP5 generation"),
                    };
                    if !timing.is_empty() {
                        self.lpf.push(timing);
                    }
                }
                Constraint::Custom(s) => self.lpf.push(s.clone()),
                Constraint::Slew(k) => {
                    let tag = match k {
                        SlewType::Fast => "FAST",
                        SlewType::Normal => "SLOW",
                    };
                    self.lpf
                        .push(format!("IOBUF PORT \"{}\" SLEWRATE={}", prefix, tag));
                }
            }
        }
    }
    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.namespace.pop();
    }
    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }
}

pub fn generate_lpf<U: Block>(uut: &U) -> String {
    let mut lpf = LPFGenerator::default();
    uut.accept("top", &mut lpf);
    let mut lpf_uniq = vec![];
    lpf_uniq.push("BLOCK RESETPATHS".to_string());
    lpf_uniq.push("BLOCK ASYNCPATHS".to_string());
    for line in lpf.lpf {
        if !lpf_uniq.contains(&line) {
            lpf_uniq.push(line);
        }
    }
    lpf_uniq.join(";\n") + ";\n"
}
