use crate::core::ast::VerilogLiteral;
use crate::core::constraint::PinConstraint;
use crate::core::prelude::TypeKind;
use crate::core::synth::VCDValue;
use crate::core::type_descriptor::TypeDescriptor;

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AtomKind {
    InputParameter,
    OutputParameter,
    StubInputSignal,
    StubOutputSignal,
    Constant,
    LocalSignal,
    InOutParameter,
    OutputPassthrough,
}

impl AtomKind {
    pub fn is_parameter(&self) -> bool {
        match self {
            AtomKind::InputParameter
            | AtomKind::OutputParameter
            | AtomKind::InOutParameter
            | AtomKind::OutputPassthrough => true,
            _ => false,
        }
    }
    pub fn is_stub(&self) -> bool {
        match self {
            AtomKind::StubInputSignal | AtomKind::StubOutputSignal => true,
            _ => false,
        }
    }
}

#[doc(hidden)]
pub trait Atom {
    fn bits(&self) -> usize;
    fn connected(&self) -> bool;
    fn changed(&self) -> bool;
    fn kind(&self) -> AtomKind;
    fn descriptor(&self) -> TypeDescriptor;
    fn vcd(&self) -> VCDValue;
    fn id(&self) -> usize;
    fn verilog(&self) -> VerilogLiteral;
    fn constraints(&self) -> Vec<PinConstraint>;
}

pub fn is_atom_an_enum(atom: &dyn Atom) -> bool {
    match atom.descriptor().kind {
        TypeKind::Enum(_) => true,
        _ => false,
    }
}

pub fn is_atom_signed(atom: &dyn Atom) -> bool {
    match atom.descriptor().kind {
        TypeKind::Signed(_) => true,
        _ => false,
    }
}

pub fn get_atom_typename(atom: &dyn Atom) -> String {
    atom.descriptor().name.to_string()
}
