use crate::atom::AtomKind::{StubInputSignal, StubOutputSignal};
use crate::atom::{Atom, AtomKind};
use crate::block::Block;
use crate::named_path::NamedPath;
use crate::probe::Probe;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
struct ModuleDetails {
    atoms: Vec<AtomDetails>,
    sub_modules: Vec<String>,
    enums: Vec<EnumDefinition>
}

#[derive(Clone, Debug, PartialEq)]
struct EnumDefinition {
    pub type_name: String,
    pub discriminant: String,
    pub value: usize
}

#[derive(Clone, Debug)]
struct AtomDetails {
    name: String,
    kind: AtomKind,
    width: usize,
}

fn verilog_atom_name(x: &AtomKind) -> &str {
    match x {
        AtomKind::InputParameter => "in",
        AtomKind::OutputParameter => "out",
        AtomKind::StubInputSignal => "reg",
        AtomKind::StubOutputSignal => "wire",
        AtomKind::Constant => "localparam",
        AtomKind::LocalSignal => "wire",
    }
}

fn decl(x: &AtomDetails) -> String {
    if x.width == 1 {
        format!("{} {}", verilog_atom_name(&x.kind), x.name)
    } else {
        format!("{} [{}:0] {}", verilog_atom_name(&x.kind), x.width - 1, x.name)
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
    fn add_enum(&mut self, module: &str, signal: &dyn Atom) {
        let entry = self.details.entry(module.into()).or_default();
        let enum_name = signal.type_name();
        let enum_values = (0..(1 << signal.bits()))
            .map(|x|
                EnumDefinition {
                    type_name: enum_name.into(),
                    discriminant: signal.name(x).into(),
                    value: x
                })
            .filter(|x| x.discriminant.len() != 0)
            .collect::<Vec<_>>();
        entry.enums.extend(enum_values.into_iter())
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
        println!(
            "Atom: name {} path {} namespace {} enum {} type {}",
            name,
            self.path.to_string(),
            self.namespace.flat("_"),
            signal.is_enum(),
            signal.type_name()
        );
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
            width: signal.bits(),
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
                width: signal.bits(),
            };
            let parent_name = self.path.parent();
            self.add_atom(&parent_name, parent_param);
        }
        if signal.is_enum() {
            self.add_enum(&module_path, signal);
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
            let atoms = &module_details.atoms;
            let args = atoms
                .iter()
                .filter(|x| x.kind.is_parameter())
                .collect::<Vec<_>>();
            let stubs = atoms
                .iter()
                .filter(|x| x.kind.is_stub())
                .collect::<Vec<_>>();
            let consts = atoms
                .iter()
                .filter(|x| x.kind == AtomKind::Constant)
                .collect::<Vec<_>>();
            let locals = atoms
                .iter()
                .filter(|x| x.kind == AtomKind::LocalSignal)
                .collect::<Vec<_>>();
            let arg_names = stubs
                .iter()
                .map(|x| x.name.to_owned())
                .collect::<Vec<_>>()
                .join(",");
            println!("({})", arg_names);
            println!("\n// Module arguments");
            args.iter().for_each(|x| println!("{}", decl(x)));
            let submodules = &module_details.sub_modules;
            println!("\n// Constant declarations");
            consts.iter().for_each(|x| println!("{}", decl(x)));
            println!("\n// Enums");
            module_details.enums.iter().for_each(|x |
              println!("localparam {}_{} = {}", x.type_name, x.discriminant, x.value));
            println!("\n// Stub signals");
            stubs.iter().for_each(|x| println!("{}", decl(x)));
            println!("\n// Local signals");
            locals.iter().for_each(|x| println!("{}", decl(x)));
            println!("\n// Sub module instances");
            submodules.iter().for_each(|x| println!("sub module {}", x));
            println!("end module {}", module_name);
        }
    }
}
