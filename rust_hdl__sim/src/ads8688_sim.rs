use rust_hdl__core::prelude::*;
use rust_hdl__widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Init,
    Ready,
    GettingCmd,
    WaitSlaveIdle,
    DoWrite,
    DoRead,
    DoNoop,
    DoCommand,
    ReadBack,
    DoConvert,
}

#[derive(LogicBlock)]
pub struct ADS8688Simulator {
    pub wires: SPIWiresSlave,
    pub clock: Signal<In, Clock>,
    // RAM that stores the register contents (these are Program Registers)
    reg_ram: RAM<Bits<8>, 6>,
    // Used to split bits out of the SPI message
    // Organized as: <cmd_flag> <reg_address> <r/w>
    reg_address: Signal<Local, Bits<6>>,
    rw_flag: Signal<Local, Bit>,
    cmd_flag: Signal<Local, Bit>,
    noop_flag: Signal<Local, Bit>,
    // The SPI slave device
    spi_slave: SPISlave<64>,
    // FSM state
    state: DFF<State>,
    // FLOP to hold the register address
    reg_address_flop: DFF<Bits<6>>,
    // FLOP to hold the last written value
    readback_flop: DFF<Bits<8>>,
    // Rolling counter to emulate conversions
    conversion_counter: DFF<Bits<12>>,
    // Command register
    command_register: DFF<Bits<8>>,
    // Output register for reading
    output_register: DFF<Bits<16>>,
}

fn ram_init_vec() -> Vec<u8> {
    // Create the initial contents of the RAM.
    // The following registers are 0xFF, all others are 0x00 by default
    // 1, 0x16, 0x17, 0x16 + 5, 0x17 + 5, ... (8 times)
    let mut ram_init = vec![0_u8; 0x40];
    ram_init[1] = 0xFF;
    for ch in 0..8 {
        ram_init[0x15 + ch * 5 + 1] = 0xFF;
        ram_init[0x15 + ch * 5 + 2] = 0xFF;
    }
    ram_init
}

#[test]
fn test_ram_init_vec_is_correct() {
    let vec = ram_init_vec();
    assert_eq!(vec[1], 0xFF);
    assert_eq!(vec[0x16], 0xFF);
    assert_eq!(vec[0x17], 0xFF);
    assert_eq!(vec[0x39], 0xFF);
    assert_eq!(vec[0x3A], 0xFF);
}

impl ADS8688Simulator {
    pub fn new(config: SPIConfig) -> Self {
        assert!(config.clock_speed > 10 * config.speed_hz);
        let reg_ram = ram_init_vec().iter().map(|x| x.to_bits()).into();
        Self {
            wires: Default::default(),
            clock: Default::default(),
            reg_ram,
            reg_address: Default::default(),
            rw_flag: Default::default(),
            cmd_flag: Default::default(),
            noop_flag: Default::default(),
            spi_slave: SPISlave::new(config),
            state: Default::default(),
            reg_address_flop: Default::default(),
            readback_flop: Default::default(),
            conversion_counter: Default::default(),
            command_register: Default::default(),
            output_register: Default::default(),
        }
    }
}

