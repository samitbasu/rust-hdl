use crate::atom::AtomKind;

pub trait Direction {
    const NAME: &'static str;
    const KIND: AtomKind;
}

pub struct In {}

pub struct Out {}

impl Direction for In {
    const NAME: &'static str = "in";
    const KIND: AtomKind = AtomKind::InputParameter;
}

impl Direction for Out {
    const NAME: &'static str = "out";
    const KIND: AtomKind = AtomKind::OutputParameter;
}
