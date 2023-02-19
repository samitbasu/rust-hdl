use crate::core::prelude::*;
use rust_hdl_private_core::prelude::*;
use rust_hdl_private_widgets::prelude::*;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct OpalKellyBlinky {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>>,
    pub pulser: Pulser,
}

impl OpalKellyBlinky {
    pub fn new<B: OpalKellyBSP>() -> Self {
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            led: B::leds(),
            pulser: Pulser::new(MHZ48, 1.0, Duration::from_millis(500)),
        }
    }
}

impl Logic for OpalKellyBlinky {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        self.pulser.clock.next = self.ok_host.ti_clk.val();
        self.pulser.enable.next = true;
        if self.pulser.pulse.val() {
            self.led.next = 0xFF.into();
        } else {
            self.led.next = 0x00.into();
        }
    }
}
