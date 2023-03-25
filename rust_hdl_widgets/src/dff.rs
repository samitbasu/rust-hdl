use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct DFF<T: Synth> {
    pub d: Signal<In, T>,
    pub q: Signal<Out, T>,
    pub clock: Signal<In, Clock>,
}

impl<T: Synth> Default for DFF<T> {
    fn default() -> DFF<T> {
        Self {
            d: Signal::default(),
            q: Signal::default(),
            clock: Signal::default(),
        }
    }
}

impl<T: Synth> Logic for DFF<T> {
    fn update(&mut self) {
        if self.clock.pos_edge() {
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

always @(posedge clock) begin
   q <= d;
end
      ",
            T::default().verilog()
        ))
    }
    fn timing(&self) -> Vec<TimingInfo> {
        vec![TimingInfo {
            name: "dff".into(),
            clock: "clock".into(),
            inputs: vec!["d".into()],
            outputs: vec!["q".into()],
        }]
    }
}

#[macro_export]
macro_rules! dff_setup {
    ($self: ident, $clock: ident, $($dff: ident),+) => {
        $($self.$dff.clock.next = $self.$clock.val());+;
        $($self.$dff.d.next = $self.$dff.q.val());+;
    }
}
