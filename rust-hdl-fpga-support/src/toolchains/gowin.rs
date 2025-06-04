use rust_hdl_core::prelude::*;

use super::map_signal_type_to_gowin_string;

#[derive(Default)]
struct CSTGenerator {
    path: NamedPath,
    namespace: NamedPath,
    cst: Vec<String>,
}

impl Probe for CSTGenerator {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
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
                let prefix = if signal.bits() == 1 {
                    format!("{}", name)
                } else {
                    format!("{}[{}]", name, pin.index)
                };

                match &pin.constraint {
                    Constraint::Location(l) => {
                        self.cst.push(format!("IO_LOC \"{}\" {};", prefix, l));
                    }
                    Constraint::Kind(k) => {
                        let name = map_signal_type_to_gowin_string(k);
                        self.cst
                            .push(format!("IO_PORT \"{}\" IO_TYPE={};", prefix, name))
                    }
                    Constraint::Custom(s) => self.cst.push(s.clone()),
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

pub fn generate_cst<U: Block>(uut: &U) -> String {
    let mut cst = CSTGenerator::default();
    uut.accept("top", &mut cst);
    cst.cst.join("\n") + "\n"
}
