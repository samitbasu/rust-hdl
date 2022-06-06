use crate::core::prelude::*;
use crate::core::timing::TimingInfo;
use crate::widgets::dff::DFF;
use std::ops::BitXor;

#[derive(Clone, Debug, LogicBlock)]
pub struct DFFWithInit<T: Synth + BitXor<Output = T>> {
    pub d: Signal<In, T>,
    pub q: Signal<Out, T>,
    pub clock: Signal<In, Clock>,
    pub init: Constant<T>,
    pub dff: DFF<T>,
}

impl<T: Synth + BitXor<Output = T>> DFFWithInit<T> {
    pub fn new(init: T) -> Self {
        Self {
            d: Default::default(),
            q: Default::default(),
            clock: Default::default(),
            init: Constant::new(init),
            dff: Default::default(),
        }
    }
}

impl<T: Synth + BitXor<Output = T>> Logic for DFFWithInit<T> {
    #[hdl_gen]
    fn update(&mut self) {
        self.dff.clock.next = self.clock.val();
        self.q.next = self.dff.q.val() ^ self.init.val();
        self.dff.d.next = self.d.val() ^ self.init.val();
    }
}
