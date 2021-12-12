use crate::core::prelude::*;

#[derive(LogicBlock, Default)]
pub struct TristateBuffer<D: Synth> {
    pub bus: Signal<InOut, D>,
    pub write_enable: Signal<In, Bit>,
    pub write_data: Signal<In, D>,
    pub read_data: Signal<Out, D>,
}

impl<D: Synth> Logic for TristateBuffer<D> {
    fn update(&mut self) {
        if self.write_enable.val() {
            self.bus.next = self.write_data.val();
        }
        self.read_data.next = self.bus.val();
        self.bus.set_tristate_is_output(self.write_enable.val());
    }

    fn connect(&mut self) {
        self.bus.connect();
        self.read_data.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Custom(format!("\
    assign bus = write_enable ? write_data : {WIDTH}'bz;
always @(*) read_data = bus;", WIDTH = D::BITS))
    }
}

#[test]
fn test_tristate_synthesizes() {
    let mut uut = TopWrap::new(TristateBuffer::<Bits<8>>::default());
    uut.uut.write_data.connect();
    uut.uut.write_enable.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("tristate", &vlog).unwrap()
}