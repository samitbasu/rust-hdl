use rust_hdl_private_core::prelude::*;

#[derive(LogicBlock, Default)]
pub struct OpenDrainBuffer {
    pub bus: Signal<InOut, Bit>,
    pub control: OpenDrainReceiver,
}

impl Logic for OpenDrainBuffer {
    fn update(&mut self) {
        if self.control.drive_low.val() {
            self.bus.next = false;
        }
        self.control.line_state.next = self.bus.val();
        self.bus
            .set_tristate_is_output(self.control.drive_low.val());
    }

    fn connect(&mut self) {
        self.bus.connect();
        self.control.line_state.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Custom(format!(
            "\
    assign bus = control$drive_low ? 0 : 1'bz;
    always @(*) control$line_state = bus;"
        ))
    }
}

#[test]
fn test_opendrain_synthesizes() {
    let mut uut = OpenDrainBuffer::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("open_drain", &vlog).unwrap()
}

#[derive(LogicInterface, Default)]
#[join = "OpenDrainReceiver"]
pub struct OpenDrainDriver {
    pub drive_low: Signal<Out, Bit>,
    pub line_state: Signal<In, Bit>,
}

#[derive(LogicInterface, Default)]
#[join = "OpenDrainDriver"]
pub struct OpenDrainReceiver {
    pub drive_low: Signal<In, Bit>,
    pub line_state: Signal<Out, Bit>,
}
