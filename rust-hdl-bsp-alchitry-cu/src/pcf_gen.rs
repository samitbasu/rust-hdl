use rust_hdl_core::named_path::NamedPath;
use rust_hdl_core::prelude::*;

#[derive(Default)]
struct PCFGenerator {
    path: NamedPath,
    namespace: NamedPath,
    pcf: Vec<String>,
}

impl Probe for PCFGenerator {
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
            let namespace = self.namespace.flat("$");
            let name = if namespace.is_empty() {
                name.to_owned()
            } else {
                format!("{}${}", namespace, name)
            };
            for pin in &signal.constraints() {
                match &pin.constraint {
                    Constraint::Location(l) => {
                        if signal.bits() == 1 {
                            self.pcf.push(format!("set_io {} {}", name, l))
                        } else {
                            self.pcf
                                .push(format!("set_io {}[{}] {}", name, pin.index, l))
                        }
                    }
                    Constraint::Custom(s) => self.pcf.push(s.clone()),
                    _ => {
                        panic!("Pin constraint type {:?} is unsupported!", pin.constraint)
                    }
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

pub fn generate_pcf<U: Block>(uut: &U) -> String {
    let mut pcf = PCFGenerator::default();
    uut.accept("top", &mut pcf);
    pcf.pcf.join("\n") + "\n"
}
