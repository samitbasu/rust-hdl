use rust_hdl_core::prelude::*;
use rust_hdl_widgets::sync_rom::SyncROM;
use rust_hdl_yosys_synth::yosys_validate;
use std::collections::BTreeMap;

#[derive(LogicBlock)]
struct SyncROMTest {
    rom: SyncROM<Bits<4>, 4>,
}

impl SyncROMTest {
    pub fn new() -> SyncROMTest {
        let mut rom = BTreeMap::new();
        for i in 0_u32..16 {
            rom.insert(Bits::<4>::from(i), Bits::<4>::from(15 - i));
        }
        SyncROMTest {
            rom: SyncROM::new(rom),
        }
    }
}

impl Logic for SyncROMTest {
    fn update(&mut self) {}
}

#[test]
fn test_synthesis_sync_rom() {
    let mut uut = SyncROMTest::new();
    uut.rom.address.connect();
    uut.rom.clock.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("srom", &vlog).unwrap();
}
