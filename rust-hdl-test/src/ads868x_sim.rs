use rust_hdl_core::prelude::*;
use rust_hdl_core::simulate::SimError;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::prelude::*;
use rust_hdl_widgets::spi_master::{SPIConfig, SPIMaster};
use rust_hdl_widgets::spi_slave::SPISlave;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum ADS868XState {
    Ready,
    Waiting,
    Dispatch,
    ReadWordCmd,
    ReadByteCmd,
    WriteWordCmd,
    WriteMSBCmd,
    WriteLSBCmd,
    WriteDone,
    Nop,
}

#[derive(LogicBlock)]
pub struct ADS868XSimulator {
    pub mosi: Signal<In, Bit>,
    pub mclk: Signal<In, Bit>,
    pub msel: Signal<In, Bit>,
    pub miso: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    // RAM to store register values
    reg_ram: RAM<Bits<16>, 5>,
    // SPI slave device
    spi_slave: SPISlave<32>,
    // FSM State
    state: DFF<ADS868XState>,
    // Rolling counter to emulate conversions
    conversion_counter: DFF<Bits<16>>,
    // Inbound register
    inbound: DFF<Bits<32>>,
    // Local signal to store the command bits
    read_cmd: Signal<Local, Bits<5>>,
    write_cmd: Signal<Local, Bits<7>>,
    address: Signal<Local, Bits<9>>,
    data_parity: Signal<Local, Bit>,
    id_parity: Signal<Local, Bit>,
}

impl ADS868XSimulator {
    pub fn spi_hw() -> SPIConfig {
        SPIConfig {
            clock_speed: 48_000_000,
            cs_off: true,
            mosi_off: true,
            speed_hz: 400_000,
            cpha: false,
            cpol: false,
        }
    }
    pub fn spi_sw() -> SPIConfig {
        SPIConfig {
            clock_speed: 1_000_000,
            cs_off: true,
            mosi_off: true,
            speed_hz: 10_000,
            cpha: false,
            cpol: false,
        }
    }

    pub fn new(spi_config: SPIConfig) -> Self {
        assert!(spi_config.clock_speed > 10 * spi_config.speed_hz);
        Self {
            mosi: Default::default(),
            mclk: Default::default(),
            msel: Default::default(),
            miso: Default::default(),
            clock: Default::default(),
            reg_ram: Default::default(),
            spi_slave: SPISlave::new(spi_config),
            state: DFF::new(ADS868XState::Ready),
            conversion_counter: Default::default(),
            inbound: Default::default(),
            read_cmd: Default::default(),
            write_cmd: Default::default(),
            address: Default::default(),
            data_parity: Default::default(),
            id_parity: Default::default(),
        }
    }
}

#[test]
fn test_indexing() {
    let val: Bits<32> = 0b11000_00_101_001_100_00000000_00000000_u32.into();
    assert_eq!(val.get_bits::<5>(27).index(), 0b11000_usize);
    assert_eq!(val.get_bits::<9>(16).index(), 0b101_001_100_usize);
}

