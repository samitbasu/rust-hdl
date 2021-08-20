use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::spi_master::{SPIConfig, SPIMaster, SPIWires};

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
    uut.bus.link_connect();
    uut.master.bits_outbound.connect();
    uut.master.continued_transaction.connect();
    uut.master.data_outbound.connect();
    uut.master.start_send.connect();
    uut.connect_all();
    yosys_validate("spi_0", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut SPITestAsync| x.clock.next = !x.clock.val());
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
    sim.run_traced(uut, 100_000, std::fs::File::create("spi_txn.vcd").unwrap())
        .unwrap();
}
