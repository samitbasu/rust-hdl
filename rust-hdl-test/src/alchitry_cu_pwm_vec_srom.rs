use std::collections::BTreeMap;

use rust_hdl_core::check_connected::check_connected;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::prelude::*;

use crate::snore;
use rust_hdl_widgets::sync_rom::SyncROM;
use rust_hdl_alchitry_cu::ice_pll::ICE40PLLBlock;

#[derive(LogicBlock)]
pub struct FaderWithSyncROM<const F: u64> {
    pub clock: Signal<In, Clock<F>>,
    pub active: Signal<Out, Bit>,
    pub enable: Signal<In, Bit>,
    strobe: Strobe<32, {F}>,
    pwm: PulseWidthModulator<6, {F}>,
    rom: SyncROM<Bits<8>, Bits<6>, {F}>,
    counter: DFF<Bits<8>, {F}>,
}

impl<const F: u64> FaderWithSyncROM<F> {
    pub fn new(phase: u32) -> Self {
        let rom = (0..256_u32)
            .map(|x| (Bits::<8>::from(x), snore::snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            clock: Signal::default(),
            active: Signal::new_with_default(false),
            enable: Signal::default(),
            strobe: Strobe::new(120.0),
            pwm: PulseWidthModulator::default(),
            rom: SyncROM::new(rom),
            counter: DFF::new(Bits::<8>::default()),
        }
    }
}

impl<const F: u64> Logic for FaderWithSyncROM<F> {
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

#[derive(LogicBlock)]
pub struct AlchitryCuPWMVecSyncROM<const P: usize> {
    clock: Signal<In, Clock<100_000_000>>,
    leds: Signal<Out, Bits<8>>,
    local: Signal<Local, Bits<8>>,
    faders: [FaderWithSyncROM<25_000_000>; 8],
    pll: ICE40PLLBlock<100_000_000, 25_000_000>,
}

impl<const P: usize> Logic for AlchitryCuPWMVecSyncROM<P> {
    #[hdl_gen]
    fn update(&mut self) {
        self.pll.clock_in.next = self.clock.val();
        for i in 0_usize..8_usize {
            self.faders[i].clock.next = self.pll.clock_out.val();
            self.faders[i].enable.next = true;
        }
        self.local.next = 0x00_u8.into();
        for i in 0_usize..8_usize {
            self.local.next = self.local.val().replace_bit(i, self.faders[i].active.val());
        }
        self.leds.next = self.local.val();
    }
}

impl<const P: usize> Default for AlchitryCuPWMVecSyncROM<P> {
    fn default() -> Self {
        let faders : [FaderWithSyncROM<25_000_000>; 8] =
            [
                FaderWithSyncROM::new(0),
                FaderWithSyncROM::new(18),
                FaderWithSyncROM::new(36),
                FaderWithSyncROM::new(54),
                FaderWithSyncROM::new(72),
                FaderWithSyncROM::new(90),
                FaderWithSyncROM::new(108),
                FaderWithSyncROM::new(128),
            ];
        Self {
            clock: rust_hdl_alchitry_cu::pins::clock(),
            leds: rust_hdl_alchitry_cu::pins::leds(),
            local: Signal::default(),
            faders,
            pll: ICE40PLLBlock::default(),
        }
    }
}

#[test]
fn test_pwm_vec_sync_rom_synthesizes() {
    let mut uut : AlchitryCuPWMVecSyncROM<6> = AlchitryCuPWMVecSyncROM::default();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("pwm_cu_srom", &vlog).unwrap();
    rust_hdl_alchitry_cu::synth::generate_bitstream(uut, "pwm_cu_srom");
}
