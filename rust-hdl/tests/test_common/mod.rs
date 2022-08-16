#![allow(dead_code)]

use std::collections::BTreeMap;
use std::f64::consts::PI;

use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
use rust_hdl::widgets::test_helpers::snore;

pub mod fifo_tester;
pub mod soc;

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
    #[cfg(test)]
    pub fn new(clock_frequency: u64, phase: u32) -> Self {
        let rom = (0..256)
            .map(|x| (x.to_bits(), snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            clock: Signal::default(),
            active: Signal::new_with_default(false),
            enable: Signal::default(),
            strobe: Strobe::new(clock_frequency, 120.0),
            pwm: PulseWidthModulator::default(),
            rom: SyncROM::new(rom),
            counter: Default::default(),
        }
    }
}

impl Logic for FaderWithSyncROM {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, strobe, pwm, counter);
        self.rom.clock.next = self.clock.val();
        self.rom.address.next = self.counter.q.val();
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
        self.strobe.enable.next = self.enable.val();
        self.pwm.enable.next = self.enable.val();
        self.active.next = self.pwm.active.val();
        self.pwm.threshold.next = self.rom.data.val();
    }
}
