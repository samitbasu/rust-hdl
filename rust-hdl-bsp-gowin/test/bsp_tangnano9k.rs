use rust_hdl::prelude::*;
use rust_hdl_bsp_gowin::tangnano9k::{pins, synth};

#[derive(LogicBlock)]
pub struct Blinky {
    pulser: Pulser,
    clock: Signal<In, Clock>,
    lamp: Signal<Out, Bits<6>>,
}

impl Default for Blinky {
    fn default() -> Self {
        Blinky {
            pulser: Pulser::new(
                pins::CLOCK_SPEED_27MHZ,
                1.0,
                std::time::Duration::from_millis(250),
            ),
            clock: pins::clock(),
            lamp: pins::lamp(),
        }
    }
}

impl Logic for Blinky {
    #[hdl_gen]
    fn update(&mut self) {
        self.pulser.enable.next = true;
        self.pulser.clock.next = self.clock.val();

        self.lamp.next = 0.into();
        if self.pulser.pulse.val() {
            self.lamp.next = 0xff.into();
        }
    }
}

#[test]
fn main() {
    synth::generate_bitstream(
        Blinky::default(),
        true, // auto download
        target_path!("tangnano9k/blink"),
        "", // yosys(or oss-cad-suite) path, "" or same "/home/xxx/software/oss-cad-suite/bin"
    );
}
