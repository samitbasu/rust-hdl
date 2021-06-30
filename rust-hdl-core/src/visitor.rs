use crate::block::Block;
use crate::atom::Atom;

pub trait Visitor {
    fn visit(&mut self, _node: &dyn Block) {}
    fn visit_atom(&mut self, _atom: &dyn Atom) {}
}
