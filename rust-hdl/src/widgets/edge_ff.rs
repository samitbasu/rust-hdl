use crate::core::prelude::*;
use crate::core::timing::TimingInfo;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct EdgeDFF<T: Synth> {
    pub d: Signal<In, T>,
    pub q: Signal<Out, T>,
    pub clk: Signal<In, Clock>,
}

impl<T: Synth> EdgeDFF<T> {
    pub fn new(init: T) -> EdgeDFF<T> {
        Self {
            d: Signal::default(),
            q: Signal::new_with_default(init),
            clk: Signal::default(),
        }
    }
}

// TODO - make this specializable
impl<T: Synth> Logic for EdgeDFF<T> {
    fn update(&mut self) {
        if self.clk.pos_edge() {
            self.q.next = self.d.val()
        }
    }
    fn connect(&mut self) {
        self.q.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Custom(format!(
            "\
initial begin
   q = {:x};
end

always @(posedge clk) q <= d;",
            self.q.verilog()
        ))
    }
    fn timing(&self) -> Vec<TimingInfo> {
        vec![TimingInfo {
            name: "edge_ff".to_string(),
            clock: "clk".to_string(),
            inputs: vec!["d".into()],
            outputs: vec!["q".into()],
        }]
    }
}
