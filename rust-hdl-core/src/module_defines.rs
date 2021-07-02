use crate::named_path::NamedPath;
use crate::probe::Probe;
use crate::block::Block;
use crate::atom::{Atom, AtomKind};
use std::collections::HashMap;
use crate::atom::AtomKind::{StubInputSignal, StubOutputSignal};

#[derive(Clone, Debug, Default)]
struct ModuleDetails {
    atoms: Vec<AtomDetails>,
    sub_modules: Vec<String>,
}

#[derive(Clone, Debug)]
struct AtomDetails {
    name: String,
    kind: AtomKind,
    width: usize,
}

fn verilog_param(x: &AtomKind) -> &str {
    match x {
        AtomKind::InputParameter => "in",
        AtomKind::OutputParameter => "out",
        AtomKind::StubInputSignal => "reg",
        AtomKind::StubOutputSignal => "wire",
    }
}

fn decl(x: &AtomDetails) -> String {
    if x.width == 1 {
        format!("{} {}", verilog_param(&x.kind), x.name)
    } else {
        format!("{} [{}:0] {}", verilog_param(&x.kind), x.width-1, x.name)
    }
}

#[derive(Default)]
pub struct ModuleDefines {
    path: NamedPath,
    namespace: NamedPath,
    details: HashMap<String, ModuleDetails>,
}

impl ModuleDefines {
    fn add_atom(&mut self, module: &str, atom: AtomDetails) {
        let entry = self.details.entry(module.into()).or_default();
        entry.atoms.push(atom)
    }
    fn add_submodule(&mut self, module: &str, submodule: &str) {
        let entry = self.details.entry(module.into()).or_default();
        entry.sub_modules.push(submodule.into())
    }
}

impl Probe for ModuleDefines {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        let top_level = self.path.to_string();
        self.path.push(name);
        self.namespace.reset();
        self.add_submodule(&top_level, &self.path.to_string());
    }

    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.namespace.push(name);
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        println!("Atom: name {} path {} namespace {}", name, self.path.to_string(), self.namespace.flat("_"));
        let module_path = self.path.to_string();
        let module_name = self.path.last();
        let namespace = self.namespace.flat("_");
        let name = if namespace.is_empty() {
            name.to_owned()
        } else {
            format!("{}_{}", namespace, name)
        };
        let param = AtomDetails {
            name: name.clone(),
            kind: signal.kind(),
            width: signal.bits()
        };
        if param.kind.is_parameter() {
            let kind = if param.kind == AtomKind::InputParameter {
                StubInputSignal
            } else {
                StubOutputSignal
            };
            let parent_param = AtomDetails {
                name: format!("{}_{}", module_name, name.to_owned()),
                kind,
                width: signal.bits()
            };
            let parent_name = self.path.parent();
            self.add_atom(&parent_name, parent_param);
        }
        self.add_atom(&module_path, param);
    }

    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.namespace.pop();
    }

    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }
}

impl ModuleDefines {
    pub fn defines(&self) {
        for k in self.details.iter() {
            let module_name = k.0;
            let module_details = k.1;
            println!("\n\nmodule {}", module_name);
            let atoms= &module_details.atoms;
            let args = atoms
                .iter()
                .filter(|x| x.kind.is_parameter())
                .collect::<Vec<_>>();
            let locals = atoms
                .iter()
                .filter(|x| x.kind.is_stub())
                .collect::<Vec<_>>();
            let arg_names = args
                .iter()
                .map(|x| x.name.to_owned())
                .collect::<Vec<_>>()
                .join(",");
            println!("({})", arg_names);
            println!("\n// Module arguments");
            args.iter()
                .for_each(|x| println!("{}", decl(x)));
            let submodules = &module_details.sub_modules;
            println!("\n// Stub signals");
            locals.iter()
                .for_each(|x| println!("{}", decl(x)));
            println!("\n// Sub module instances");
            submodules.iter()
                .for_each(|x| println!("sub module {}", x));
            println!("end module {}", module_name);
        }
    }
}



