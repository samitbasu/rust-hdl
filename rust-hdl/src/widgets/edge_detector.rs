use crate::core::prelude::*;
use crate::dff_setup;
use crate::widgets::dff::DFF;

#[derive(LogicBlock)]
pub struct EdgeDetector {
    pub input_signal: Signal<In, Bit>,
    pub edge_signal: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    prev: DFF<Bit>,
    current: DFF<Bit>,
    is_rising: Constant<Bit>,
}

impl EdgeDetector {
    pub fn new(is_rising: bool) -> Self {
        Self {
            input_signal: Default::default(),
            edge_signal: Default::default(),
            clock: Default::default(),
            reset: Default::default(),
            prev: Default::default(),
            current: Default::default(),
            is_rising: Constant::new(is_rising),
        }
    }
}

impl Logic for EdgeDetector {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, reset, prev, current);
        self.prev.d.next = self.current.q.val();
        self.current.d.next = self.is_rising.val() ^ self.input_signal.val();
        self.edge_signal.next = !self.current.q.val() & self.prev.q.val();
    }
}

#[test]
fn test_edge_detector_synthesizes() {
    let mut uut = EdgeDetector::new(false);
    uut.connect_all();
    yosys_validate("edge", &generate_verilog(&uut)).unwrap();
}
