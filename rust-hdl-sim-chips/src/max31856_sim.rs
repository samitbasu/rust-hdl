use crate::ad7193_sim::AD7193Config;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;
use std::num::Wrapping;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum MAX31856State {
    Start,
    Ready,
    GettingCmd,
    ReadCmd,
    WaitSlaveIdle,
    WriteCmd,
    DoWrite,
}

#[derive(LogicBlock)]
pub struct MAX31856Simulator {
    // Slave SPI bus
    pub mosi: Signal<In, Bit>,
    pub mclk: Signal<In, Bit>,
    pub msel: Signal<In, Bit>,
    pub miso: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    // RAM that stores the memory contents
    reg_ram: RAM<Bits<8>, 4>,
    // Used to handle auto conversions
    auto_conversions_enabled: DFF<Bit>,
    auto_conversion_strobe: Strobe<32>,
    auto_conversion_counter: DFF<Bits<19>>,
    // Separate bits out of the SPI message
    cmd: Signal<Local, Bits<8>>,
    reg_index: Signal<Local, Bits<4>>,
    rw_flag: Signal<Local, Bit>,
    // The SPI slave device
    spi_slave: SPISlave<64>,
    // FSM state:
    state: DFF<MAX31856State>,
    reg_write_index: DFF<Bits<4>>,
    // Boot timer
    boot: DFF<Bits<4>>,
}

const MAX31856_REG_INITS: [u8; 16] = [
    0x00, 0x03, 0xFF, 0x7F, 0xC0, 0x7F, 0xFF, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

impl MAX31856Simulator {
    pub fn new(config: SPIConfig) -> Self {
        let reg_ram = MAX31856_REG_INITS
            .iter()
            .map(|x| Bits::<8>::from(*x))
            .into();
        Self {
            mosi: Default::default(),
            mclk: Default::default(),
            msel: Default::default(),
            miso: Default::default(),
            clock: Default::default(),
            reg_ram,
            auto_conversions_enabled: Default::default(),
            auto_conversion_strobe: Strobe::new(config.clock_speed, 10.0),
            auto_conversion_counter: Default::default(),
            cmd: Default::default(),
            reg_index: Default::default(),
            rw_flag: Default::default(),
            spi_slave: SPISlave::new(config),
            state: DFF::new(MAX31856State::Start),
            reg_write_index: Default::default(),
            boot: DFF::default(),
        }
    }
}

impl Logic for MAX31856Simulator {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the spi bus
        self.spi_slave.mosi.next = self.mosi.val();
        self.spi_slave.mclk.next = self.mclk.val();
        self.spi_slave.msel.next = self.msel.val();
        self.miso.next = self.spi_slave.miso.val();
        // Clock the internal logic
        self.reg_ram.write_clock.next = self.clock.val();
        self.reg_ram.read_clock.next = self.clock.val();
        self.auto_conversions_enabled.clk.next = self.clock.val();
        self.auto_conversion_strobe.clock.next = self.clock.val();
        self.auto_conversion_counter.clk.next = self.clock.val();
        self.spi_slave.clock.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.reg_write_index.clk.next = self.clock.val();
        self.boot.clk.next = self.clock.val();
        // Latch prevention
        self.auto_conversions_enabled.d.next = self.auto_conversions_enabled.q.val();
        self.auto_conversion_counter.d.next = self.auto_conversion_counter.q.val();
        self.state.d.next = self.state.q.val();
        self.reg_write_index.d.next = self.reg_write_index.q.val();
        // Set default values
        self.spi_slave.start_send.next = false;
        self.spi_slave.continued_transaction.next = false;
        self.spi_slave.bits.next = 0_u16.into();
        self.spi_slave.data_outbound.next = 0_u64.into();
        self.reg_ram.write_enable.next = false;
        self.spi_slave.disabled.next = false;
        self.cmd.next = self.spi_slave.data_inbound.val().get_bits::<8>(0_usize);
        self.rw_flag.next = self.cmd.val().get_bit(7_usize);
        self.reg_index.next = self.cmd.val().get_bits::<4>(0_usize);
        self.reg_ram.read_address.next = self.reg_index.val();
        self.reg_ram.write_address.next = self.reg_write_index.q.val();
        self.reg_ram.write_data.next = self.spi_slave.data_inbound.val().get_bits::<8>(0_usize);
        self.auto_conversion_strobe.enable.next = self.auto_conversions_enabled.q.val();
        match self.state.q.val() {
            MAX31856State::Start => {
                self.boot.d.next = self.boot.q.val() + 1_u32;
                if self.boot.q.val().all() {
                    self.state.d.next = MAX31856State::Ready
                }
            }
            MAX31856State::Ready => {
                self.spi_slave.continued_transaction.next = true;
                self.spi_slave.bits.next = 8_u16.into();
                self.spi_slave.data_outbound.next = 0xFF_u64.into();
                self.spi_slave.start_send.next = true;
                self.state.d.next = MAX31856State::GettingCmd;
            }
            MAX31856State::GettingCmd => {
                if self.spi_slave.transfer_done.val() {
                    if !self.rw_flag.val() {
                        self.state.d.next = MAX31856State::ReadCmd;
                    } else {
                        self.reg_write_index.d.next = self.reg_index.val();
                        self.state.d.next = MAX31856State::WriteCmd;
                    }
                }
            }
            MAX31856State::ReadCmd => {
                self.spi_slave.continued_transaction.next = true;
                self.spi_slave.bits.next = 8_usize.into();
                self.spi_slave.data_outbound.next = bit_cast::<64, 8>(self.reg_ram.read_data.val());
                self.spi_slave.start_send.next = true;
                self.state.d.next = MAX31856State::WaitSlaveIdle;
            }
            MAX31856State::WaitSlaveIdle => {
                if !self.spi_slave.busy.val() {
                    self.state.d.next = MAX31856State::Ready;
                }
            }
            MAX31856State::WriteCmd => {
                self.spi_slave.continued_transaction.next = true;
                self.spi_slave.bits.next = 8_usize.into();
                self.spi_slave.data_outbound.next = 0xFF_u8.into();
                self.spi_slave.start_send.next = true;
                self.state.d.next = MAX31856State::DoWrite;
            }
            MAX31856State::DoWrite => {
                if self.spi_slave.transfer_done.val() {
                    self.reg_ram.write_enable.next = true;
                    self.state.d.next = MAX31856State::WaitSlaveIdle;
                }
            }
        }
    }
}

