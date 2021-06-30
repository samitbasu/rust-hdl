use crate::synth::Synth;

#[derive(Copy, Clone, Debug)]
pub struct Constant<T: Synth> {
    pub val: T,
}

impl<T: Synth> Constant<T> {
    pub fn new(val: T) -> Constant<T> {
        Constant { val }
    }
}
