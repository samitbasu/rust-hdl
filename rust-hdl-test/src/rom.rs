use rust_hdl_widgets::rom::ROM;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;

#[derive(LogicBlock)]
struct ROMTest {
    rom: ROM<Bits<4>, 4>
}

impl ROMTest {
    pub fn new() -> ROMTest {
        let rom = [
            15_u32, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0
        ].iter().map(|x| (*x).into()).collect::<Vec<Bits<4>>>();
        ROMTest {
            rom: ROM::new(rom)
        }
    }
}

impl Logic for ROMTest {
    fn update(&mut self) {
    }
}

#[test]
fn test_synthesis_rom() {
    let mut uut = ROMTest::new();
    uut.rom.address.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("rom", &vlog);
}
