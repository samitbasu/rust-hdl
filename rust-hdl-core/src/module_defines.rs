use crate::named_path::NamedPath;
use crate::probe::Probe;
use crate::block::Block;
use crate::atom::{Atom, AtomKind};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
struct ModuleDetails {
    name: String,
    params: Vec<SignalDetails>,
    locals: Vec<SignalDetails>,
    sub_modules: Vec<String>,
}

#[derive(Clone, Debug)]
struct SignalDetails {
    name: String,
    direction: AtomKind,
    width: usize,
}

fn decl(x: &SignalDetails) -> String {
    if x.width == 1 {
        format!("{:?} {}", x.direction, x.name)
    } else {
        format!("{:?} [{}:0] {}", x.direction, x.width-1, x.name)
    }
}

#[derive(Default)]
pub struct ModuleDefines {
    path: NamedPath,
    atoms: HashMap<String, ModuleDetails>,
}

impl Probe for ModuleDefines {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        let top_level = self.path.to_string();
        let mut details = self.atoms.get(&top_level).unwrap_or(&ModuleDetails::default()).clone();
        self.path.push(name);
        details.sub_modules.push(self.path.to_string());
        self.atoms.insert(top_level, details);
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        let module_name = self.path.to_string();
        let mut details = self.atoms.get(&module_name).unwrap_or(&ModuleDetails::default()).clone();
        details.name = module_name.clone();
        let param = SignalDetails {
            name: name.to_owned(),
            direction: signal.kind(),
            width: signal.bits()
        };
        details.params.push(param);
        self.atoms.insert(module_name.to_owned(), details);
    }

    fn visit_end_scope(&mut self, name: &str, node: &dyn Block) {
        self.path.pop();
    }
}

impl ModuleDefines {
    pub fn defines(&self) {
        for k in self.atoms.iter() {
            let module_name = k.0;
            let module_details = k.1;
            println!("\n\nmodule {}", module_details.name);
            let params = &module_details.params;
            println!("({})", params
                .iter()
                .map(|x| x.name.to_owned())
                .collect::<Vec<_>>()
                .join(",")
            );
            params.iter()
                .for_each(|x| println!("{}", decl(x)));
            let submodules = &module_details.sub_modules;
            submodules.iter()
                .for_each(|x| println!("sub module {}", x));
            println!("end module {}", module_name);
        }
    }
}



