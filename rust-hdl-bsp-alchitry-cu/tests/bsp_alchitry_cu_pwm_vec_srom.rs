#[cfg(test)]
use rust_hdl::prelude::*;
use rust_hdl_bsp_alchitry_cu::{pins, synth};
use rust_hdl_fpga_support::lattice::ice40::ice_pll::ICE40PLLBlock;

const MHZ25: u64 = 25_000_000;
const MHZ100: u64 = 100_000_000;

#[derive(LogicBlock)]
pub struct AlchitryCuPWMVecSyncROM<const P: usize> {
    clock: Signal<In, Clock>,
    leds: Signal<Out, Bits<8>>,
    local: Signal<Local, Bits<8>>,
    faders: [FaderWithSyncROM; 8],
    pll: ICE40PLLBlock<MHZ100, MHZ25>,
}

impl<const P: usize> Logic for AlchitryCuPWMVecSyncROM<P> {
    #[hdl_gen]
    fn update(&mut self) {
        self.pll.clock_in.next = self.clock.val();
        for i in 0..8 {
            self.faders[i].clock.next = self.pll.clock_out.val();
            self.faders[i].enable.next = self.pll.locked.val();
        }
        self.local.next = 0x00.into();
        for i in 0..8 {
            self.local.next = self.local.val().replace_bit(i, self.faders[i].active.val());
        }
        self.leds.next = self.local.val();
    }
}

impl<const P: usize> AlchitryCuPWMVecSyncROM<P> {
    fn new(clock_frequency: u64) -> Self {
        let faders: [FaderWithSyncROM; 8] = [
            FaderWithSyncROM::new(clock_frequency, 0),
            FaderWithSyncROM::new(clock_frequency, 18),
            FaderWithSyncROM::new(clock_frequency, 36),
            FaderWithSyncROM::new(clock_frequency, 54),
            FaderWithSyncROM::new(clock_frequency, 72),
            FaderWithSyncROM::new(clock_frequency, 90),
            FaderWithSyncROM::new(clock_frequency, 108),
            FaderWithSyncROM::new(clock_frequency, 128),
        ];
        Self {
            clock: pins::clock(),
            leds: pins::leds(),
            local: Signal::default(),
            faders,
            pll: ICE40PLLBlock::default(),
        }
    }
}

#[test]
fn test_pwm_vec_sync_rom_synthesizes() {
    let mut uut: AlchitryCuPWMVecSyncROM<6> = AlchitryCuPWMVecSyncROM::new(25_000_000);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("pwm_cu_srom", &vlog).unwrap();
    synth::generate_bitstream(uut, target_path!("alchitry_cu/pwm_cu_srom"));
}
