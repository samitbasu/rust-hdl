use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct MISOPortTest {
    bus_m: LocalBusM<16, 8>,
    port_a: MISOPort<16, 8>,
    port_b: MISOPort<16, 8>,
    clock: Signal<In, Clock>,
}

impl Logic for MISOPortTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.port_a.clock.next = self.clock.val();
        self.port_b.clock.next = self.clock.val();
        // Input ports for port_a
        self.port_a.bus.strobe.next = self.bus_m.strobe.val();
        self.port_a.bus.addr.next = self.bus_m.addr.val();
        self.port_a.bus.from_master.next = self.bus_m.from_master.val();
        // Input ports for port_b
        self.port_b.bus.strobe.next = self.bus_m.strobe.val();
        self.port_b.bus.addr.next = self.bus_m.addr.val();
        self.port_b.bus.from_master.next = self.bus_m.from_master.val();
        // Output ports
        self.bus_m.to_master.next =
            self.port_a.bus.to_master.val() | self.port_b.bus.to_master.val();
        self.bus_m.ready.next = self.port_a.bus.ready.val() | self.port_b.bus.ready.val();
    }
}

impl Default for MISOPortTest {
    fn default() -> Self {
        Self {
            bus_m: Default::default(),
            port_a: MISOPort::new(34_usize.into()),
            port_b: MISOPort::new(36_usize.into()),
            clock: Default::default(),
        }
    }
}

#[test]
fn test_port_test_synthesizes() {
    let mut uut = MISOPortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.port_a.ready_in.connect();
    uut.port_b.ready_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("test_port", &vlog).unwrap();
}

#[test]
fn test_port_test_works() {
    let mut uut = MISOPortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.port_a.ready_in.connect();
    uut.port_b.ready_in.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MISOPortTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus_m.addr.next = 34_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        sim_assert!(sim, x.bus_m.to_master.val() == 0xDEAD_u16, x);
        x.bus_m.addr.next = 36_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        sim_assert!(sim, x.bus_m.to_master.val() == 0xCAFE_u16, x);
        wait_clock_cycles!(sim, clock, x, 50);
        x.bus_m.addr.next = 34_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        sim_assert!(sim, x.bus_m.to_master.val() == 0xBEEF_u16, x);
        x.bus_m.addr.next = 36_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        sim_assert!(sim, x.bus_m.to_master.val() == 0xBABE_u16, x);
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        x.port_a.ready_in.next = true;
        wait_clock_true!(sim, clock, x);
        x.port_a.port_in.next = 0xDEAD_u16.into();
        wait_clock_cycles!(sim, clock, x, 50);
        x.port_a.port_in.next = 0xBEEF_u16.into();
        wait_clock_cycles!(sim, clock, x, 50);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        x.port_b.ready_in.next = true;
        wait_clock_true!(sim, clock, x);
        x.port_b.port_in.next = 0xCAFE_u16.into();
        wait_clock_cycles!(sim, clock, x, 50);
        x.port_b.port_in.next = 0xBABE_u16.into();
        wait_clock_cycles!(sim, clock, x, 50);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        2000,
        std::fs::File::create(vcd_path!("miso_port.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock)]
struct MISOWidePortTest {
    bus_m: LocalBusM<16, 8>,
    port_a: MISOWidePort<64, 16, 8>,
    port_b: MISOWidePort<64, 16, 8>,
    clock: Signal<In, Clock>,
}

impl Logic for MISOWidePortTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.port_a.clock.next = self.clock.val();
        self.port_b.clock.next = self.clock.val();
        // Input ports for port_a
        self.port_a.bus.strobe.next = self.bus_m.strobe.val();
        self.port_a.bus.addr.next = self.bus_m.addr.val();
        self.port_a.bus.from_master.next = self.bus_m.from_master.val();
        // Input ports for port_b
        self.port_b.bus.strobe.next = self.bus_m.strobe.val();
        self.port_b.bus.addr.next = self.bus_m.addr.val();
        self.port_b.bus.from_master.next = self.bus_m.from_master.val();
        // Output ports
        self.bus_m.to_master.next =
            self.port_a.bus.to_master.val() | self.port_b.bus.to_master.val();
        self.bus_m.ready.next = self.port_a.bus.ready.val() | self.port_b.bus.ready.val();
    }
}

impl Default for MISOWidePortTest {
    fn default() -> Self {
        Self {
            bus_m: Default::default(),
            port_a: MISOWidePort::new(34_usize.into()),
            port_b: MISOWidePort::new(36_usize.into()),
            clock: Default::default(),
        }
    }
}

#[test]
fn test_wide_port_test_synthesizes() {
    let mut uut = MISOWidePortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.port_a.strobe_in.connect();
    uut.port_b.strobe_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("test_wide_port", &vlog).unwrap();
}

#[test]
fn test_wide_port_test_works() {
    let mut uut = MISOWidePortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.port_a.strobe_in.connect();
    uut.port_b.strobe_in.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MISOWidePortTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MISOWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus_m.addr.next = 34_usize.into();
        wait_clock_cycle!(sim, clock, x);
        for val in [0xDEAD_u16, 0xBEEF, 0x1234, 0xABCD] {
            x = sim.watch(|x| x.bus_m.ready.val(), x)?;
            sim_assert!(sim, x.bus_m.to_master.val() == val, x);
            x.bus_m.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.bus_m.strobe.next = false;
        }
        wait_clock_cycle!(sim, clock, x);
        x.bus_m.addr.next = 36_usize.into();
        wait_clock_cycle!(sim, clock, x);
        for val in [0xCAFE_u16, 0xFEED, 0xBABE, 0x5EA1] {
            x = sim.watch(|x| x.bus_m.ready.val(), x)?;
            sim_assert!(sim, x.bus_m.to_master.val() == val, x);
            x.bus_m.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.bus_m.strobe.next = false;
        }
        wait_clock_cycle!(sim, clock, x);
        x.bus_m.addr.next = 34_usize.into();
        wait_clock_cycle!(sim, clock, x);
        for val in [0x0123_u16, 0x4567, 0x89AB, 0xCDEF] {
            x = sim.watch(|x| x.bus_m.ready.val(), x)?;
            sim_assert!(sim, x.bus_m.to_master.val() == val, x);
            x.bus_m.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.bus_m.strobe.next = false;
        }
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 25);
        x.port_a.port_in.next = 0xDEADBEEF1234ABCD_u64.into();
        x.port_a.strobe_in.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.port_a.strobe_in.next = false;
        wait_clock_cycles!(sim, clock, x, 50);
        x.port_a.port_in.next = 0x0123_4567_89AB_CDEF_u64.into();
        x.port_a.strobe_in.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.port_a.strobe_in.next = false;
        wait_clock_cycles!(sim, clock, x, 50);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.port_b.port_in.next = 0xCAFEFEEDBABE5EA1_u64.into();
        x.port_b.strobe_in.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.port_a.strobe_in.next = false;
        wait_clock_cycles!(sim, clock, x, 50);
        sim.done(x)
    });
    let ret = sim.run_traced(
        Box::new(uut),
        5000,
        std::fs::File::create(vcd_path!("miso_wide_port.vcd")).unwrap(),
    );
    ret.unwrap();
}

