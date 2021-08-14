use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::ram::RAM;
use std::collections::BTreeMap;

make_domain!(Mhz1, 1_000_000);

#[derive(LogicBlock)]
struct RAMTest {
    pub clock: Signal<In, Clock, Mhz1>,
    pub ram: RAM<Bits<5>, Bits<16>, Mhz1, Mhz1>,
}

impl RAMTest {
    pub fn new() -> RAMTest {
        Self {
            clock: Signal::default(),
            ram: RAM::new(Default::default()),
        }
    }
}

impl Logic for RAMTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.ram.write.clock.next = self.clock.val();
        self.ram.read.clock.next = self.clock.val();
    }
}

#[test]
fn test_synthesis_ram() {
    let mut uut = RAMTest::new();
    uut.clock.connect();
    uut.ram.write.enable.connect();
    uut.ram.write.data.connect();
    uut.ram.write.address.connect();
    uut.ram.read.address.connect();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("ram", &vlog).unwrap();
}

#[test]
fn test_ram_works() {
    let mut uut = RAMTest::new();
    uut.clock.connect();
    uut.ram.write.enable.connect();
    uut.ram.write.data.connect();
    uut.ram.write.address.connect();
    uut.ram.read.address.connect();
    uut.connect_all();
    yosys_validate("ram", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..32)
        .map(|_| Bits::<16>::from(rand::random::<u16>()))
        .collect::<Vec<_>>();
    sim.add_clock(5, |x: &mut RAMTest| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<RAMTest>| {
        println!("Init test bench");
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in rdata.iter().enumerate() {
            x.ram.write.address.next = (sample.0 as u32).into();
            x.ram.write.data.next = (*sample.1).into();
            x.ram.write.enable.next = true.into();
            wait_clock_cycle!(sim, clock, x);
        }
        x.ram.write.enable.next = false.into();
        wait_clock_cycle!(sim, clock, x);
        for sample in rdata.iter().enumerate() {
            x.ram.read.address.next = (sample.0 as u32).into();
            wait_clock_cycle!(sim, clock, x);
            assert_eq!(x.ram.read.data.val().raw(), *sample.1);
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(uut, 512 * 10, std::fs::File::create("ram.vcd").unwrap())
        .unwrap();
}
