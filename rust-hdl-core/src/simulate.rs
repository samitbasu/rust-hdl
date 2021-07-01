use crate::block::Block;

pub fn simulate<B: Block>(uut: &mut B, max_iters: usize) -> bool {
    for _ in 0..max_iters {
        uut.update_all();
        if !uut.has_changed() {
            return true;
        }
    }
    false
}
