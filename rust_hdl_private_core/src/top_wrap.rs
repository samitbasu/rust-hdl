use crate::{ast::Verilog, block::Block, logic::Logic, probe::Probe, timing::TimingInfo};

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

impl<U: Block> Block for TopWrap<U> {
    fn connect_all(&mut self) {
        self.connect();
        self.uut.connect_all();
    }
    fn update_all(&mut self) {
        self.update();
        self.uut.update_all();
    }
    fn has_changed(&self) -> bool {
        self.uut.has_changed()
    }
    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        probe.visit_start_scope(name, self);
        self.uut.accept("uut", probe);
        probe.visit_end_scope(name, self);
    }
}
