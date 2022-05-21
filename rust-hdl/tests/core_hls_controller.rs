use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;

#[derive(LogicBlock)]
struct ControllerTest {
    to_cpu: FIFOReadController<Bits<16>>,
    from_cpu: FIFOWriteController<Bits<16>>,
    to_cpu_fifo: SyncFIFO<Bits<16>, 6, 7, 1>,
    from_cpu_fifo: SyncFIFO<Bits<16>, 6, 7, 1>,
    controller: BaseController<2>,
    bridge: Bridge<16, 2, 2>,
    port: MOSIPort<16>,
    iport: MISOPort<16>,
    clock: Signal<In, Clock>,
}

impl Default for ControllerTest {
    fn default() -> Self {
        Self {
            to_cpu: Default::default(),
            from_cpu: Default::default(),
            to_cpu_fifo: Default::default(),
            from_cpu_fifo: Default::default(),
            controller: Default::default(),
            bridge: Bridge::new(["port", "iport"]),
            port: Default::default(),
            iport: Default::default(),
            clock: Default::default(),
        }
    }
}

impl Logic for ControllerTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clocks
        clock!(self, clock, to_cpu_fifo, from_cpu_fifo, controller);
        // Connect the test interfaces
        FIFOWriteController::<Bits<16>>::join(
            &mut self.from_cpu,
            &mut self.from_cpu_fifo.bus_write,
        );
        FIFOReadResponder::<Bits<16>>::join(
            &mut self.from_cpu_fifo.bus_read,
            &mut self.controller.from_cpu,
        );
        FIFOReadController::<Bits<16>>::join(&mut self.to_cpu, &mut self.to_cpu_fifo.bus_read);
        FIFOWriteResponder::<Bits<16>>::join(
            &mut self.to_cpu_fifo.bus_write,
            &mut self.controller.to_cpu,
        );
        // Connect the controller to the bridge
        SoCBusController::<16, 2>::join(&mut self.controller.bus, &mut self.bridge.upstream);
        // Connect the MOSI port to node 0 of the bridge
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.port.bus);
        SoCPortController::<16>::join(&mut self.bridge.nodes[1], &mut self.iport.bus);
        self.port.ready.next = true;
    }
}

