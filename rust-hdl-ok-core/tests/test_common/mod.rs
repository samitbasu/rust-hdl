#![allow(dead_code)]

use std::collections::BTreeMap;
use std::f64::consts::PI;

use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

pub mod blinky;

pub mod ddr;

pub mod download;
pub mod fifo_tester;

pub mod fir;

pub mod mux_spi;

pub mod pipe;
pub mod soc;

pub mod spi;

pub mod tools;

pub mod wave;

pub mod wire;

pub fn snore<const P: usize>(x: u32) -> Bits<P> {
    let amp = (f64::exp(f64::sin(((x as f64) - 128.0 / 2.) * PI / 128.0)) - 0.36787944) * 108.0;
    let amp = (amp.max(0.0).min(255.0).floor() / 255.0 * (1 << P) as f64) as u8;
    amp.into()
}

#[derive(LogicBlock)]
pub struct FaderWithSyncROM {
    pub reset: Signal<In, Reset>,
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
        let rom = (0..256_u32)
            .map(|x| (Bits::<8>::from(x), snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            reset: Default::default(),
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
        clock_reset!(self, clock, reset, strobe, pwm, counter);
        self.rom.clock.next = self.clock.val();
        self.rom.address.next = self.counter.q.val();
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
        self.strobe.enable.next = self.enable.val();
        self.pwm.enable.next = self.enable.val();
        self.active.next = self.pwm.active.val();
        self.pwm.threshold.next = self.rom.data.val();
    }
}
