use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

mod test_common;
use test_common::soc::SoCTestChip;

#[test]
fn test_soc_chip_works() {
    let mut uut = SoCTestChip::default();
    uut.sys_clock.connect();
    uut.clock.connect();
    uut.cpu_bus.write.connect();
    uut.cpu_bus.to_controller.connect();
    uut.cpu_bus.read.connect();
    uut.connect_all();
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
            wait_clock_cycles!(sim, clock, x, 5);
            // A ping is 0x01XX, where XX is the code returned by the controller
            x.cpu_bus.to_controller.next = (0x0167_u16 + iter).into();
            x.cpu_bus.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.cpu_bus.write.next = false;
            wait_clock_cycles!(sim, clock, x, 5);
            // Insert a NOOP
            x.cpu_bus.to_controller.next = 0_u16.into();
            x.cpu_bus.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.cpu_bus.write.next = false;
            wait_clock_cycles!(sim, clock, x, 5);
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<SoCTestChip>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            x = sim.watch(|x| !x.cpu_bus.empty.val(), x)?;
            sim_assert!(
                sim,
                x.cpu_bus.from_controller.val() == (0x0167_u16 + iter),
                x
            );
            x.cpu_bus.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.cpu_bus.read.next = false;
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
    let mut uut = SoCTestChip::default();
    uut.sys_clock.connect();
    uut.clock.connect();
    uut.cpu_bus.write.connect();
    uut.cpu_bus.to_controller.connect();
    uut.cpu_bus.read.connect();
    uut.connect_all();
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
        wait_clock_cycles!(sim, clock, x, 5);
        x = sim.watch(|x| !x.cpu_bus.full.val(), x)?;
        // Write the 4 data elements to port 0x53
        x.cpu_bus.to_controller.next = 0x0353_u16.into();
        x.cpu_bus.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.cpu_bus.write.next = false;
        x = sim.watch(|x| !x.cpu_bus.full.val(), x)?;
        x.cpu_bus.to_controller.next = 0x0004_u16.into();
        x.cpu_bus.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.cpu_bus.write.next = false;
        for datum in data_in.clone() {
            x = sim.watch(|x| !x.cpu_bus.full.val(), x)?;
            x.cpu_bus.to_controller.next = datum.into();
            x.cpu_bus.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.cpu_bus.write.next = false;
        }
        x = sim.watch(|x| !x.cpu_bus.full.val(), x)?;
        // Read the 4 data elements from port 0x54
        x.cpu_bus.to_controller.next = 0x0254_u16.into();
        x.cpu_bus.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.cpu_bus.write.next = false;
        x = sim.watch(|x| !x.cpu_bus.full.val(), x)?;
        x.cpu_bus.to_controller.next = 0x0004_u16.into();
        x.cpu_bus.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.cpu_bus.write.next = false;
        for datum in data_in.clone() {
            x = sim.watch(|x| !x.cpu_bus.empty.val(), x)?;
            sim_assert!(sim, x.cpu_bus.from_controller.val() == (datum << 1), x);
            x.cpu_bus.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.cpu_bus.read.next = false;
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
