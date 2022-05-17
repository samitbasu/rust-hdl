use rust_hdl::core::prelude::*;

mod test_common;
use test_common::soc::SoCTestChip;

#[cfg(test)]
fn make_test_chip() -> SoCTestChip {
    let mut uut = SoCTestChip::default();
    uut.connect_all();
    uut
}

#[test]
fn test_soc_chip_works() {
    let uut = make_test_chip();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SoCTestChip>| x.clock.next = !x.clock.val());
    sim.add_clock(4, |x: &mut Box<SoCTestChip>| {
        x.sys_clock.next = !x.sys_clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SoCTestChip>| {
        let mut x = sim.init()?;
        // Send 10 pings
        // Send a PING command
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            wait_clock_cycles!(sim, clock, x, 20);
            // A ping is 0x01XX, where XX is the code returned by the controller
            x.from_cpu.data.next = (0x0167_u16 + iter).into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
            wait_clock_cycles!(sim, clock, x, 5);
            // Insert a NOOP
            x.from_cpu.data.next = 0_u16.into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
            wait_clock_cycles!(sim, clock, x, 5);
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<SoCTestChip>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            x = sim.watch(|x| !x.to_cpu.empty.val(), x)?;
            sim_assert!(sim, x.to_cpu.data.val() == (0x0167_u16 + iter), x);
            x.to_cpu.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.to_cpu.read.next = false;
        }
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        5_000,
        std::fs::File::create(vcd_path!("soc_chip_ping.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_soc_chip_read_write_works() {
    let uut = make_test_chip();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SoCTestChip>| x.clock.next = !x.clock.val());
    sim.add_clock(4, |x: &mut Box<SoCTestChip>| {
        x.sys_clock.next = !x.sys_clock.val()
    });
    let data_in = [0xDEAD_u16, 0xBEEF_u16, 0xCAFE_u16, 0xBABE_u16];
    sim.add_testbench(move |mut sim: Sim<SoCTestChip>| {
        let mut x = sim.init()?;
        // Send 10 pings
        // Send a PING command
        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 20);
        x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
        // Write the 4 data elements to port 0x00
        x.from_cpu.data.next = 0x0300_u16.into();
        x.from_cpu.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.from_cpu.write.next = false;
        x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
        x.from_cpu.data.next = 0x0004_u16.into();
        x.from_cpu.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.from_cpu.write.next = false;
        for datum in data_in.clone() {
            x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
            x.from_cpu.data.next = datum.into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
        }
        x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
        // Read the 4 data elements from port 0x01
        x.from_cpu.data.next = 0x0201_u16.into();
        x.from_cpu.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.from_cpu.write.next = false;
        x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
        x.from_cpu.data.next = 0x0004_u16.into();
        x.from_cpu.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.from_cpu.write.next = false;
        for datum in data_in.clone() {
            x = sim.watch(|x| !x.to_cpu.empty.val(), x)?;
            sim_assert!(sim, x.to_cpu.data.val() == (datum << 1), x);
            x.to_cpu.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.to_cpu.read.next = false;
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        50_000,
        std::fs::File::create(vcd_path!("soc_chip_pipe.vcd")).unwrap(),
    )
    .unwrap();
}
