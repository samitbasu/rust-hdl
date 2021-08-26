use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::spi_master::{SPIConfig, SPIMaster, SPIWires};
use rust_hdl_widgets::spi_slave::SPISlave;

#[derive(LogicBlock)]
struct SPITestAsync {
    clock: Signal<In, Clock>,
    bus: SPIWires,
    master: SPIMaster<64>,
}

impl Logic for SPITestAsync {
    #[hdl_gen]
    fn update(&mut self) {
        self.bus.link(&mut self.master.wires);
        self.master.clock.next = self.clock.val();
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
    uut.clock.connect();
    uut.bus.link_connect_dest();
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
        wait_clock_true!(sim, clock, x);
        x.master.data_outbound.next = 0xDEADBEEF_u32.into();
        x.master.bits_outbound.next = 32_usize.into();
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
        std::fs::File::create("spi_txn.vcd").unwrap(),
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
        self.master.clock.next = self.clock.val();
        self.slave.clock.next = self.clock.val();
        self.slave.mosi.next = self.master.wires.mosi.val();
        self.slave.mclk.next = self.master.wires.mclk.val();
        self.slave.msel.next = self.master.wires.msel.val();
        self.master.wires.miso.next = self.slave.miso.val();
    }
}

#[cfg(test)]
fn mk_spi_config(flags: [bool; 3]) -> SPIConfig {
    SPIConfig {
        clock_speed: 48_000_000,
        cs_off: flags[0],
        mosi_off: false,
        speed_hz: 1_000_000,
        cpha: flags[1],
        cpol: flags[2],
    }
}

#[test]
fn test_spi_xchange_modes() {
    test_spi_xchange(mk_spi_config([false, false, false]));
    test_spi_xchange(mk_spi_config([false, false, true]));
    test_spi_xchange(mk_spi_config([false, true, false]));
    test_spi_xchange(mk_spi_config([false, true, true]));
    test_spi_xchange(mk_spi_config([true, false, false]));
    test_spi_xchange(mk_spi_config([true, false, true]));
    test_spi_xchange(mk_spi_config([true, true, false]));
    test_spi_xchange(mk_spi_config([true, true, true]));
}

#[cfg(test)]
fn test_spi_xchange(config: SPIConfig) {
    let mut uut = SPITestPair::new(config);
    uut.clock.connect();
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
    yosys_validate("spi_1", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SPITestPair>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<SPITestPair>| {
        let mut x = sim.init()?;
        for _ in 0..4 {
            wait_clock_true!(sim, clock, x);
            x.master.data_outbound.next = 0xDEADBEEF_u32.into();
            x.master.bits_outbound.next = 32_usize.into();
            x.master.start_send.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.master.start_send.next = false;
            x = sim.watch(|x| x.master.transfer_done.val().into(), x)?;
            sim_assert!(sim, x.master.data_inbound.val() == 0xCAFEBABE_u32, x);
            wait_clock_cycle!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<SPITestPair>| {
        let mut x = sim.init()?;
        for _ in 0..4 {
            wait_clock_true!(sim, clock, x);
            x.slave.data_outbound.next = 0xCAFEBABE_u32.into();
            x.slave.bits.next = 32_usize.into();
            x.slave.start_send.next = true;
            wait_clock_cycle!(sim, clock, x);
            x = sim.watch(|x| x.slave.transfer_done.val().into(), x)?;
            sim_assert!(sim, x.slave.data_inbound.val() == 0xDEADBEEF_u32, x);
            sim_assert!(sim, x.slave.bits.val() == 32_u32, x);
        }
        sim.done(x)
    });
    //    sim.run_traced(uut, 1_000_000, std::fs::File::create("spi_x1.vcd").unwrap()).unwrap()
    sim.run(Box::new(uut), 1_000_000).unwrap();
}
