use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;

#[derive(LogicBlock)]
pub struct SyncROM<A: Synth + Ord, D: Synth> {
    pub address: Signal<In, A>,
    pub clock: Signal<In, Clock>,
    pub data: Signal<Out, D>,
    _sim: BTreeMap<A, D>,
}

impl<A: Synth + Ord, D: Synth> SyncROM<A, D> {
    pub fn new(values: BTreeMap<A, D>) -> Self {
        Self {
            address: Signal::default(),
            data: Signal::new_with_default(D::default()),
            clock: Signal::default(),
            _sim: values,
        }
    }
}

impl<A: Synth + Ord, D: Synth> Logic for SyncROM<A, D> {
    fn update(&mut self) {
        if self.clock.pos_edge() {
            self.data.next = *self._sim.get(&self.address.val()).unwrap_or(&D::default());
        }
    }

    fn connect(&mut self) {
        self.data.connect();
    }

    fn hdl(&self) -> Verilog {
        let init = self._sim.iter()
            .map(|x| format!("mem[{}] = {}", x.0.verilog().to_string(), x.1.verilog().to_string()))
            .collect::<Vec<_>>()
            .join(";\n");
        Verilog::Custom(format!("\
reg[{D}:0] mem [{Acount}:0];

initial begin
{init};
end

always @(posedge clock) begin
   data <= mem[address];
end", D = D::BITS - 1, Acount = (1 << A::BITS) - 1, init = init))
    }
}