impl Logic for ADS868XSimulator {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the spi bus
        self.spi_slave.mosi.next = self.mosi.val();
        self.spi_slave.mclk.next = self.mclk.val();
        self.spi_slave.msel.next = self.msel.val();
        self.miso.next = self.spi_slave.miso.val();
        // Clock internal components
        self.reg_ram.read_clock.next = self.clock.val();
        self.reg_ram.write_clock.next = self.clock.val();
        self.spi_slave.clock.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.conversion_counter.clk.next = self.clock.val();
        self.inbound.clk.next = self.clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.conversion_counter.d.next = self.conversion_counter.q.val();
        // Set default values
        self.spi_slave.start_send.next = false;
        self.spi_slave.continued_transaction.next = false;
        self.spi_slave.bits.next = 0_u16.into();
        self.spi_slave.data_outbound.next = 0_u64.into();
        self.reg_ram.write_enable.next = false;
        self.reg_ram.write_data.next = 0_usize.into();
        self.spi_slave.disabled.next = false;
        self.read_cmd.next = self.inbound.q.val().get_bits::<5>(27_usize);
        self.write_cmd.next = self.inbound.q.val().get_bits::<7>(25_usize);
        self.address.next = self.inbound.q.val().get_bits::<9>(16_usize);
        self.reg_ram.write_address.next = bit_cast::<5, 9>(self.address.val() >> 1_usize);
        self.reg_ram.read_address.next = 0_u32.into();
        self.inbound.d.next = self.inbound.q.val();
        self.data_parity.next = self.conversion_counter.q.val().xor();
        self.id_parity.next = (self.reg_ram.read_data.val() & 0x0FF_u32).xor();
        match self.state.q.val() {
            ADS868XState::Ready => {
                self.state.d.next = ADS868XState::Nop;
            }
            ADS868XState::Waiting => {
                if self.spi_slave.transfer_done.val() {
                    self.inbound.d.next = self.spi_slave.data_inbound.val();
                    self.state.d.next = ADS868XState::Dispatch;
                }
            }
            ADS868XState::Dispatch => {
                if self.read_cmd.val() == 0b11001_u32 {
                    self.state.d.next = ADS868XState::ReadWordCmd;
                    self.reg_ram.read_address.next =
                        bit_cast::<5, 9>(self.address.val() >> 1_usize);
                } else if self.read_cmd.val() == 0b01001_u32 {
                    self.state.d.next = ADS868XState::ReadByteCmd;
                    self.reg_ram.read_address.next =
                        bit_cast::<5, 9>(self.address.val() >> 1_usize);
                } else if self.write_cmd.val() == 0b11010_00_u32 {
                    self.state.d.next = ADS868XState::WriteWordCmd;
                } else if self.write_cmd.val() == 0b11010_01_u32 {
                    self.state.d.next = ADS868XState::WriteMSBCmd;
                    self.reg_ram.read_address.next =
                        bit_cast::<5, 9>(self.address.val() >> 1_usize);
                } else if self.write_cmd.val() == 0b11010_10_u32 {
                    self.state.d.next = ADS868XState::WriteLSBCmd;
                    self.reg_ram.read_address.next =
                        bit_cast::<5, 9>(self.address.val() >> 1_usize);
                } else {
                    self.reg_ram.read_address.next = 0x02_u32.into();
                    self.state.d.next = ADS868XState::Nop;
                }
            }
            ADS868XState::ReadWordCmd => {
                self.spi_slave.data_outbound.next =
                    bit_cast::<32, 16>(self.reg_ram.read_data.val());
                self.spi_slave.bits.next = 16_usize.into();
                self.spi_slave.start_send.next = true;
                self.state.d.next = ADS868XState::Waiting;
            }
            ADS868XState::ReadByteCmd => {
                if self.address.val().get_bit(0_usize) {
                    self.spi_slave.data_outbound.next =
                        bit_cast::<32, 16>(self.reg_ram.read_data.val() >> 8_u32);
                } else {
                    self.spi_slave.data_outbound.next =
                        bit_cast::<32, 16>(self.reg_ram.read_data.val() & 0xFF_u32);
                }
                self.spi_slave.bits.next = 8_usize.into();
                self.spi_slave.start_send.next = true;
                self.state.d.next = ADS868XState::Waiting;
            }
            ADS868XState::WriteWordCmd => {
                self.reg_ram.write_data.next =
                    bit_cast::<16, 32>(self.inbound.q.val() & 0xFFFF_u32);
                self.reg_ram.write_enable.next = true;
                self.state.d.next = ADS868XState::WriteDone;
            }
            ADS868XState::WriteLSBCmd => {
                self.reg_ram.write_data.next =
                    bit_cast::<16, 32>(self.inbound.q.val() & 0x00FF_u32)
                        | (self.reg_ram.read_data.val() & 0xFF00_u32);
                self.reg_ram.write_enable.next = true;
                self.state.d.next = ADS868XState::WriteDone;
            }
            ADS868XState::WriteMSBCmd => {
                self.reg_ram.write_data.next =
                    bit_cast::<16, 32>(self.inbound.q.val() & 0xFF00_u32)
                        | (self.reg_ram.read_data.val() & 0x00FF_u32);
                self.reg_ram.write_enable.next = true;
                self.state.d.next = ADS868XState::WriteDone;
            }
            ADS868XState::WriteDone => {
                self.spi_slave.bits.next = 32_usize.into();
                self.spi_slave.data_outbound.next = self.inbound.q.val();
                self.spi_slave.start_send.next = true;
                self.state.d.next = ADS868XState::Waiting;
            }
            ADS868XState::Nop => {
                self.spi_slave.bits.next = 32_usize.into();
                self.spi_slave.data_outbound.next =
                    (bit_cast::<32, 16>(self.conversion_counter.q.val()) << 16_u32)
                        | bit_cast::<32, 16>(self.reg_ram.read_data.val() & 0x0FF_u32) << 12_u32
                        | bit_cast::<32, 1>(self.data_parity.val().into()) << 11_u32
                        | bit_cast::<32, 1>((self.data_parity.val() ^ self.id_parity.val()).into())
                            << 10_u32;
                self.spi_slave.start_send.next = true;
                self.state.d.next = ADS868XState::Waiting;
                self.conversion_counter.d.next = self.conversion_counter.q.val() + 1_u32;
            }
        }
    }
}

