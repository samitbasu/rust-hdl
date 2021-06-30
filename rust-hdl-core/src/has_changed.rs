use crate::atom::Atom;
use crate::block::Block;
use crate::visitor::Visitor;

struct HasChanged(bool);

impl Visitor for HasChanged {
    #[inline(always)]
    fn visit_atom(&mut self, atom: &dyn Atom) {
        self.0 |= atom.changed()
    }
}

#[inline(always)]
pub(crate) fn has_changed(uut: &dyn Block) -> bool {
    let mut visitor = HasChanged(false);
    uut.accept(&mut visitor);
    visitor.0
}
