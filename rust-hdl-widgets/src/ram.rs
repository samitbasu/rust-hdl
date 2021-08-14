use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;

#[derive(LogicInterface, Default)]
pub struct RAMRead<D: Synth, const N: usize> {
    pub address: Signal<In, Bits<N>>,
    pub clock: Signal<In, Clock>,
    pub data: Signal<Out, D>,
}

#[derive(LogicInterface, Default)]
pub struct RAMWrite<D: Synth, const N: usize> {
    pub address: Signal<In, Bits<N>>,
    pub clock: Signal<In, Clock>,
    pub data: Signal<In, D>,
    pub enable: Signal<In, bool>,
}

#[derive(LogicBlock, Default)]
pub struct RAM<D: Synth, const N: usize> {
    pub read: RAMRead<D, N>,
    pub write: RAMWrite<D, N>,
    _sim: BTreeMap<Bits<N>, D>,
}

impl<D: Synth, const N: usize> RAM<D, N> {
    pub fn new(values: BTreeMap<Bits<N>, D>) -> Self {
        Self {
            read: Default::default(),
            write: Default::default(),
            _sim: values,
        }
    }
}

impl<D: Synth, const N: usize> Logic for RAM<D, N> {
    fn update(&mut self) {
        if self.read.clock.pos_edge() {
            self.read.data.next = *self
                ._sim
                .get(&self.read.address.val())
                .unwrap_or(&D::default());
        }
        if self.write.clock.pos_edge() && self.write.enable.val() {
            self._sim
                .insert(self.write.address.val(), self.write.data.val());
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
            Acount = (1 << N) - 1,
            init = init
        ))
    }
}
