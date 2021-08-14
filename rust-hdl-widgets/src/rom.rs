use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;

#[derive(LogicBlock)]
pub struct ROM<D: Synth, const N: usize> {
    pub address: Signal<In, Bits<N>>,
    pub data: Signal<Out, D>,
    _sim: BTreeMap<Bits<N>, D>,
}

impl<D: Synth, const N: usize> ROM<D, N> {
    pub fn new(values: BTreeMap<Bits<N>, D>) -> Self {
        Self {
            address: Signal::default(),
            data: Signal::new_with_default(D::default()),
            _sim: values,
        }
    }
}

impl<D: Synth, const N: usize> Logic for ROM<D, N> {
    fn update(&mut self) {
        self.data.next = *self._sim.get(&self.address.val()).unwrap_or(&D::default());
    }

    fn connect(&mut self) {
        self.data.connect();
    }

    fn hdl(&self) -> Verilog {
        let cases = self
            ._sim
            .iter()
            .map(|x| {
                format!(
                    "  {}: data = {};",
                    x.0.verilog().to_string(),
                    x.1.verilog().to_string()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Verilog::Custom(format!(
            "\
always @*
case (address)
  {cases}
  default: data = {default};
endcase
        ",
            cases = cases,
            default = D::default().verilog().to_string()
        ))
    }
}