impl Logic for ADS8688Simulator {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the spi bus
        SPIWiresSlave::link(&mut self.wires, &mut self.spi_slave.wires);
        // Clock internal components
        self.reg_ram.read_clock.next = self.clock.val();
        self.reg_ram.write_clock.next = self.clock.val();
        clock!(self, clock, spi_slave);
        dff_setup!(
            self,
            clock,
            state,
            readback_flop,
            reg_address_flop,
            conversion_counter,
            output_register,
            command_register
        );
        // Set default values, and unpack the command word
        self.spi_slave.start_send.next = false;
        self.cmd_flag.next = self.spi_slave.data_inbound.val().get_bit(7);
        self.reg_address.next = self.spi_slave.data_inbound.val().get_bits::<6>(1);
        self.rw_flag.next = self.spi_slave.data_inbound.val().get_bit(0);
        self.noop_flag.next = !self.spi_slave.data_inbound.val().get_bits::<8>(0).any();
        self.reg_ram.read_address.next = self.reg_address_flop.q.val();
        self.reg_ram.write_address.next = self.reg_address_flop.q.val();
        self.spi_slave.continued_transaction.next = false;
        self.spi_slave.bits.next = 0.into();
        self.spi_slave.data_outbound.next = 0.into();
        self.reg_ram.write_enable.next = false;
        self.reg_ram.write_data.next = 0.into();
        self.spi_slave.disabled.next = false;
        match self.state.q.val() {
            State::Init => {
                if !self.spi_slave.busy.val() {
                    self.state.d.next = State::Ready;
                }
            }
            State::Ready => {
                self.spi_slave.continued_transaction.next = true;
                self.spi_slave.bits.next = 8.into();
                self.spi_slave.data_outbound.next = 0x00.into();
                self.spi_slave.start_send.next = true;
                self.state.d.next = State::GettingCmd;
            }
            State::GettingCmd => {
                self.reg_address_flop.d.next = self.reg_address.val();
                if self.spi_slave.transfer_done.val() {
                    self.state.d.next = State::WaitSlaveIdle;
                    if self.noop_flag.val() {
                        self.state.d.next = State::DoNoop;
                    } else {
                        if self.cmd_flag.val() {
                            self.state.d.next = State::DoCommand;
                        } else {
                            if self.rw_flag.val() {
                                self.spi_slave.continued_transaction.next = true;
                                self.spi_slave.bits.next = 8.into();
                                self.spi_slave.start_send.next = true;
                                self.state.d.next = State::DoWrite;
                            } else {
                                self.spi_slave.continued_transaction.next = true;
                                self.spi_slave.bits.next = 8.into();
                                self.spi_slave.start_send.next = true;
                                self.state.d.next = State::DoRead;
                            }
                        }
                    }
                }
            }
            State::WaitSlaveIdle => {
                if !self.spi_slave.busy.val() {
                    self.state.d.next = State::Ready;
                }
            }
            State::DoRead => {
                if self.spi_slave.transfer_done.val() {
                    self.spi_slave.continued_transaction.next = true;
                    self.spi_slave.bits.next = 8.into();
                    self.spi_slave.data_outbound.next =
                        bit_cast::<64, 8>(self.reg_ram.read_data.val());
                    self.spi_slave.start_send.next = true;
                    self.state.d.next = State::WaitSlaveIdle;
                }
            }
            State::DoWrite => {
                if self.spi_slave.transfer_done.val() {
                    self.reg_ram.write_data.next =
                        self.spi_slave.data_inbound.val().get_bits::<8>(0);
                    self.readback_flop.d.next = self.spi_slave.data_inbound.val().get_bits::<8>(0);
                    self.reg_ram.write_enable.next = true;
                    self.state.d.next = State::ReadBack;
                }
            }
            State::ReadBack => {
                self.spi_slave.continued_transaction.next = true;
                self.spi_slave.bits.next = 8.into();
                self.spi_slave.data_outbound.next = bit_cast::<64, 8>(self.readback_flop.q.val());
                self.spi_slave.start_send.next = true;
                self.state.d.next = State::WaitSlaveIdle;
            }
            State::DoNoop => {
                self.state.d.next = State::DoConvert;
                self.output_register.d.next =
                    bit_cast::<16, 3>(self.command_register.q.val().get_bits::<3>(2)) << 12
                        | bit_cast::<16, 12>(self.conversion_counter.q.val());
                self.conversion_counter.d.next = self.conversion_counter.q.val() + 1;
            }
            State::DoCommand => {
                self.command_register.d.next = self.spi_slave.data_inbound.val().get_bits::<8>(0);
                self.reg_ram.write_address.next = 0x3F.into();
                self.reg_ram.write_data.next = self.spi_slave.data_inbound.val().get_bits::<8>(0);
                self.reg_ram.write_enable.next = true;
                self.state.d.next = State::DoConvert;
                self.output_register.d.next =
                    bit_cast::<16, 3>(self.command_register.q.val().get_bits::<3>(2)) << 12
                        | bit_cast::<16, 12>(self.conversion_counter.q.val());
                self.conversion_counter.d.next = self.conversion_counter.q.val() + 1;
            }
            State::DoConvert => {
                self.spi_slave.data_outbound.next =
                    bit_cast::<64, 16>(self.output_register.q.val());
                self.spi_slave.continued_transaction.next = true;
                self.spi_slave.bits.next = 16.into();
                self.spi_slave.start_send.next = true;
                self.state.d.next = State::WaitSlaveIdle;
            }
            _ => {
                self.state.d.next = State::Init;
            }
        }
    }
}

fn basic_spi_config() -> SPIConfig {
    SPIConfig {
        clock_speed: 1_000_000,
        cs_off: true,
        mosi_off: false,
        speed_hz: 10_000,
        cpha: false,
        cpol: true,
    }
}

#[test]
fn test_ads8688_synthesizes() {
    let mut uut = ADS8688Simulator::new(basic_spi_config());
    uut.connect_all();
    yosys_validate("ads8688", &generate_verilog(&uut)).unwrap();
}

