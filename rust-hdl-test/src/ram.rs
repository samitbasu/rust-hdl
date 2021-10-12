use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::ram::RAM;

#[derive(LogicBlock)]
struct RAMTest {
    pub clock: Signal<In, Clock>,
    pub ram: RAM<Bits<16>, 5>,
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
        self.ram.write_clock.next = self.clock.val();
        self.ram.read_clock.next = self.clock.val();
    }
}

#[test]
fn test_synthesis_ram() {
    let mut uut = RAMTest::new();
    uut.clock.connect();
    uut.ram.write_enable.connect();
    uut.ram.write_data.connect();
    uut.ram.write_address.connect();
    uut.ram.read_address.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("ram_2", &vlog).unwrap();
}

#[test]
fn test_ram_works() {
    let mut uut = RAMTest::new();
    uut.clock.connect();
    uut.ram.write_enable.connect();
    uut.ram.write_data.connect();
    uut.ram.write_address.connect();
    uut.ram.read_address.connect();
    uut.connect_all();
    yosys_validate("ram", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..32)
        .map(|_| Bits::<16>::from(rand::random::<u16>()))
        .collect::<Vec<_>>();
    sim.add_clock(5, |x: &mut Box<RAMTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<RAMTest>| {
        println!("Init test bench");
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in rdata.iter().enumerate() {
            x.ram.write_address.next = (sample.0 as u32).into();
            x.ram.write_data.next = (*sample.1).into();
            x.ram.write_enable.next = true;
            wait_clock_cycle!(sim, clock, x);
        }
        x.ram.write_enable.next = false.into();
        wait_clock_cycle!(sim, clock, x);
        for sample in rdata.iter().enumerate() {
            x.ram.read_address.next = (sample.0 as u32).into();
            wait_clock_cycle!(sim, clock, x);
            assert_eq!(x.ram.read_data.val(), *sample.1);
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        512 * 10,
        std::fs::File::create("ram.vcd").unwrap(),
    )
    .unwrap();
}
