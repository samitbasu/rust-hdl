use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;
use std::marker::PhantomData;

#[derive(LogicBlock)]
pub struct SyncROM<A: Synth + Ord, D: Synth, F: Domain> {
    pub address: Signal<In, A, F>,
    pub clock: Signal<In, Clock, F>,
    pub data: Signal<Out, D, F>,
    _sim: BTreeMap<A, D>,
}

impl<A: Synth + Ord, D: Synth, F: Domain> SyncROM<A, D, F> {
    pub fn new(values: BTreeMap<A, D>) -> Self {
        Self {
            address: Signal::default(),
            data: Signal::new_with_default(D::default()),
            clock: Signal::default(),
            _sim: values,
        }
    }
}

impl<A: Synth + Ord, D: Synth, F: Domain> Logic for SyncROM<A, D, F> {
    fn update(&mut self) {
        if self.clock.pos_edge() {
            self.data.next = Tagged(
                *self
                    ._sim
                    .get(&self.address.val().raw())
                    .unwrap_or(&D::default()),
                PhantomData,
            );
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
            Acount = (1 << A::BITS) - 1,
            init = init
        ))
    }
}
