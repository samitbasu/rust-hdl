use crate::core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct DFF<T: Synth> {
    pub d: Signal<In, T>,
    pub q: Signal<Out, T>,
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    _reset_val: T,
}

impl<T: Synth> DFF<T> {
    pub fn new_with_reset_val(init: T) -> Self {
        Self {
            _reset_val: init,
            ..Default::default()
        }
    }
}

impl<T: Synth> Default for DFF<T> {
    fn default() -> DFF<T> {
        Self {
            d: Signal::default(),
            q: Signal::default(),
            clock: Signal::default(),
            reset: Signal::default(),
            _reset_val: T::default(),
        }
    }
}

impl<T: Synth> Logic for DFF<T> {
    fn update(&mut self) {
        if self.clock.pos_edge() | self.reset.pos_edge() {
            if self.reset.val().into() {
                self.q.next = self._reset_val;
            } else {
                self.q.next = self.d.val()
            }
        }
    }
    fn connect(&mut self) {
        self.q.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Custom(format!(
            "\
always @(posedge clock or posedge reset) begin
   if (reset) begin
      q <= {:x};
   end else begin
      q <= d;
   end
end
      ",
            self._reset_val.verilog()
        ))
    }
}

#[macro_export]
macro_rules! dff_setup {
    ($self: ident, $clock: ident, $reset: ident, $($dff: ident),+) => {
        $($self.$dff.clock.next = $self.$clock.val());+;
        $($self.$dff.reset.next = $self.$reset.val());+;
        $($self.$dff.d.next = $self.$dff.q.val());+;
    }
}
