use crate::ast::{Verilog, VerilogLink, VerilogLiteral};
use crate::atom::AtomKind::{StubInputSignal, StubOutputSignal};
use crate::atom::{is_atom_signed, Atom, AtomKind};
use crate::block::Block;
use crate::check_error::check_all;
use crate::code_writer::CodeWriter;
use crate::named_path::NamedPath;
use crate::probe::Probe;
use crate::type_descriptor::{TypeDescriptor, TypeKind};
use crate::verilog_gen::{verilog_combinatorial, verilog_link_extraction};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
struct SubModuleInvocation {
    kind: String,
    name: String,
}

#[derive(Clone, Debug, Default)]
struct ModuleDetails {
    atoms: Vec<AtomDetails>,
    sub_modules: Vec<SubModuleInvocation>,
    enums: Vec<EnumDefinition>,
    code: Verilog,
    links: Vec<VerilogLink>,
}

#[derive(Clone, Debug, PartialEq)]
struct EnumDefinition {
    pub type_name: String,
    pub discriminant: String,
    pub value: usize,
}

#[derive(Clone, Debug)]
struct AtomDetails {
    name: String,
    kind: AtomKind,
    width: usize,
    const_val: VerilogLiteral,
    signed: bool,
}

fn verilog_atom_name(x: &AtomKind) -> &str {
    match x {
        AtomKind::InputParameter => "input wire",
        AtomKind::OutputParameter => "output reg",
        AtomKind::StubInputSignal => "reg",
        AtomKind::StubOutputSignal => "wire",
        AtomKind::Constant => "localparam",
        AtomKind::LocalSignal => "reg",
        AtomKind::InOutParameter => "inout wire",
        AtomKind::OutputPassthrough => "output wire",
    }
}

fn decl(x: &AtomDetails) -> String {
    let signed = if x.signed { "signed" } else { "" };
    if x.kind == AtomKind::Constant {
        format!(
            "{} {} {} = {};",
            verilog_atom_name(&x.kind),
            signed,
            x.name,
            x.const_val
        )
    } else {
        if x.width == 1 {
            format!("{} {} {};", verilog_atom_name(&x.kind), signed, x.name)
        } else {
            format!(
                "{} {} [{}:0] {};",
                verilog_atom_name(&x.kind),
                signed,
                x.width - 1,
                x.name
            )
        }
    }
}

#[derive(Default)]
pub struct ModuleDefines {
    path: NamedPath,
    namespace: NamedPath,
    details: BTreeMap<String, ModuleDetails>,
}

impl ModuleDefines {
    fn add_atom(&mut self, module: &str, atom: AtomDetails) {
        let entry = self.details.entry(module.into()).or_default();
        entry.atoms.push(atom)
    }
    fn add_submodule(&mut self, module: &str, name: &str, kind: &str) {
        let entry = self.details.entry(module.into()).or_default();
        entry.sub_modules.push(SubModuleInvocation {
            kind: kind.to_owned(),
            name: name.to_owned(),
        });
    }
    fn add_enums(&mut self, module: &str, descriptor: &TypeDescriptor) {
        let entry = self.details.entry(module.into()).or_default();
        let enum_name = descriptor.name.clone();
        match &descriptor.kind {
            TypeKind::Enum(x) => {
                for (ndx, label) in x.iter().enumerate() {
                    let def = EnumDefinition {
                        type_name: enum_name.clone(),
                        discriminant: label.into(),
                        value: ndx,
                    };
                    if !entry.enums.contains(&def) {
                        entry.enums.push(def);
                    }
                }
            }
            TypeKind::Composite(x) => {
                for item in x {
                    self.add_enums(module, &item.kind);
                }
            }
            _ => {}
        }
    }
    fn add_code(&mut self, module: &str, code: Verilog) {
        let entry = self.details.entry(module.into()).or_default();
        entry.links = match &code {
            Verilog::Combinatorial(code) => verilog_link_extraction(code),
            _ => {
                vec![]
            }
        };
        entry.code = code;
    }
}

impl Probe for ModuleDefines {
    fn visit_start_scope(&mut self, name: &str, node: &dyn Block) {
        let top_level = self.path.to_string();
        self.path.push(name);
        self.namespace.reset();
        self.add_submodule(&top_level, name, &self.path.to_string());
        self.add_code(&self.path.to_string(), node.hdl());
    }

    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.namespace.push(name);
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        let module_path = self.path.to_string();
        let module_name = self.path.last();
        let namespace = self.namespace.flat("$");
        let name = if namespace.is_empty() {
            name.to_owned()
        } else {
            format!("{}${}", namespace, name)
        };
        let param = AtomDetails {
            name: name.clone(),
            kind: signal.kind(),
            width: signal.bits(),
            const_val: signal.verilog(),
            signed: is_atom_signed(signal),
        };
        if param.kind.is_parameter() {
            let kind = if param.kind == AtomKind::InputParameter {
                StubInputSignal
            } else {
                StubOutputSignal
            };
            let parent_param = AtomDetails {
                name: format!("{}${}", module_name, name.to_owned()),
                kind,
                width: signal.bits(),
                const_val: signal.verilog(),
                signed: is_atom_signed(signal),
            };
            let parent_name = self.path.parent();
            self.add_atom(&parent_name, parent_param);
        }
        self.add_enums(&module_path, &signal.descriptor());
        self.add_enums(&self.path.parent(), &signal.descriptor());
        self.add_atom(&module_path, param);
    }

    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.namespace.pop();
    }

    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }
}