#[derive(LogicBlock)]
struct Test8688 {
    clock: Signal<In, Clock>,
    master: SPIMaster<64>,
    adc: ADS8688Simulator,
}

impl Logic for Test8688 {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, master, adc);
        SPIWiresMaster::join(&mut self.master.wires, &mut self.adc.wires);
    }
}

impl Default for Test8688 {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            master: SPIMaster::new(basic_spi_config()),
            adc: ADS8688Simulator::new(basic_spi_config()),
        }
    }
}

#[cfg(test)]
fn mk_test8688() -> Test8688 {
    let mut uut = Test8688::default();
    uut.master.bits_outbound.connect();
    uut.master.start_send.connect();
    uut.master.continued_transaction.connect();
    uut.master.data_outbound.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_yosys_validate_test_fixture() {
    let uut = mk_test8688();
    yosys_validate("ads8688_test_1", &generate_verilog(&uut)).unwrap();
}

#[cfg(test)]
fn do_spi_txn(
    bits: u16,
    value: u64,
    continued: bool,
    mut x: Box<Test8688>,
    sim: &mut Sim<Test8688>,
) -> Result<(Bits<64>, Box<Test8688>), SimError> {
    wait_clock_true!(sim, clock, x);
    x.master.data_outbound.next = value.to_bits();
    x.master.bits_outbound.next = bits.to_bits();
    x.master.continued_transaction.next = continued;
    x.master.start_send.next = true;
    wait_clock_cycle!(sim, clock, x);
    x.master.start_send.next = false;
    x = sim
        .watch(|x| x.master.transfer_done.val().into(), x)
        .unwrap();
    let ret = x.master.data_inbound.val();
    for _ in 0..50 {
        wait_clock_cycle!(sim, clock, x);
    }
    Ok((ret, x))
}

#[cfg(test)]
fn reg_read(
    reg_index: u32,
    x: Box<Test8688>,
    sim: &mut Sim<Test8688>,
) -> Result<(u8, Box<Test8688>), SimError> {
    // Shift the register index so that the lower 8 bits are free to hold the result
    let cmd = (reg_index << 17).into();
    let result = do_spi_txn(24, cmd, false, x, sim)?;
    let reg_val = (result.0 & 0xFF).get_bits::<8>(0).to_u8();
    Ok((reg_val, result.1))
}

#[cfg(test)]
fn reg_write(
    reg_index: u32,
    reg_value: u32,
    x: Box<Test8688>,
    sim: &mut Sim<Test8688>,
) -> Result<(u8, Box<Test8688>), SimError> {
    // We will read back the value...
    let cmd = ((reg_index << 17) | (1 << 16) | (reg_value << 8)).into();
    let ret = do_spi_txn(24, cmd, false, x, sim)?;
    Ok((ret.0.get_bits::<8>(0).to_u8(), ret.1))
}

#[cfg(test)]
fn cmd_write(
    cmd_value: u32,
    x: Box<Test8688>,
    sim: &mut Sim<Test8688>,
) -> Result<Box<Test8688>, SimError> {
    let cmd = (cmd_value << 8).into();
    let ret = do_spi_txn(16, cmd, false, x, sim)?;
    Ok(ret.1)
}

#[test]
fn test_reg_reads() {
    let uut = mk_test8688();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Test8688>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<Test8688>| {
        let mut x = sim.init()?;
        // Wait for reset to complete
        wait_clock_cycles!(sim, clock, x, 20);
        let expected = ram_init_vec()
            .into_iter()
            .map(|x| x as LiteralType)
            .collect::<Vec<_>>();
        let mut reg_val;
        for ndx in 0..0x3F {
            println!("Reading register index {}", ndx);
            (reg_val, x) = reg_read(ndx, x, &mut sim)?;
            println!("Value {} -> {:x}", ndx, reg_val);
            sim_assert_eq!(sim, u64::from(reg_val), expected[ndx as usize], x);
            wait_clock_true!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.run(Box::new(uut), 1_000_000).unwrap();
}

#[test]
fn test_reg_writes() {
    let uut = mk_test8688();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Test8688>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<Test8688>| {
        let mut x = sim.init()?;
        // Wait for reset to complete
        wait_clock_cycles!(sim, clock, x, 20);
        // Do the first read to initialize the chip
        let result = do_spi_txn(32, 0x00, false, x, &mut sim)?;
        x = result.1;
        let result = reg_write(5, 0xAF, x, &mut sim)?;
        x = result.1;
        println!("Write is {}", result.0);
        sim_assert_eq!(sim, result.0, 0xAF, x);
        let reg_val;
        // Now read it back using a read command
        (reg_val, x) = reg_read(5, x, &mut sim)?;
        sim_assert_eq!(sim, reg_val, 0xAF, x);
        sim.done(x)
    });
    sim.run(Box::new(uut), 1_000_000).unwrap();
    //sim.run_to_file(Box::new(uut), 1_000_000, &vcd_path!("ads8688_write.vcd")).unwrap()
}

#[test]
fn test_cmd_write() {
    let uut = mk_test8688();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Test8688>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<Test8688>| {
        let mut x = sim.init()?;
        // Wait for reset to complete
        wait_clock_cycles!(sim, clock, x, 20);
        // Do the first read to initialize the chip
        let result = do_spi_txn(32, 0x0, false, x, &mut sim)?;
        x = result.1;
        x = cmd_write(0xC4, x, &mut sim)?;
        // Now read it back using a read command
        let mut reg_val;
        (reg_val, x) = reg_read(0x3F, x, &mut sim)?;
        sim_assert_eq!(sim, reg_val, 0xC4, x);
        // Issue a NOOP
        x = cmd_write(0x00, x, &mut sim)?;
        // Now read it back using a read command
        (reg_val, x) = reg_read(0x3F, x, &mut sim)?;
        sim_assert_eq!(sim, reg_val, 0xC4, x);
        sim.done(x)
    });
    //    sim.run(Box::new(uut), 1_000_000).unwrap();
    sim.run_to_file(
        Box::new(uut),
        1_000_000,
        &vcd_path!("ads8688_cmd_write.vcd"),
    )
    .unwrap();
}

#[test]
fn test_conversion() {
    let uut = mk_test8688();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Test8688>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<Test8688>| {
        let mut x = sim.init()?;
        // Wait for reset to complete
        wait_clock_cycles!(sim, clock, x, 20);
        // Do the first read to initialize the chip
        let result = do_spi_txn(32, 0x0, false, x, &mut sim)?;
        x = result.1;
        x = cmd_write(0xC8, x, &mut sim)?;
        // Now read it back using a read command
        let reg_val;
        (reg_val, x) = reg_read(0x3F, x, &mut sim)?;
        sim_assert_eq!(sim, reg_val, 0xC8, x);
        let mut conversion;
        for ndx in 0..4 {
            (conversion, x) = do_spi_txn(24, 0x0, false, x, &mut sim)?;
            println!("Conversion value {:x}", conversion);
            sim_assert_eq!(sim, conversion, 0x2002 + ndx, x);
        }
        sim.done(x)
    });
    sim.run(Box::new(uut), 1_000_000).unwrap();
    //    sim.run_to_file(Box::new(uut), 1_000_000, &vcd_path!("ads8688_conversion.vcd")).unwrap();
}

