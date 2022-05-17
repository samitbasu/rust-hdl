use crate::core::prelude::*;

#[derive(LogicBlock, Default)]
pub struct OpenDrainBuffer {
    pub bus: Signal<InOut, Bit>,
    pub enable: Signal<In, Bit>,
    pub read_data: Signal<Out, Bit>,
}

impl Logic for OpenDrainBuffer {
    fn update(&mut self) {
        if self.enable.val() {
            self.bus.next = false;
        }
        self.read_data.next = self.bus.val();
        self.bus.set_tristate_is_output(self.enable.val());
    }

    fn connect(&mut self) {
        self.bus.connect();
        self.read_data.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Custom(format!(
            "\
    assign bus = enable ? 0 : 1'bz;
    always @(*) read_data = bus;"
        ))
    }
}

#[test]
fn test_opendrain_synthesizes() {
    let mut uut = OpenDrainBuffer::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("open_drain", &vlog).unwrap()
}
