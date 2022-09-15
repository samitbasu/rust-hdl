use crate::core::ast::{Verilog, VerilogExpression};
use crate::core::atom::{Atom, AtomKind};
use crate::core::block::Block;
use crate::core::check_error::{CheckError, PathedName, PathedNameList};
use crate::core::prelude::{NamedPath, VerilogVisitor};
use crate::core::probe::Probe;
use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Mode {
    Ignore,
    Read,
    Write,
}

struct VerilogWriteCollector {
    vars_written: HashSet<String>,
    vars_read: HashSet<String>,
    mode: Mode,
}

impl Default for VerilogWriteCollector {
    fn default() -> Self {
        Self {
            vars_written: Default::default(),
            vars_read: Default::default(),
            mode: Mode::Ignore,
        }
    }
}

fn get_write_list(uut: &dyn Block) -> HashSet<String> {
    match &uut.hdl() {
        Verilog::Combinatorial(code) => {
            let mut det = VerilogWriteCollector::default();
            det.visit_block(code);
            det.vars_written
        }
        _ => Default::default(),
    }
}

impl VerilogVisitor for VerilogWriteCollector {
    fn visit_slice_assignment(
        &mut self,
        base: &VerilogExpression,
        _width: &usize,
        offset: &VerilogExpression,
        replacement: &VerilogExpression,
    ) {
        let current_mode = self.mode;
        self.mode = Mode::Read;
        self.visit_expression(offset);
        self.visit_expression(replacement);
        self.mode = Mode::Write;
        self.visit_expression(base);
        self.mode = current_mode;
    }

    fn visit_signal(&mut self, c: &str) {
        let myname = c.replace("$next", "");
        match self.mode {
            Mode::Ignore => {}
            Mode::Write => {
                self.vars_written.insert(myname);
            }
            Mode::Read => {
                self.vars_read.insert(myname);
            }
        }
    }

    fn visit_assignment(&mut self, l: &VerilogExpression, r: &VerilogExpression) {
        let current_mode = self.mode;
        self.mode = Mode::Read;
        self.visit_expression(r);
        self.mode = Mode::Write;
        self.visit_expression(l);
        self.mode = current_mode;
    }
}

#[derive(Default)]
struct CheckInputsNotDriven {
    path: NamedPath,
    namespace: NamedPath,
    input_parameters: Vec<Vec<String>>,
    failures: PathedNameList,
}

impl Probe for CheckInputsNotDriven {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        self.input_parameters.push(vec![]);
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
        match signal.kind() {
            AtomKind::InputParameter => {
                self.input_parameters.last_mut().unwrap().push(name);
            }
            _ => {}
        }
    }

    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.namespace.pop();
    }

    fn visit_end_scope(&mut self, _name: &str, node: &dyn Block) {
        let written = get_write_list(node);
        let my_params = self.input_parameters.last().unwrap();
        for param in my_params {
            if written.contains(param) {
                self.failures.push(PathedName {
                    path: self.path.to_string(),
                    name: param.to_owned(),
                })
            }
        }
        self.path.pop();
        self.input_parameters.pop();
    }
}


/// Check a circuit to make sure that `Signal`s of type `In` are 
/// not written by the HDL kernel.  In RustHDL, you are not allowed
/// to write to input signals from within a module.
/// ```rust
/// use rust_hdl::prelude::*;
/// use rust_hdl::core::check_write_inputs::check_inputs_not_written;
/// 
/// #[derive(LogicBlock, Default)]
/// struct BadGuy {
///    pub in1: Signal<In, Bit>,
/// }
/// 
/// impl Logic for BadGuy {
///    #[hdl_gen]
///    fn update(&mut self) {
///       self.in1.next = false; // <-- rustc is OK with this, but RustHDL is not.
///    }
/// }
/// 
/// let mut uut = BadGuy::default(); uut.connect_all();
/// assert!(check_inputs_not_written(&uut).is_err());
/// ```
pub fn check_inputs_not_written(uut: &dyn Block) -> Result<(), CheckError> {
    let mut visitor = CheckInputsNotDriven::default();
    uut.accept("uut", &mut visitor);
    if visitor.failures.is_empty() {
        Ok(())
    } else {
        Err(CheckError::WritesToInputs(visitor.failures))
    }
}
