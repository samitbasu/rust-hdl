use crate::atom::Atom;
use crate::block::Block;

pub trait Probe {
    fn visit_start_scope(&mut self, _name: &str, _node: &dyn Block) {}
    fn visit_start_namespace(&mut self, _name: &str, _node: &dyn Block) {}
    fn visit_atom(&mut self, _name: &str, _signal: &dyn Atom) {}
    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {}
    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {}
}