fn get_link_equivalence(link: &VerilogLink) -> (String, String) {
    match link {
        VerilogLink::Forward(link) => (
            format!("{}${}", link.other_name, link.my_name),
            format!("{}${}", link.owner_name, link.my_name),
        ),
        VerilogLink::Backward(link) => (
            format!("{}${}", link.owner_name, link.my_name),
            format!("{}${}", link.other_name, link.my_name),
        ),
        VerilogLink::Bidirectional(link) => {
            if link.my_name.is_empty() {
                (link.owner_name.clone(), link.other_name.clone())
            } else {
                (
                    format!("{}${}", link.other_name, link.my_name),
                    format!("{}${}", link.owner_name, link.my_name),
                )
            }
        }
    }
}

impl ModuleDefines {
    fn sub_module_invocation(
        &self,
        module_details: &ModuleDetails,
        child: &SubModuleInvocation,
        io: &mut CodeWriter,
    ) {
        let entry = self.details.get(&child.kind).unwrap();
        let submodule_kind = match &entry.code {
            Verilog::Blackbox(b) => &b.name,
            _ => &child.kind,
        };
        let child_args = entry
            .atoms
            .iter()
            .filter(|x| x.kind.is_parameter())
            .map(|x| {
                let arg_name = format!("{}${}", child.name, x.name);
                let arg_name = if self.stub_is_linked_to_module_argument(module_details, &arg_name)
                {
                    self.get_linked_argument_name(module_details, &arg_name)
                } else {
                    arg_name
                };
                format!(".{}({})", x.name, arg_name)
            })
            .collect::<Vec<_>>()
            .join(",\n");
        io.add(format!("{} {}(\n", submodule_kind, child.name));
        io.push();
        io.add(child_args);
        io.pop();
        io.add(");\n");
    }
    fn module_argument_is_passed_through_to_submodule(
        &self,
        module_details: &ModuleDetails,
        module_arg_name: &str,
    ) -> bool {
        for child in &module_details.sub_modules {
            let entry = self.details.get(&child.kind).unwrap();
            for child_arg in &entry.atoms {
                let arg_name = format!("{}${}", child.name, child_arg.name);
                if self.get_linked_argument_name(module_details, &arg_name) == module_arg_name {
                    return true;
                }
            }
        }
        false
    }
    fn get_linked_argument_name(&self, module_details: &ModuleDetails, arg_name: &str) -> String {
        for link in &module_details.links {
            let equiv = get_link_equivalence(link);
            if arg_name == equiv.0 {
                return equiv.1.clone();
            }
            if arg_name == equiv.1 {
                return equiv.0.clone();
            }
        }
        arg_name.to_string()
    }
    fn signal_name_is_module_argument(
        &self,
        module_details: &ModuleDetails,
        signal_name: &str,
    ) -> bool {
        // We now know the stub is linked to something... but is that a module argument?
        for atom in &module_details.atoms {
            if atom.kind.is_parameter() && atom.name == signal_name {
                return true;
            }
        }
        false
    }
    fn stub_is_linked_to_module_argument(
        &self,
        module_details: &ModuleDetails,
        atom_name: &str,
    ) -> bool {
        for link in &module_details.links {
            let equiv = get_link_equivalence(link);
            if atom_name == equiv.0 || atom_name == equiv.1 {
                let linked_name = if atom_name == equiv.0 {
                    equiv.1
                } else {
                    equiv.0
                };
                if self.signal_name_is_module_argument(module_details, &linked_name) {
                    return true;
                }
            }
        }
        false
    }
    fn process_module(
        &self,
        module_name: &str,
        module_details: &ModuleDetails,
        io: &mut CodeWriter,
    ) {
        // Remap the output parameters to pass through (net type) in case we have a wrapper
        let atoms_passthrough = &module_details
            .atoms
            .iter()
            .map(|x| {
                let mut y = x.clone();
                if y.kind == AtomKind::OutputParameter {
                    y.kind = AtomKind::OutputPassthrough;
                }
                y
            })
            .collect::<Vec<_>>();
        let wrapper_mode = if let Verilog::Wrapper(_) = &module_details.code {
            true
        } else {
            false
        };
        let atoms = if wrapper_mode {
            io.add("\n// v-- Setting output parameters to net type for wrapped code.\n");
            &atoms_passthrough
        } else {
            &module_details.atoms
        };
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
        let module_args = args
            .iter()
            .map(|x| x.name.to_owned())
            .collect::<Vec<_>>()
            .join(",");
        io.add(format!("\n\nmodule {}({});", module_name, module_args));
        io.push();
        if !args.is_empty() {
            io.add("\n// Module arguments");
            args.iter().for_each(|x| {
                if !self.module_argument_is_passed_through_to_submodule(module_details, &x.name)
                    || x.kind != AtomKind::OutputParameter
                {
                    io.add(decl(x))
                } else {
                    // For some synthesis engines, you cannot pass a module argument
                    // to a child module if it is of reg type
                    let mut x = (*x).clone();
                    x.kind = AtomKind::OutputPassthrough;
                    io.add(decl(&x))
                }
            });
        }
        let submodules = &module_details.sub_modules;
        if !consts.is_empty() {
            io.add("\n// Constant declarations");
            consts.iter().for_each(|x| io.add(decl(x)));
        }
        if !module_details.enums.is_empty() & !wrapper_mode {
            io.add("\n// Enums");
            module_details.enums.iter().for_each(|x| {
                io.add(format!(
                    "localparam {} = {};",
                    x.discriminant.replace("::", "$"),
                    x.value
                ))
            });
        }
        if !stubs.is_empty() & !wrapper_mode {
            io.add("\n// Stub signals");
            stubs.iter().for_each(|x| {
                if !self.stub_is_linked_to_module_argument(module_details, &x.name) {
                    io.add(decl(x))
                }
            });
        }
        if !locals.is_empty() & !wrapper_mode {
            io.add("\n// Local signals");
            locals.iter().for_each(|x| io.add(decl(x)));
        }
        if !submodules.is_empty() & !wrapper_mode {
            io.add("\n// Sub module instances");
            for child in submodules {
                self.sub_module_invocation(module_details, child, io);
            }
        }
        match &module_details.code {
            Verilog::Combinatorial(code) => {
                io.add("\n// Update code");
                io.add(verilog_combinatorial(code));
            }
            Verilog::Custom(code) => {
                io.add("\n// Update code (custom)");
                io.add(code);
            }
            Verilog::Wrapper(c) => {
                io.add("\n// Update code (wrapper)");
                io.add(&c.code);
            }
            Verilog::Blackbox(_) => {}
            Verilog::Empty => {}
        }
        for x in &module_details.links {
            let equiv = get_link_equivalence(x);
            if !self.signal_name_is_module_argument(module_details, &equiv.0)
                & !self.signal_name_is_module_argument(module_details, &equiv.1)
            {
                let txt = match x {
                    VerilogLink::Forward(x) => {
                        format!(
                            "always @(*) {}${} = {}${};",
                            x.other_name.replace("[", "$").replace("]", ""),
                            x.my_name,
                            x.owner_name.replace("[", "$").replace("]", ""),
                            x.my_name
                        )
                    }
                    VerilogLink::Backward(x) => {
                        format!(
                            "always @(*) {}${} = {}${};",
                            x.owner_name.replace("[", "$").replace("]", ""),
                            x.my_name,
                            x.other_name.replace("[", "$").replace("]", ""),
                            x.my_name
                        )
                    }
                    VerilogLink::Bidirectional(x) => {
                        if x.my_name.is_empty() {
                            format!("assign {} = {};", x.owner_name, x.other_name)
                        } else {
                            format!(
                                "assign {}${} = {}${};",
                                x.owner_name, x.my_name, x.other_name, x.my_name
                            )
                        }
                    }
                };
                io.add_line(txt);
            }
        }
        io.pop();
        io.add(format!("endmodule // {}", module_name));
    }

    pub fn defines(&self) -> String {
        let mut io = CodeWriter::default();
        self.details
            .iter()
            .filter(|x| x.0.len() != 0)
            .filter(|x| !matches!(x.1.code, Verilog::Blackbox(_)))
            .for_each(|k| {
                let module_name = k.0;
                let module_details = k.1;
                self.process_module(module_name, module_details, &mut io);
            });
        self.details.iter().for_each(|x| match &x.1.code {
            Verilog::Blackbox(b) => io.add(&b.code),
            Verilog::Wrapper(w) => io.add(&w.cores),
            _ => {}
        });
        io.to_string()
    }
}

pub fn generate_verilog<U: Block>(uut: &U) -> String {
    let mut defines = ModuleDefines::default();
    check_all(uut).unwrap(); // TODO - make this not panic...
    uut.accept("top", &mut defines);
    defines.defines()
}

pub fn generate_verilog_unchecked<U: Block>(uut: &U) -> String {
    let mut defines = ModuleDefines::default();
    uut.accept("top", &mut defines);
    defines.defines()
}
