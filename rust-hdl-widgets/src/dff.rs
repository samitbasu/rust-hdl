use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct DFF<T: Synth, F: Domain> {
    pub d: Signal<In, T, F>,
    pub q: Signal<Out, T, F>,
    pub clk: Signal<In, Clock, F>,
}

impl<T: Synth, F: Domain> Default for DFF<T, F> {
    fn default() -> DFF<T, F> {
        Self::new(T::default())
    }
}

impl<T: Synth, F: Domain> DFF<T, F> {
    pub fn new(init: T) -> DFF<T, F> {
        Self {
            d: Signal::default(),
            q: Signal::new_with_default(init), // This should be marked as a register, since we write to it on a clock edge
            clk: Signal::default(),
        }
    }
}

impl<T: Synth, F: Domain> Logic for DFF<T, F> {
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
}
