use crate::logic::Logic;
use crate::scoped_visitor::ScopedVisitor;

pub trait Block : Logic {
    fn update_all(&mut self);
    fn has_changed(&self) -> bool;
    fn accept_scoped(&self, name: &str, visitor: &mut dyn ScopedVisitor);
}
