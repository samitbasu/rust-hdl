use crate::atom::Atom;
use crate::block::Block;

pub trait Probe {
    fn visit_start_scope(&mut self, name: &str, node: &dyn Block);
    fn visit_start_namespace(&mut self, name: &str, node: &dyn Block);
    fn visit_atom(&mut self, name: &str, signal: &dyn Atom);
    fn visit_end_namespace(&mut self, name: &str, node: &dyn Block);
    fn visit_end_scope(&mut self, name: &str, node: &dyn Block);
}
