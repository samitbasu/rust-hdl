use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct SPITestAsync {
    clock: Signal<In, Clock>,
    bus: SPIWiresMaster,
    master: SPIMaster<64>,
}

impl Logic for SPITestAsync {
    #[hdl_gen]
    fn update(&mut self) {
        SPIWiresMaster::link(&mut self.bus, &mut self.master.wires);
        clock!(self, clock, master);
    }
}

impl Default for SPITestAsync {
    fn default() -> Self {
        let config = SPIConfig {
            clock_speed: 100_000_000,
            cs_off: false,
            mosi_off: false,
            speed_hz: 2500000,
            cpha: false,
            cpol: false,
        };
        Self {
            clock: Default::default(),
            bus: Default::default(),
            master: SPIMaster::new(config),
        }
    }
}

#[test]
fn test_spi_txn_completes() {
    let mut uut = SPITestAsync::default();
    uut.master.bits_outbound.connect();
    uut.master.continued_transaction.connect();
    uut.master.data_outbound.connect();
    uut.master.start_send.connect();
    uut.connect_all();
    yosys_validate("spi_0", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SPITestAsync>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<SPITestAsync>| {
        let mut x = sim.init()?;
        wait_clock_cycles!(sim, clock, x, 4);
        wait_clock_true!(sim, clock, x);
        x.master.data_outbound.next = 0xDEADBEEF.into();
        x.master.bits_outbound.next = 32.into();
        x.master.start_send.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.master.start_send.next = false;
        x = sim.watch(|x| x.master.transfer_done.val().into(), x)?;
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create(vcd_path!("spi_txn.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock)]
struct SPITestPair {
    clock: Signal<In, Clock>,
    master: SPIMaster<64>,
    slave: SPISlave<64>,
}

impl SPITestPair {
    pub fn new(config: SPIConfig) -> Self {
        Self {
            clock: Default::default(),
            master: SPIMaster::new(config),
            slave: SPISlave::new(config),
        }
    }
}

impl Logic for SPITestPair {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, master, slave);
        SPIWiresMaster::join(&mut self.master.wires, &mut self.slave.wires);
    }
}

#[cfg(test)]
fn mk_spi_config(flags: [bool; 4]) -> SPIConfig {
    SPIConfig {
        clock_speed: 48_000_000,
        cs_off: flags[0],
        mosi_off: flags[1],
        speed_hz: 1_200_000,
        cpha: flags[2],
        cpol: flags[3],
    }
}

#[test]
fn test_spi_xchange_mode_0000() {
    test_spi_xchange(mk_spi_config([false, false, false, false]), "0000");
}

#[test]
fn test_spi_xchange_mode_0001() {
    test_spi_xchange(mk_spi_config([false, false, false, true]), "0001");
}

#[test]
fn test_spi_xchange_mode_0010() {
    test_spi_xchange(mk_spi_config([false, false, true, false]), "0010");
}

#[test]
fn test_spi_xchange_mode_0011() {
    test_spi_xchange(mk_spi_config([false, false, true, true]), "0011");
}

#[test]
fn test_spi_xchange_mode_0100() {
    test_spi_xchange(mk_spi_config([false, true, false, false]), "0100");
}

#[test]
fn test_spi_xchange_mode_0101() {
    test_spi_xchange(mk_spi_config([false, true, false, true]), "0101");
}

#[test]
fn test_spi_xchange_mode_0110() {
    test_spi_xchange(mk_spi_config([false, true, true, false]), "0110");
}

#[test]
fn test_spi_xchange_mode_0111() {
    test_spi_xchange(mk_spi_config([false, true, true, true]), "0111");
}

#[test]
fn test_spi_xchange_mode_1000() {
    test_spi_xchange(mk_spi_config([true, false, false, false]), "1000");
}

#[test]
fn test_spi_xchange_mode_1001() {
    test_spi_xchange(mk_spi_config([true, false, false, true]), "1001");
}

#[test]
fn test_spi_xchange_mode_1010() {
    test_spi_xchange(mk_spi_config([true, false, true, false]), "1010");
}

#[test]
fn test_spi_xchange_mode_1011() {
    test_spi_xchange(mk_spi_config([true, false, true, true]), "1011");
}

#[test]
fn test_spi_xchange_mode_1100() {
    test_spi_xchange(mk_spi_config([true, true, false, false]), "1100");
}

#[test]
fn test_spi_xchange_mode_1101() {
    test_spi_xchange(mk_spi_config([true, true, false, true]), "1101");
}

#[test]
fn test_spi_xchange_mode_1110() {
    test_spi_xchange(mk_spi_config([true, true, true, false]), "1110");
}

#[test]
fn test_spi_xchange_mode_1111() {
    test_spi_xchange(mk_spi_config([true, true, true, true]), "1111");
}

#[cfg(test)]
fn test_spi_xchange(config: SPIConfig, name: &str) {
    let mut uut = SPITestPair::new(config);
    uut.master.continued_transaction.connect();
    uut.master.start_send.connect();
    uut.master.data_outbound.connect();
    uut.master.bits_outbound.connect();
    uut.slave.data_outbound.connect();
    uut.slave.start_send.connect();
    uut.slave.continued_transaction.connect();
    uut.slave.disabled.connect();
    uut.slave.bits.connect();
    uut.connect_all();
    yosys_validate(&format!("spi_{}", name), &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SPITestPair>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<SPITestPair>| {
        let mut x = sim.init()?;
        wait_clock_cycles!(sim, clock, x, 16);
        for _ in 0..4 {
            wait_clock_true!(sim, clock, x);
            x.master.data_outbound.next = 0xDEADBEEF.into();
            x.master.bits_outbound.next = 32.into();
            x.master.start_send.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.master.start_send.next = false;
            x = sim.watch(|x| x.master.transfer_done.val().into(), x)?;
            sim_assert_eq!(sim, x.master.data_inbound.val(), 0xCAFEBABE_u64, x);
            wait_clock_cycle!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<SPITestPair>| {
        let mut x = sim.init()?;
        wait_clock_cycles!(sim, clock, x, 16);
        for _ in 0..4 {
            wait_clock_true!(sim, clock, x);
            x.slave.data_outbound.next = 0xCAFEBABE.into();
            x.slave.bits.next = 32.into();
            x.slave.start_send.next = true;
            wait_clock_cycle!(sim, clock, x);
            x = sim.watch(|x| x.slave.transfer_done.val().into(), x)?;
            sim_assert_eq!(sim, x.slave.data_inbound.val(), 0xDEADBEEF_u64, x);
            sim_assert_eq!(sim, x.slave.bits.val(), 32, x);
        }
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 1_000_000, &vcd_path!(format!("spi_xfer_test_{}.vcd", name))).unwrap();
    //sim.run(Box::new(uut), 1_000_000).unwrap();
}
