use crate::block::Block;
use crate::atom::Atom;

pub trait ScopedVisitor {
    fn visit_start_scope(&mut self, name: &str, node: &dyn Block);
    fn visit_atom(&mut self, name: &str, signal: &dyn Atom);
    fn visit_end_scope(&mut self, name: &str, node: &dyn Block);
}