#[cfg(test)]
fn make_controller_test() -> ControllerTest {
    let mut uut = ControllerTest::default();
    uut.clock.connect();
    uut.from_cpu.data.connect();
    uut.from_cpu.write.connect();
    uut.to_cpu.read.connect();
    uut.iport.port_in.connect();
    uut.iport.ready_in.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_controller_test_synthesizes() {
    let uut = make_controller_test();
    let vlog = generate_verilog(&uut);
    yosys_validate("controller", &vlog).unwrap();
}

#[test]
fn test_ping_works() {
    let uut = make_controller_test();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<ControllerTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<ControllerTest>| {
        let mut x = sim.init()?;
        // Send a PING command
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            wait_clock_cycles!(sim, clock, x, 5);
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
    sim.add_testbench(move |mut sim: Sim<ControllerTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            x = sim.watch(|x| !x.to_cpu.empty.val(), x)?;
            sim_assert!(sim, x.to_cpu.data.val() == (0x0167_u16 + iter), x);
            x.to_cpu.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.to_cpu.read.next = false;
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        5000,
        std::fs::File::create(vcd_path!("controller_ping.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_write_command_works() {
    let uut = make_controller_test();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<ControllerTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<ControllerTest>| {
        let mut x = sim.init()?;
        // Send a PING command
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            wait_clock_cycles!(sim, clock, x, 5);
            // A write command looks like 0x03XXYYYY, where XX is the address, YYYY is the count
            // followed by count data elements.
            // Write the command
            x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
            x.from_cpu.data.next = 0x0300_u16.into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
            // Then the count
            x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
            x.from_cpu.data.next = (iter + 1).into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
            // Then the data elements
            for ndx in 0..(iter + 1) {
                x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
                x.from_cpu.data.next = (0x7870_u16 + ndx).into();
                x.from_cpu.write.next = true;
                wait_clock_cycle!(sim, clock, x);
                x.from_cpu.write.next = false;
            }
            // Insert a NOOPd
            x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
            x.from_cpu.data.next = 0_u16.into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
            wait_clock_cycles!(sim, clock, x, 5);
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<ControllerTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            for ndx in 0..(iter + 1) {
                x = sim.watch(|x| x.port.strobe_out.val(), x)?;
                sim_assert!(sim, x.port.port_out.val() == (0x7870_u32 + ndx), x);
                wait_clock_cycle!(sim, clock, x);
            }
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        5000,
        std::fs::File::create(vcd_path!("controller_write.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_read_command_works() {
    let uut = make_controller_test();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<ControllerTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<ControllerTest>| {
        let mut x = sim.init()?;
        // Send a PING command
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            wait_clock_cycles!(sim, clock, x, 5);
            // A read command looks like 0x02XXYYYY, where XX is the address, YYYY is the count
            // Write the command
            x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
            x.from_cpu.data.next = 0x0201_u16.into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
            // Then the count
            x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
            x.from_cpu.data.next = (iter + 1).into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
            // Then wait for the data elements to come back to the CPU
            for ndx in 0..(iter + 1) {
                x = sim.watch(|x| !x.to_cpu.empty.val(), x)?;
                sim_assert!(sim, x.to_cpu.data.val() == 0xBEE0_u16 + ndx, x);
                x.to_cpu.read.next = true;
                wait_clock_cycle!(sim, clock, x);
                x.to_cpu.read.next = false;
            }
            // Wait 1 clock cycle, and then issue a POLL command
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.data.next = 0x0401_u16.into();
            x.from_cpu.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.from_cpu.write.next = false;
            // Read the result of the poll back
            x = sim.watch(|x| !x.to_cpu.empty.val(), x)?;
            // Port should always be ready
            sim_assert!(sim, x.to_cpu.data.val() == 0xFF01_u16, x);
            x.to_cpu.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.to_cpu.read.next = false;
            wait_clock_cycles!(sim, clock, x, 5);
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<ControllerTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for iter in 0..10 {
            wait_clock_cycles!(sim, clock, x, 10);
            for ndx in 0..(iter + 1) {
                x.iport.port_in.next = (0xBEE0_u16 + ndx).into();
                x.iport.ready_in.next = true;
                x = sim.watch(|x| x.iport.strobe_out.val(), x)?;
                wait_clock_cycle!(sim, clock, x);
            }
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        20000,
        std::fs::File::create(vcd_path!("controller_read.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_stream_command_works() {
    let uut = make_controller_test();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<ControllerTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<ControllerTest>| {
        let mut x = sim.init()?;
        // Send a PING command
        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 5);
        // A stream command looks like 0x05XX, where XX is the address to stream from
        // Write the command
        x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
        x.from_cpu.data.next = 0x0501_u16.into();
        x.from_cpu.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.from_cpu.write.next = false;
        // Wait until we have collected 100 items
        for iter in 0..100 {
            x = sim.watch(|x| !x.to_cpu.empty.val(), x)?;
            sim_assert!(sim, x.to_cpu.data.val() == 0xBAB0_u16 + iter, x);
            x.to_cpu.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.to_cpu.read.next = false;
        }
        // Send a stop command (anything non-zero)
        x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
        x.from_cpu.data.next = 0x0501_u16.into();
        x.from_cpu.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.from_cpu.write.next = false;
        // There may be extra data that comes, so discard data until the
        // CPU fifo is empty...
        while !x.to_cpu.empty.val() {
            x.to_cpu.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.to_cpu.read.next = false;
        }
        // Send a ping
        x = sim.watch(|x| !x.from_cpu.full.val(), x)?;
        x.from_cpu.data.next = 0x01FF_u16.into();
        x.from_cpu.write.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.from_cpu.write.next = false;
        // Wait for it to return
        x = sim.watch(|x| !x.to_cpu.empty.val(), x)?;
        sim_assert!(sim, x.to_cpu.data.val() == 0x01FF_u16, x);
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<ControllerTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for ndx in 0..100 {
            x.iport.port_in.next = (0xBAB0_u16 + ndx).into();
            x.iport.ready_in.next = true;
            x = sim.watch(|x| x.iport.strobe_out.val(), x)?;
            wait_clock_cycle!(sim, clock, x);
            x.iport.ready_in.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        50000,
        std::fs::File::create(vcd_path!("controller_stream.vcd")).unwrap(),
    )
    .unwrap();
}
