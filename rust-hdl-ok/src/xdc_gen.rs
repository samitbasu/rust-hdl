use crate::prelude::map_signal_type_to_xilinx_string;
use rust_hdl_core::prelude::*;

#[derive(Default)]
struct XDCGenerator {
    path: NamedPath,
    namespace: NamedPath,
    xdc: Vec<String>,
}

impl Probe for XDCGenerator {
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
            let prefix = if signal.bits() == 1 {
                format!("{}", name)
            } else {
                format!("{}[{}]", name, pin.index)
            };
            match &pin.constraint {
                Constraint::Location(l) => self.xdc.push(format!(
                    "set_property PACKAGE_PIN {location} [get_ports {{ {prefix} }}]",
                    location = l,
                    prefix = prefix
                )),
                Constraint::Slew(k) => {
                    if let SlewType::Fast = k {
                        self.xdc.push(format!(
                            "set_property SLEW FAST [get_ports {{ {prefix} }}]",
                            prefix = prefix
                        ))
                    }
                }
                Constraint::Kind(k) => {
                    let name = map_signal_type_to_xilinx_string(k);
                    self.xdc.push(format!(
                        "set_property IOSTANDARD {name} [get_ports {{ {prefix} }}]",
                        prefix = prefix,
                        name = name
                    ))
                }
                Constraint::Timing(t) => {
                    let timing = match t {
                        Timing::Periodic(p) => {
                            if p.duty_cycle != 50.0 {
                                unimplemented!("Only 50 % duty cycle clocks currently implemented");
                            }
                            format!("create_clock -name {net} -period {period} [get_ports {{ {prefix} }}]",
                                net=p.net,
                                period=p.period_nanoseconds,
                                prefix=prefix)
                        }
                        Timing::Custom(c) => c.to_string(),
                        Timing::VivadoOutputTiming(o) => {
                            format!("set_output_delay -add_delay -clock [get_clocks {{ {clock} }}] {delay} [get_ports {{ {prefix} }}]",
                                    clock = o.clock, delay = o.delay_nanoseconds, prefix=prefix)
                        }
                        Timing::VivadoInputTiming(i) => {
                            format!(
"set_input_delay -add_delay -max -clock [get_clocks {{ {clock} }}] {max} [get_ports {{ {prefix} }}]
set_input_delay -add_delay -min -clock [get_clocks {{ {clock} }}] {min} [get_ports {{ {prefix} }}]
set_multicycle_path -setup -from [get_ports {{ {prefix} }}] {cycles}",
                                clock = i.clock,
                                max = i.max_nanoseconds,
                                min = i.min_nanoseconds,
                                prefix = prefix,
                                cycles = i.multicycle
                            )
                        }
                        VivadoClockGroup(c) => {
                            format!(
                                "set_clock_groups -asynchronous {}",
                                c.iter()
                                    .map(|g| format!("-group {{ {} }}", g.join(" ")))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            )
                        }
                        VivadoFalsePath(p) => {
                            format!(
                                "set_false_path -from [get_pins -hierarchical -regexp {from}] -to [get_pins -hierarchical -regexp {to}]",
                                from = p.from_regexp,
                                to = p.to_regexp
                            )
                        }
                        _ => {
                            unimplemented!("Constraint type {:?} is not implemented for Vivado", t)
                        }
                    };
                    self.xdc.push(timing);
                }
                Constraint::Custom(s) => self.xdc.push(s.clone()),
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

pub fn generate_xdc<U: Block>(uut: &U) -> String {
    let mut xdc = XDCGenerator::default();
    uut.accept("top", &mut xdc);
    let mut xdc_uniq = vec![];
    for line in xdc.xdc {
        if !xdc_uniq.contains(&line) {
            xdc_uniq.push(line);
        }
    }
    xdc_uniq.join("\n")
        + "
set_property CFGBVS VCCO [current_design]
set_property CONFIG_VOLTAGE 3.3 [current_design]
set_property BITSTREAM.GENERAL.COMPRESS True [current_design]
    "
}
