use crate::test_common::FaderWithSyncROM;
use rust_hdl::core::prelude::*;
use rust_hdl_ok_core::core::prelude::*;

#[derive(LogicBlock)]
pub struct OpalKellyWave {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>>,
    pub local: Signal<Local, Bits<8>>,
    pub faders: [FaderWithSyncROM; 8],
}

impl Logic for OpalKellyWave {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        for i in 0_usize..8_usize {
            self.faders[i].clock.next = self.ok_host.ti_clk.val();
            self.faders[i].enable.next = true;
        }
        self.local.next = 0x00_u8.into();
        for i in 0_usize..8_usize {
            self.local.next = self
                .local
                .val()
                .replace_bit(i, !self.faders[i].active.val());
        }
        self.led.next = self.local.val();
    }
}

impl OpalKellyWave {
    pub fn new<B: OpalKellyBSP>() -> Self {
        let faders: [FaderWithSyncROM; 8] = [
            FaderWithSyncROM::new(MHZ48, 0),
            FaderWithSyncROM::new(MHZ48, 18),
            FaderWithSyncROM::new(MHZ48, 36),
            FaderWithSyncROM::new(MHZ48, 54),
            FaderWithSyncROM::new(MHZ48, 72),
            FaderWithSyncROM::new(MHZ48, 90),
            FaderWithSyncROM::new(MHZ48, 108),
            FaderWithSyncROM::new(MHZ48, 128),
        ];
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            local: Signal::default(),
            faders,
            led: B::leds(),
        }
    }
}
