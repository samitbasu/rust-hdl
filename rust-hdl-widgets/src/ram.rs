use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;

#[derive(LogicInterface, Default)]
pub struct RAMWrite<D: Synth, const N: usize> {
    pub address: Signal<In, Bits<N>>,
    pub clock: Signal<In, Clock>,
    pub data: Signal<In, D>,
    pub enable: Signal<In, bool>,
}

#[derive(LogicBlock, Default)]
pub struct RAM<D: Synth, const N: usize> {
    pub read_address: Signal<In, Bits<N>>,
    pub read_clock: Signal<In, Clock>,
    pub read_data: Signal<Out, D>,
    pub write_address: Signal<In, Bits<N>>,
    pub write_clock: Signal<In, Clock>,
    pub write_data: Signal<In, D>,
    pub write_enable: Signal<In, bool>,
    _sim: BTreeMap<Bits<N>, D>,
}

impl<D: Synth, const N: usize> RAM<D, N> {
    pub fn new(values: BTreeMap<Bits<N>, D>) -> Self {
        Self {
            _sim: values,
            ..Default::default()
        }
    }
}

impl<D: Synth, const N: usize> Logic for RAM<D, N> {
    fn update(&mut self) {
        if self.read_clock.pos_edge() {
            self.read_data.next = *self
                ._sim
                .get(&self.read_address.val())
                .unwrap_or(&D::default());
        }
        if self.write_clock.pos_edge() && self.write_enable.val() {
            self._sim
                .insert(self.write_address.val(), self.write_data.val());
        }
    }

    fn connect(&mut self) {
        self.read_data.connect();
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
