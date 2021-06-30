use crate::block::Block;
use crate::has_changed::has_changed;
use crate::update_all::update_all;

pub fn simulate(uut: &mut dyn Block, max_iters: usize) -> bool {
    for _ in 0..max_iters {
        update_all(uut);
        if !has_changed(uut) {
            return true;
        }
    }
    false
}
