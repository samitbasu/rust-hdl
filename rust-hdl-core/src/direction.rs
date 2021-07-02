use crate::atom::AtomKind;

pub trait Direction: Clone {
    const KIND: AtomKind;
}

#[derive(Default, Clone, Debug)]
pub struct In {}

#[derive(Default, Clone, Debug)]
pub struct Out {}

#[derive(Default, Clone, Debug)]
pub struct Local {}

impl Direction for In {
    const KIND: AtomKind = AtomKind::InputParameter;
}

impl Direction for Out {
    const KIND: AtomKind = AtomKind::OutputParameter;
}

impl Direction for Local {
    const KIND: AtomKind = AtomKind::LocalSignal;
}