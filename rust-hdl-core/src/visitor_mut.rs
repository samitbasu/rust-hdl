use crate::block::Block;

pub trait VisitorMut {
    fn visit(&mut self, _node: &mut dyn Block) {}
}
