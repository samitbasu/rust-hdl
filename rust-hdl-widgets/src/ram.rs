use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;
use std::marker::PhantomData;

#[derive(LogicInterface)]
pub struct RAMRead<A: Synth + Ord, D: Synth, R: Domain> {
    pub address: Signal<In, A, R>,
    pub clock: Signal<In, Clock, R>,
    pub data: Signal<Out, D, R>,
}

impl<A: Synth + Ord, D: Synth, R: Domain> Default for RAMRead<A, D, R> {
    fn default() -> Self {
        Self {
            address: Default::default(),
            clock: Default::default(),
            data: Default::default(),
        }
    }
}

#[derive(LogicInterface)]
pub struct RAMWrite<A: Synth + Ord, D: Synth, W: Domain> {
    pub address: Signal<In, A, W>,
    pub clock: Signal<In, Clock, W>,
    pub data: Signal<In, D, W>,
    pub enable: Signal<In, bool, W>,
}

impl<A: Synth + Ord, D: Synth, W: Domain> Default for RAMWrite<A, D, W> {
    fn default() -> Self {
        Self {
            address: Default::default(),
            clock: Default::default(),
            data: Default::default(),
            enable: Default::default(),
        }
    }
}

#[derive(LogicBlock, Default)]
pub struct RAM<A: Synth + Ord, D: Synth, R: Domain, W: Domain> {
    pub read: RAMRead<A, D, R>,
    pub write: RAMWrite<A, D, W>,
    _sim: BTreeMap<A, D>,
}

impl<A: Synth + Ord, D: Synth, R: Domain, W: Domain> RAM<A, D, R, W> {
    pub fn new(values: BTreeMap<A, D>) -> Self {
        Self {
            read: Default::default(),
            write: Default::default(),
            _sim: values,
        }
    }
}

impl<A: Synth + Ord, D: Synth, R: Domain, W: Domain> Logic for RAM<A, D, R, W> {
    fn update(&mut self) {
        if self.read.clock.pos_edge() {
            self.read.data.next = Tagged(
                *self
                    ._sim
                    .get(&self.read.address.val().raw())
                    .unwrap_or(&D::default()),
                PhantomData,
            );
        }
        if self.write.clock.pos_edge() && self.write.enable.val().raw() {
            self._sim
                .insert(self.write.address.val().raw(), self.write.data.val().raw());
        }
    }

    fn connect(&mut self) {
        self.read.data.connect();
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
reg[{D}:0] mem[{Acount}:0];

initial begin
{init};
end

always @(posedge read_clock) begin
   read_data <= mem[read_address];
end

always @(posedge write_clock) begin
   if (write_enable) begin
      mem[write_address] <= write_data;
   end
end
            ",
            D = D::BITS - 1,
            Acount = (1 << A::BITS) - 1,
            init = init
        ))
    }
}
