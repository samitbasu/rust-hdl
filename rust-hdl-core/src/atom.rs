use num_bigint::BigUint;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AtomKind {
    InputParameter,
    OutputParameter,
    StubInputSignal,
    StubOutputSignal,
    Constant,
    LocalSignal,
}

impl AtomKind {
    pub fn is_parameter(&self) -> bool {
        match self {
            AtomKind::InputParameter | AtomKind::OutputParameter => true,
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

pub trait Atom {
    fn bits(&self) -> usize;
    fn connected(&self) -> bool;
    fn changed(&self) -> bool;
    fn kind(&self) -> AtomKind;
    fn is_enum(&self) -> bool;
    fn name(&self, ndx: usize) -> &'static str;
    fn type_name(&self) -> &'static str;
    fn value(&self) -> BigUint;
}