#[test]
fn test_ads8689_synthesizes() {
    let mut uut = ADS868XSimulator::new(ADS868XSimulator::spi_sw());
    uut.mosi.connect();
    uut.mclk.connect();
    uut.msel.connect();
    uut.clock.connect();
    uut.connect_all();
    yosys_validate("ads8689", &generate_verilog(&uut)).unwrap();
}

#[derive(LogicBlock)]
struct Test8689 {
    clock: Signal<In, Clock>,
    master: SPIMaster<32>,
    adc: ADS868XSimulator,
}

impl Logic for Test8689 {
    #[hdl_gen]
    fn update(&mut self) {
        self.master.clock.next = self.clock.val();
        self.adc.clock.next = self.clock.val();
        self.adc.mosi.next = self.master.wires.mosi.val();
        self.adc.msel.next = self.master.wires.msel.val();
        self.adc.mclk.next = self.master.wires.mclk.val();
        self.master.wires.miso.next = self.adc.miso.val();
    }
}

impl Default for Test8689 {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            master: SPIMaster::new(ADS868XSimulator::spi_sw()),
            adc: ADS868XSimulator::new(ADS868XSimulator::spi_sw()),
        }
    }
}

fn do_spi_txn(
    bits: u16,
    value: u64,
    continued: bool,
    mut x: Box<Test8689>,
    sim: &mut Sim<Test8689>,
) -> Result<(Bits<32>, Box<Test8689>), SimError> {
    wait_clock_true!(sim, clock, x);
    x.master.data_outbound.next = value.into();
    x.master.bits_outbound.next = bits.into();
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
fn mk_test8689() -> Test8689 {
    let mut uut = Test8689::default();
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
    let uut = mk_test8689();
    yosys_validate("8689_1", &generate_verilog(&uut)).unwrap();
}

#[test]
fn test_reg_writes() {
    let uut = mk_test8689();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Test8689>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<Test8689>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        wait_clock_cycle!(sim, clock, x);
        // Write an ID to register 2...
        let result = do_spi_txn(32, 0xd0_02_00_02, false, x, &mut sim)?;
        x = result.1;
        wait_clock_cycle!(sim, clock, x);
        wait_clock_cycle!(sim, clock, x);
        let result = do_spi_txn(32, 0x48_02_00_00, false, x, &mut sim)?;
        x = result.1;
        let result = do_spi_txn(8, 0x00, false, x, &mut sim)?;
        println!("ID Register read {:x}", result.0);
        x = result.1;
        sim_assert!(sim, result.0.index() == 2, x);
        /*
        # Output should be 0x40 0x08
        [ 0xd0 0x10 0x40 0x08 ] % [ 0xc8 0x10 0x00 0x00 ] % { 0x00 0x00 ]
         */
        wait_clock_cycle!(sim, clock, x);
        let result = do_spi_txn(32, 0xd0_10_40_08, false, x, &mut sim)?;
        x = result.1;
        wait_clock_cycle!(sim, clock, x);
        let result = do_spi_txn(32, 0xc8_10_00_00, false, x, &mut sim)?;
        x = result.1;
        wait_clock_cycle!(sim, clock, x);
        let result = do_spi_txn(16, 0x00, false, x, &mut sim)?;
        x = result.1;
        sim_assert!(sim, result.0.index() == 0x40_08, x);
        for i in 0..5 {
            wait_clock_cycle!(sim, clock, x);
            let result = do_spi_txn(32, 0x00_00_00_00, false, x, &mut sim)?;
            x = result.1;
            println!("Reading is {:x}", result.0);
            sim_assert!(
                sim,
                (result.0 & 0xFFFF0000_usize) == ((i + 2) << 16) as u32,
                x
            );
            let parity_bit = result.0 & 0x800_usize != 0_usize;
            let data: Bits<32> = (result.0 & 0xFFFF0000_usize) >> 16_usize;
            sim_assert!(sim, data.xor() == parity_bit, x);
        }
        sim.done(x)
    });
    sim.run(Box::new(uut), 1_000_000).unwrap();
}

#[test]
fn test_parity_calculations() {
    for sample in [
        0x00020C00_u32,
        0x92ab1400_u32,
        0x734b1800,
        0x4fc81400,
        0x7bee1400,
        0x94821800,
        0x5eb31400,
        0x4eaa1400,
        0x8ac91800,
        0x95321800,
        0x54c01800,
        0x561a1800,
        0x91601800,
        0x7e401800,
        0x50961400,
    ] {
        let mut data = (sample & 0xFFFF_0000) >> 16;
        let mut parity = false;
        for _ in 0..16 {
            parity = parity ^ (data & 0x1 != 0);
            data = data >> 1;
        }
        let adc_flag = (sample & 0x800) != 0;
        assert_eq!(adc_flag, parity);
    }
}
