#[derive(Copy, Clone, Debug)]
pub enum AtomKind {
    InputParameter,
    OutputParameter,
    LocalSignal,
}

pub trait Atom {
    fn bits(&self) -> usize;
    fn connected(&self) -> bool;
    fn changed(&self) -> bool;
    fn kind(&self) -> AtomKind;
}
