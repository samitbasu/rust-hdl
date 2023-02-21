use rust_hdl_private_core::prelude::*;

#[derive(LogicBlock)]
pub struct TopWrap<U: Block> {
    pub uut: U,
}

impl<U: Block> TopWrap<U> {
    pub fn new(uut: U) -> Self {
        Self { uut }
    }
}

impl<U: Block> Logic for TopWrap<U> {
    fn update(&mut self) {}
}
