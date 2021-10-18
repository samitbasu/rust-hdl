use std::collections::BTreeMap;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;

use crate::snore::snore;

#[derive(LogicBlock)]
pub struct FaderWithSyncROM {
    pub clock: Signal<In, Clock>,
    pub active: Signal<Out, Bit>,
    pub enable: Signal<In, Bit>,
    strobe: Strobe<32>,
    pwm: PulseWidthModulator<6>,
    rom: SyncROM<Bits<6>, 8>,
    counter: DFF<Bits<8>>,
}

impl FaderWithSyncROM {
    pub fn new(clock_frequency: u64, phase: u32) -> Self {
        let rom = (0..256_u32)
            .map(|x| (Bits::<8>::from(x), snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            clock: Signal::default(),
            active: Signal::new_with_default(false),
            enable: Signal::default(),
            strobe: Strobe::new(clock_frequency, 120.0),
            pwm: PulseWidthModulator::default(),
            rom: SyncROM::new(rom),
            counter: DFF::new(Bits::<8>::default()),
        }
    }
}

impl Logic for FaderWithSyncROM {
    #[hdl_gen]
    fn update(&mut self) {
        self.strobe.clock.next = self.clock.val();
        self.pwm.clock.next = self.clock.val();
        self.counter.clk.next = self.clock.val();
        self.rom.clock.next = self.clock.val();
        self.rom.address.next = self.counter.q.val();
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
        self.strobe.enable.next = self.enable.val();
        self.pwm.enable.next = self.enable.val();
        self.active.next = self.pwm.active.val();
        self.pwm.threshold.next = self.rom.data.val();
    }
}
