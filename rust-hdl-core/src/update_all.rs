use crate::visitor_mut::VisitorMut;
use crate::block::Block;

struct Updater {}

impl VisitorMut for Updater {
    #[inline(always)]
    fn visit(&mut self, node: &mut dyn Block) {
        node.update()
    }
}

#[inline(always)]
pub(crate) fn update_all(uut: &mut dyn Block) {
    let mut visit = Updater {};
    uut.accept_mut(&mut visit);
}