#[test]
fn test_max31856_synthesizes() {
    use rust_hdl_yosys_synth::yosys_validate;
    let mut uut = MAX31856Simulator::new(SPIConfig {
        clock_speed: 1_000_000,
        cs_off: true,
        mosi_off: true,
        speed_hz: 10_000,
        cpha: true,
        cpol: true,
    });
    uut.mosi.connect();
    uut.mclk.connect();
    uut.msel.connect();
    uut.clock.connect();
    uut.connect_all();
    yosys_validate("max31856", &generate_verilog(&uut)).unwrap();
}

#[derive(LogicBlock)]
struct Test31856 {
    clock: Signal<In, Clock>,
    master: SPIMaster<64>,
    uut: MAX31856Simulator,
}

impl Logic for Test31856 {
    #[hdl_gen]
    fn update(&mut self) {
        self.master.clock.next = self.clock.val();
        self.uut.clock.next = self.clock.val();
        self.uut.mosi.next = self.master.wires.mosi.val();
        self.uut.msel.next = self.master.wires.msel.val();
        self.uut.mclk.next = self.master.wires.mclk.val();
        self.master.wires.miso.next = self.uut.miso.val();
    }
}

impl Default for Test31856 {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            master: SPIMaster::new(AD7193Config::sw().spi),
            uut: MAX31856Simulator::new(AD7193Config::sw().spi),
        }
    }
}

fn reg_read(
    reg_index: u32,
    x: Box<Test31856>,
    sim: &mut Sim<Test31856>,
) -> Result<(Bits<64>, Box<Test31856>), SimError> {
    let cmd = (reg_index << 8) as u64;
    let result = do_spi_txn(16, cmd, false, x, sim)?;
    let reg_val = result.0 & 0xFF_usize;
    Ok((reg_val, result.1))
}

fn reg_write(
    reg_index: u32,
    reg_value: u64,
    x: Box<Test31856>,
    sim: &mut Sim<Test31856>,
) -> Result<Box<Test31856>, SimError> {
    let mut cmd = (((1 << 7) | reg_index) << 8) as u64;
    cmd = cmd | (reg_value & 0xFF);
    let ret = do_spi_txn(16, cmd, false, x, sim)?;
    Ok(ret.1)
}

