use crate::logic::Logic;
use crate::scoped_visitor::ScopedVisitor;
use crate::visitor_mut::VisitorMut;
use crate::visitor::Visitor;

pub trait Block : Logic {
    fn accept(&self, visitor: &mut dyn Visitor);
    fn accept_mut(&mut self, visitor: &mut dyn VisitorMut);
    fn accept_scoped(&self, name: &str, visitor: &mut dyn ScopedVisitor);
}
