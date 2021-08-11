use rust_hdl_core::prelude::*;
use std::collections::BTreeMap;
use std::marker::PhantomData;

#[derive(LogicBlock)]
pub struct RAM<A: Synth + Ord, D: Synth, R: Domain, W: Domain> {
    pub read_address: Signal<In, A, R>,
    pub read_clock: Signal<In, Clock, R>,
    pub read_data: Signal<Out, D, R>,
    pub write_address: Signal<In, A, W>,
    pub write_clock: Signal<In, Clock, W>,
    pub write_data: Signal<In, D, W>,
    pub write_enable: Signal<In, bool, W>,
    _sim: BTreeMap<A, D>,
}

impl<A: Synth + Ord, D: Synth, R: Domain, W: Domain> RAM<A, D, R, W> {
    pub fn new(values: BTreeMap<A, D>) -> Self {
        Self {
            read_address: Default::default(),
            read_clock: Default::default(),
            read_data: Default::default(),
            write_address: Default::default(),
            write_clock: Default::default(),
            write_data: Default::default(),
            write_enable: Default::default(),
            _sim: values,
        }
    }
}

impl<A: Synth + Ord, D: Synth, R: Domain, W: Domain> Logic for RAM<A, D, R, W> {
    fn update(&mut self) {
        if self.read_clock.pos_edge() {
            self.read_data.next = Tagged(
                *self
                    ._sim
                    .get(&self.read_address.val().raw())
                    .unwrap_or(&D::default()),
                PhantomData,
            );
        }
        if self.write_clock.pos_edge() && self.write_enable.val().raw() {
            self._sim
                .insert(self.write_address.val().raw(), self.write_data.val().raw());
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
            Acount = (1 << A::BITS) - 1,
            init = init
        ))
    }
}
