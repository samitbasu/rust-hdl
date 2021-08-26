use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::rom::ROM;
use std::collections::BTreeMap;
use std::fs::File;

#[derive(LogicBlock)]
struct ROMTest {
    rom: ROM<Bits<4>, 4>,
}

impl ROMTest {
    pub fn new() -> ROMTest {
        let mut rom = BTreeMap::new();
        for i in 0_u32..16 {
            rom.insert(Bits::<4>::from(i), Bits::<4>::from(15 - i));
        }
        ROMTest { rom: ROM::new(rom) }
    }
}

impl Logic for ROMTest {
    fn update(&mut self) {}
}

#[test]
fn test_synthesis_rom() {
    let mut uut = ROMTest::new();
    uut.rom.address.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("rom", &vlog).unwrap();
}

#[test]
fn test_rom_works() {
    let mut sim = Simulation::new();
    sim.add_testbench(|mut sim: Sim<ROMTest>| {
        let mut x = sim.init()?;
        for i in 0_u32..16 {
            x.rom.address.next = Bits::<4>::from(i).into();
            x = sim.wait(1, x)?;
            assert_eq!(x.rom.data.val(), Bits::<4>::from(15 - i));
        }
        sim.done(x)?;
        Ok(())
    });
    let mut uut = ROMTest::new();
    uut.rom.address.connect();
    uut.connect_all();
    sim.run_traced(Box::new(uut), 100, File::create("ROM.vcd").unwrap())
        .unwrap();
}
