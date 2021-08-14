use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;

#[derive(LogicBlock)]
pub struct SyncROM<D: Synth, const N: usize> {
    pub address: Signal<In, Bits<N>>,
    pub clock: Signal<In, Clock>,
    pub data: Signal<Out, D>,
    _sim: BTreeMap<Bits<N>, D>,
}

impl<D: Synth, const N: usize> SyncROM<D, N> {
    pub fn new(values: BTreeMap<Bits<N>, D>) -> Self {
        Self {
            address: Signal::default(),
            data: Signal::new_with_default(D::default()),
            clock: Signal::default(),
            _sim: values,
        }
    }
}

impl<D: Synth, const N: usize> Logic for SyncROM<D, N> {
    fn update(&mut self) {
        if self.clock.pos_edge() {
            self.data.next = *self._sim.get(&self.address.val()).unwrap_or(&D::default());
        }
    }

    fn connect(&mut self) {
        self.data.connect();
    }

    fn hdl(&self) -> Verilog {
        let init = self
            ._sim
            .iter()
            .map(|x| {
                format!(
                    "mem[{}] = {}",
                    x.0.verilog().to_string(),
                    x.1.verilog().to_string()
                )
            })
            .collect::<Vec<_>>()
            .join(";\n");
        Verilog::Custom(format!(
            "\
reg[{D}:0] mem [{Acount}:0];

initial begin
{init};
end

always @(posedge clock) begin
   data <= mem[address];
end",
            D = D::BITS - 1,
            Acount = (1 << N) - 1,
            init = init
        ))
    }
}