fn do_spi_txn(
    bits: u16,
    value: u64,
    continued: bool,
    mut x: Box<Test31856>,
    sim: &mut Sim<Test31856>,
) -> Result<(Bits<64>, Box<Test31856>), SimError> {
    wait_clock_true!(sim, clock, x);
    wait_clock_cycles!(sim, clock, x, 10);
    x.master.data_outbound.next = value.into();
    x.master.bits_outbound.next = bits.into();
    x.master.continued_transaction.next = continued;
    x.master.start_send.next = true;
    wait_clock_cycle!(sim, clock, x);
    x.master.start_send.next = false;
    x = sim
        .watch(
            |x| x.clock.val().0 && x.master.transfer_done.val().into(),
            x,
        )
        .unwrap();
    let ret = x.master.data_inbound.val();
    wait_clock_true!(sim, clock, x);
    wait_clock_cycles!(sim, clock, x, 50);
    Ok((ret, x))
}

#[cfg(test)]
fn mk_test31856() -> Test31856 {
    let mut uut = Test31856::default();
    uut.clock.connect();
    uut.master.continued_transaction.connect();
    uut.master.start_send.connect();
    uut.master.data_outbound.connect();
    uut.master.bits_outbound.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_yosys_validate_test_fixture() {
    use rust_hdl_yosys_synth::yosys_validate;
    let uut = mk_test31856();
    yosys_validate("31856_1", &generate_verilog(&uut)).unwrap();
}

#[test]
fn test_reg_reads() {
    let uut = mk_test31856();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Test31856>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<Test31856>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 20);
        for ndx in 0..16 {
            println!("Reading register index {}", ndx);
            let result = reg_read(ndx, x, &mut sim)?;
            x = result.1;
            println!("Value {} -> {:x}", ndx, result.0);
            sim_assert!(
                sim,
                result.0 == Bits::<64>::from(MAX31856_REG_INITS[ndx as usize]),
                x
            );
            wait_clock_true!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.run(Box::new(uut), 1_000_000).unwrap();
}

#[test]
fn test_reg_writes() {
    let uut = mk_test31856();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Test31856>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<Test31856>| {
        let mut x = sim.init()?;
        // Initialize the chip...
        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 20);
        for ndx in 0..16 {
            let result = reg_read(ndx, x, &mut sim)?;
            x = result.1;
            sim_assert!(
                sim,
                result.0 == Bits::<64>::from(MAX31856_REG_INITS[ndx as usize]),
                x
            );
            println!("Read of register {} -> {:x}", ndx, result.0);
            x = reg_write(
                ndx,
                (MAX31856_REG_INITS[ndx as usize] as u64 + 1) as u64,
                x,
                &mut sim,
            )?;
            let result = reg_read(ndx, x, &mut sim)?;
            x = result.1;
            sim_assert!(
                sim,
                result.0
                    == Bits::<64>::from(Wrapping(MAX31856_REG_INITS[ndx as usize]) + Wrapping(1)),
                x
            );
            println!("Re-read of register {} -> {:x}", ndx, result.0);
        }
        sim.done(x)
    });
    sim.run(Box::new(uut), 1_000_000).unwrap();
}

/*
#[test]
fn test_single_conversion() {
    let uut = mk_test31856();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Test31856>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<Test31856>| {
        let mut x = sim.init()?;
        // Initialize the chip...
        let result = do_spi_txn(32, 0xFFFFFFFF_u64, false, x, &mut sim)?;
        x = result.1;
        for n in 0..3 {
            wait_clock_cycle!(sim, clock, x, 100);
            let result = do_spi_txn(32, 0x8382006, true, x, &mut sim)?;
            x = result.1;
            wait_clock_cycle!(sim, clock, x, 100);
            sim_assert!(sim, x.master.wires.miso.val(), x);
            x = sim.watch(|x| !x.master.wires.miso.val(), x)?;
            wait_clock_cycle!(sim, clock, x, 100);
            let result = reg_read(3, x, &mut sim)?;
            println!("Conversion {} -> {:x}", n, result.0);
            x = result.1;
            sim_assert!(sim, result.0 == Bits::<64>::from((n * 0x100) as u32), x);
            println!("Conversion {} completed", n);
        }
        sim.done(x)
    });
    sim.run(Box::new(uut), 10_000_000).unwrap();
}


 */
