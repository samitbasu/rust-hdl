use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;
use std::marker::PhantomData;

#[derive(LogicBlock)]
pub struct ROM<A: Synth + Ord, D: Synth, F: Domain> {
    pub address: Signal<In, A, F>,
    pub data: Signal<Out, D, F>,
    _sim: BTreeMap<A, D>,
}

impl<A: Synth + Ord, D: Synth, F: Domain> ROM<A, D, F> {
    pub fn new(values: BTreeMap<A, D>) -> Self {
        Self {
            address: Signal::default(),
            data: Signal::new_with_default(D::default()),
            _sim: values,
        }
    }
}

impl<A: Synth + Ord, D: Synth, F: Domain> Logic for ROM<A, D, F> {
    fn update(&mut self) {
        self.data.next = Tagged(*self._sim.get(&self.address.val().raw()).unwrap_or(&D::default()), PhantomData);
    }

    fn connect(&mut self) {
        self.data.connect();
    }

    fn hdl(&self) -> Verilog {
        let cases = self._sim.iter()
            .map(|x|
                format!("  {}: data = {};", x.0.verilog().to_string(),
                        x.1.verilog().to_string()))
            .collect::<Vec<_>>()
            .join("\n");
        Verilog::Custom(format!("\
always @*
case (address)
  {cases}
  default: data = {default};
endcase
        ", cases = cases, default = D::default().verilog().to_string()))
    }
}