#[test]
fn test_pipelined_conversion() {
    let uut = mk_test8688();
    let mut sim = simple_sim!(Test8688, clock, 200_000_000_000, sim, {
        let mut x = sim.init()?;
        // Wait for reset to complete
        wait_clock_cycles!(sim, clock, x, 20);
        // Do the first read to initialize the chip
        let result = do_spi_txn(32, 0x0, false, x, &mut sim)?;
        x = result.1;
        // Issue the first command to read from channel 0 - ignore the result
        x = cmd_write(0xC0, x, &mut sim)?;
        // Loop over the channels.  For each one, initiate a command to select
        // the next channel, and capture the current value as the previous
        // channel conversion.
        let mut conversion;
        for ndx in 1..8 {
            let cmd = (0xC0 + (ndx << 2)) << 16;
            (conversion, x) = do_spi_txn(24, cmd, false, x, &mut sim)?;
            println!("Conversion value [{}] -> {:x}", ndx, conversion);
            sim_assert_eq!(sim, conversion & 0xFFFF, ((ndx - 1) << 12) + ndx + 1, x);
        }
        // To get the last channel, we send a noop
        (conversion, x) = do_spi_txn(24, 0, false, x, &mut sim)?;
        println!("Conversion tail -> {:x}", conversion);
        sim_assert_eq!(sim, conversion & 0xFFFF, 0x7009, x);
        sim.done(x)
    });
    //    sim.run(Box::new(uut), 10_000_000).unwrap();
    sim.run_to_file(Box::new(uut), 1_000_000, &vcd_path!("ads8688_pipeline.vcd"))
        .unwrap();
}
