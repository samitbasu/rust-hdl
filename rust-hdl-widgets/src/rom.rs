use rust_hdl_core::prelude::*;

#[derive(LogicBlock)]
pub struct ROM<T: Synth, const N: usize> {
    pub address: Signal<In, Bits<N>>,
    pub data: Signal<Out, T>,
    _sim: Vec<T>,
}

impl<T: Synth, const N: usize> ROM<T, N> {
    pub fn new(values: Vec<T>) -> Self {
        Self {
            address: Signal::default(),
            data: Signal::new_with_default(T::default()),
            _sim: values,
        }
    }
}

impl<T: Synth, const N: usize> Logic for ROM<T, N> {
    fn update(&mut self) {
        self.data.next = T::default();
        let ndx: usize = self.address.val().into();
        if ndx < self._sim.len() {
            self.data.next = self._sim[ndx]
        }
    }

    fn hdl(&self) -> Verilog {
        let cases = self._sim.iter()
            .enumerate()
            .map(|x|
                format!("  {}: data = {}", x.0, x.1.verilog().to_string()))
            .collect::<Vec<_>>()
            .join("\n");
        Verilog::Custom(format!("\
always @*
case (address)
  {cases}
  default: data = {default}
endcase
        ", cases = cases, default = T::default().verilog().to_string()))
    }
}