#[derive(LogicBlock)]
struct MISOPortFIFOTest {
    bus_m: LocalBusM<16, 8>,
    port_a: MISOPort<16, 8>,
    fifo: SynchronousFIFO<Bits<16>, 2, 3, 1>,
    clock: Signal<In, Clock>,
}

impl Logic for MISOPortFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.port_a.bus.from_master.next = self.bus_m.from_master.val();
        self.port_a.bus.strobe.next = self.bus_m.strobe.val();
        self.port_a.bus.addr.next = self.bus_m.addr.val();
        self.bus_m.to_master.next = self.port_a.bus.to_master.val();
        self.bus_m.ready.next = self.port_a.bus.ready.val();
        self.port_a.clock.next = self.clock.val();
        self.fifo.clock.next = self.clock.val();
        self.port_a.port_in.next = self.fifo.data_out.val();
        self.port_a.ready_in.next = !self.fifo.empty.val();
        self.fifo.read.next = self.port_a.strobe_out.val();
    }
}

impl Default for MISOPortFIFOTest {
    fn default() -> Self {
        Self {
            bus_m: Default::default(),
            port_a: MISOPort::new(53_u8.into()),
            fifo: Default::default(),
            clock: Default::default(),
        }
    }
}

#[test]
fn test_miso_fifo_synthesizes() {
    let mut uut = MISOPortFIFOTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("miso_fifo", &vlog).unwrap();
}

#[test]
fn test_miso_fifo_works() {
    let mut uut = MISOPortFIFOTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    let test_data = [0xDEAD_u16, 0xBEEF, 0xCAFE, 0xBABE, 0x1234, 0x5678, 0x5423];
    sim.add_clock(5, |x: &mut Box<MISOPortFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for val in test_data.clone() {
            x = sim.watch(|x| !x.fifo.full.val(), x)?;
            x.fifo.data_in.next = val.into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false;
        }
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 10);
        x.bus_m.addr.next = 53_u8.into();
        for val in test_data.clone() {
            x = sim.watch(|x| x.bus_m.ready.val(), x)?;
            sim_assert!(sim, x.bus_m.to_master.val() == val, x);
            x.bus_m.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.bus_m.strobe.next = false;
        }
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create(vcd_path!("miso_fifo.vcd")).unwrap(),
    )
    .unwrap();
}
