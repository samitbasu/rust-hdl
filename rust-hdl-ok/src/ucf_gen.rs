use rust_hdl_core::prelude::*;

#[derive(Default)]
struct UCFGenerator {
    path: NamedPath,
    namespace: NamedPath,
    ucf: Vec<String>,
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
        if self.path.len() == 1 {
            let namespace = self.namespace.flat("_");
            let name = if namespace.is_empty() {
                name.to_owned()
            } else {
                format!("{}_{}", namespace, name)
            };
            for pin in &signal.constraints() {
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
                        let name = match k {
                            SignalType::LowVoltageCMOS_1v8 => "LVCMOS18",
                            SignalType::LowVoltageCMOS_3v3 => "LVCMOS33",
                            SignalType::StubSeriesTerminatedLogic_II => "SSTL18_II",
                            SignalType::DifferentialStubSeriesTerminatedLogic_II => "DIFF_SSTL18_II",
                            SignalType::StubSeriesTerminatedLogic_II_No_Termination => "SSTL18_II | IN_TERM=NONE",
                            SignalType::DifferentialStubSeriesTerminatedLogic_II_No_Termination => "DIFF_SSTL18_II | IN_TERM=NONE",
                            SignalType::Custom(c) => c,
                        };
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
                            Timing::Custom(c) => {
                                c.to_string()
                            }
                        };
                        self.ucf.push(timing);
                    }
                    Constraint::Custom(s) => self.ucf.push(s.clone()),
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
    ucf.ucf.join(";\n") + ";\n"
}