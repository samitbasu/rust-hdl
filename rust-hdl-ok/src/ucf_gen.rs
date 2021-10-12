use rust_hdl_core::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
struct UCFGenerator {
    path: NamedPath,
    namespace: NamedPath,
    ucf: Vec<String>,
    names: HashMap<usize, String>,
}

pub fn collect_xrefs(txt: &[String]) -> Vec<(String, String)> {
    let xref = regex::Regex::new(r#"!(\d*)!"#).unwrap();
    let mut ret = vec![];
    for line in txt {
        for capture in xref.captures_iter(line) {
            if let Some(t) = capture.get(0) {
                if let Some(p) = capture.get(1) {
                    let capture = (t.as_str().to_string(), p.as_str().to_string());
                    if !ret.contains(&capture) {
                        ret.push(capture);
                    }
                }
            }
        }
    }
    ret
}

pub fn substitute_refs(
    txt: &[String],
    xrefs: &[(String, String)],
    name_map: &HashMap<usize, String>,
) -> Vec<String> {
    let mut ret = vec![];
    for line in txt {
        let mut line = line.clone();
        for (from, to) in xrefs {
            let index = to.parse::<usize>().unwrap();
            if let Some(name) = name_map.get(&index) {
                line = line.replace(from, name);
            }
        }
        ret.push(line);
    }
    ret
}

pub fn map_signal_type_to_xilinx_string(k: &SignalType) -> &str {
    match k {
        SignalType::LowVoltageCMOS_1v8 => "LVCMOS18",
        SignalType::LowVoltageCMOS_3v3 => "LVCMOS33",
        SignalType::StubSeriesTerminatedLogic_II => "SSTL18_II",
        SignalType::DifferentialStubSeriesTerminatedLogic_II => "DIFF_SSTL18_II",
        SignalType::StubSeriesTerminatedLogic_II_No_Termination => "SSTL18_II | IN_TERM=NONE",
        SignalType::DifferentialStubSeriesTerminatedLogic_II_No_Termination => {
            "DIFF_SSTL18_II | IN_TERM=NONE"
        }
        SignalType::Custom(c) => c,
        SignalType::LowVoltageDifferentialSignal_2v5 => "LVDS_25",
        SignalType::StubSeriesTerminatedLogic_1v5 => "SSTL15",
        SignalType::LowVoltageCMOS_1v5 => "LVCMOS15",
        SignalType::DifferentialStubSeriesTerminatedLogic_1v5 => "DIFF_SSTL15",
    }
}

impl Probe for UCFGenerator {
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
                format!("NET {}", name)
            } else {
                format!("NET {}<{}>", name, pin.index)
            };
            match &pin.constraint {
                Constraint::Location(l) => {
                    self.ucf.push(format!("{} LOC={}", prefix, l));
                }
                Constraint::Kind(k) => {
                    let name = map_signal_type_to_xilinx_string(k);
                    self.ucf.push(format!("{} IOSTANDARD={}", prefix, name))
                }
                Constraint::Timing(t) => {
                    let timing = match t {
                        Timing::Periodic(p) => {
                            format!("{prefix} TNM_NET={net}; TIMESPEC {ts} = PERIOD {net} {period} ns HIGH {duty}%",
                                    prefix=prefix,
                                    net=p.net,
                                    ts=format!("TS_{}", p.net),
                                    period = p.period_nanoseconds,
                                    duty = p.duty_cycle)
                        }
                        Timing::InputTiming(i) => {
                            format!("{prefix} OFFSET = IN {offset} ns VALID {valid} {relative} !{id}!{bit} {edge}",
                                prefix = prefix,
                                offset = i.offset_nanoseconds,
                                valid = i.valid_duration_nanoseconds,
                                relative = i.relative.to_string(),
                                id = i.to_signal_id,
                                bit = if let Some(n) = i.to_signal_bit {
                                    format!("<{}>", n)
                                } else {
                                    "".into()
                                },
                                edge = i.edge_sense.to_string()
                            )
                        }
                        Timing::OutputTiming(o) => {
                            format!(
                                "{prefix} OFFSET = OUT {offset} ns {relative} !{id}!{bit} {edge}",
                                prefix = prefix,
                                offset = o.offset_nanoseconds,
                                relative = o.relative.to_string(),
                                id = o.to_signal_id,
                                bit = if let Some(n) = o.to_signal_bit {
                                    format!("<{}>", n)
                                } else {
                                    "".into()
                                },
                                edge = o.edge_sense.to_string()
                            )
                        }
                        Timing::Custom(c) => c.to_string(),
                        Timing::VivadoFalsePath(_) => "".to_string(),
                        _ => unimplemented!("Unknown timing constraint for ISE/UCF generation"),
                    };
                    if !timing.is_empty() {
                        self.ucf.push(timing);
                    }
                }
                Constraint::Custom(s) => self.ucf.push(s.clone()),
                _ => {
                    unimplemented!("Unsupported constraint type for UCF files")
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

pub fn generate_ucf<U: Block>(uut: &U) -> String {
    let mut ucf = UCFGenerator::default();
    uut.accept("top", &mut ucf);
    // Substitute the signal references
    let xrefs = collect_xrefs(&ucf.ucf);
    let ucf_lines = substitute_refs(&ucf.ucf, &xrefs, &ucf.names);
    let mut ucf_uniq = vec![];
    for line in ucf_lines {
        if !ucf_uniq.contains(&line) {
            ucf_uniq.push(line);
        }
    }
    ucf_uniq.join(";\n")
        + ";
CONFIG VCCAUX = \"3.3\"; // Required for Spartan-6
"
}
