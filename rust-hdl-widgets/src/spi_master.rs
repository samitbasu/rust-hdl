use crate::dff::DFF;
use crate::prelude::{BitSynchronizer, Strobe};
use rust_hdl_core::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum SPIState {
    Idle,
    Dwell,
    LoadBit,
    MActive,
    SampleMISO,
    MIdle,
}

#[derive(Copy, Clone)]
pub struct SPIConfig {
    pub clock_speed: u64,
    pub cs_off: bool,
    pub mosi_off: bool,
    pub speed_hz: u64,
    pub cpha: bool,
    pub cpol: bool,
}

#[derive(LogicInterface, Default)]
pub struct SPIWires {
    pub mosi: Signal<Out, Bit>,
    pub miso: Signal<In, Bit>,
    pub msel: Signal<Out, Bit>,
    pub mclk: Signal<Out, Bit>,
}

#[derive(LogicBlock)]
pub struct SPIMaster<const N: usize> {
    pub clock: Signal<In, Clock>,
    pub bits_outbound: Signal<In, Bits<16>>,
    pub data_outbound: Signal<In, Bits<N>>,
    pub data_inbound: Signal<Out, Bits<N>>,
    pub start_send: Signal<In, Bit>,
    pub transfer_done: Signal<Out, Bit>,
    pub continued_transaction: Signal<In, Bit>,
    pub wires: SPIWires,
    register_out: DFF<Bits<N>>,
    register_in: DFF<Bits<N>>,
    state: DFF<SPIState>,
    strobe: Strobe<32>,
    pointer: DFF<Bits<16>>,
    pointerm1: Signal<Local, Bits<16>>,
    clock_state: DFF<Bit>,
    done_flop: DFF<Bit>,
    msel_flop: DFF<Bit>,
    mosi_flop: DFF<Bit>,
    miso_synchronizer: BitSynchronizer,
    continued_save: DFF<Bit>,
    cs_off: Constant<Bit>,
    mosi_off: Constant<Bit>,
    cpha: Constant<Bit>,
    cpol: Constant<Bit>,
}

impl<const N: usize> SPIMaster<N> {
    pub fn new(config: SPIConfig) -> Self {
        assert!(8 * config.speed_hz <= config.clock_speed);
        Self {
            clock: Default::default(),
            bits_outbound: Default::default(),
            data_outbound: Default::default(),
            data_inbound: Default::default(),
            start_send: Default::default(),
            transfer_done: Default::default(),
            continued_transaction: Default::default(),
            wires: Default::default(),
            register_out: Default::default(),
            register_in: Default::default(),
            state: Default::default(),
            strobe: Strobe::new(config.clock_speed, 4.0 * config.speed_hz as f64),
            pointer: Default::default(),
            pointerm1: Default::default(),
            clock_state: Default::default(),
            done_flop: Default::default(),
            msel_flop: DFF::new(config.cs_off),
            mosi_flop: Default::default(),
            miso_synchronizer: Default::default(),
            continued_save: Default::default(),
            cs_off: Constant::new(config.cs_off),
            mosi_off: Constant::new(config.mosi_off),
            cpha: Constant::new(config.cpha),
            cpol: Constant::new(config.cpol),
        }
    }
}

impl<const N: usize> Logic for SPIMaster<N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Wire up the clocks.
        self.register_out.clk.next = self.clock.val();
        self.register_in.clk.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.strobe.clock.next = self.clock.val();
        self.pointer.clk.next = self.clock.val();
        self.clock_state.clk.next = self.clock.val();
        self.done_flop.clk.next = self.clock.val();
        self.msel_flop.clk.next = self.clock.val();
        self.mosi_flop.clk.next = self.clock.val();
        self.miso_synchronizer.clock.next = self.clock.val();
        self.continued_save.clk.next = self.clock.val();
        // Activate the baud strobe
        self.strobe.enable.next = true;
        // Connect the MISO synchronizer to the input line
        self.miso_synchronizer.sig_in.next = self.wires.miso.val();
        // Connect the rest of the SPI lines to the flops
        self.wires.mclk.next = self.clock_state.q.val();
        self.wires.mosi.next = self.mosi_flop.q.val();
        self.wires.msel.next = self.msel_flop.q.val();
        // Connect the output signals to the internal registers
        self.data_inbound.next = self.register_in.q.val();
        self.transfer_done.next = self.done_flop.q.val();
        // Latch prevention
        self.done_flop.d.next = false;
        self.register_out.d.next = self.register_out.q.val();
        self.state.d.next = self.state.q.val();
        self.pointer.d.next = self.pointer.q.val();
        self.register_in.d.next = self.register_in.q.val();
        self.msel_flop.d.next = self.msel_flop.q.val();
        self.continued_save.d.next = self.continued_save.q.val();
        self.mosi_flop.d.next = self.mosi_flop.q.val();
        self.clock_state.d.next = self.clock_state.q.val();
        self.pointerm1.next = self.pointer.q.val() - 1_u32;
        // The main state machine
        match self.state.q.val() {
            SPIState::Idle => {
                self.clock_state.d.next = self.cpol.val();
                if self.start_send.val() {
                    // Capture the outgoing data in our register
                    self.register_out.d.next = self.data_outbound.val();
                    self.state.d.next = SPIState::Dwell; // Transition to the DWELL state
                    self.pointer.d.next = self.bits_outbound.val(); // set bit pointer to number of bit to send (1 based)
                    self.register_in.d.next = 0_usize.into(); // Clear out the input store register
                    self.msel_flop.d.next = !self.cs_off.val(); // Activate the chip select
                    self.continued_save.d.next = self.continued_transaction.val();
                } else {
                    if !self.continued_save.q.val() {
                        self.msel_flop.d.next = self.cs_off.val(); // Set the chip select signal to be "off"
                    }
                }
                self.mosi_flop.d.next = self.mosi_off.val(); // Set the mosi signal to be "off"
            }
            SPIState::Dwell => {
                if self.strobe.strobe.val() {
                    // Dwell timeout has reached zero
                    self.state.d.next = SPIState::LoadBit; // Transition to the loadbit state
                }
            }
            SPIState::LoadBit => {
                if self.pointer.q.val().any() {
                    // We have data to send
                    self.mosi_flop.d.next = self
                        .register_out
                        .q
                        .val()
                        .get_bit(self.pointerm1.val().into()); // Fetch the corresponding bit out of the register
                    self.pointer.d.next = self.pointerm1.val().into(); // Decrement the pointer
                    self.state.d.next = SPIState::MActive; // Move to the hold mclock low state
                    self.clock_state.d.next = self.cpol.val() ^ self.cpha.val();
                } else {
                    self.mosi_flop.d.next = self.mosi_off.val(); // Set the mosi signal to be "off"
                    self.state.d.next = SPIState::Idle; // No data, go back to idle
                    self.done_flop.d.next = true; // signal the transfer is complete
                }
            }
            SPIState::MActive => {
                if self.strobe.strobe.val() {
                    self.state.d.next = SPIState::SampleMISO;
                }
            }
            SPIState::SampleMISO => {
                self.register_in.d.next = self.register_in.q.val().replace_bit(
                    self.pointer.q.val().into(),
                    self.miso_synchronizer.sig_out.val(),
                );
                self.clock_state.d.next = !self.clock_state.q.val();
                self.state.d.next = SPIState::MIdle;
            }
            SPIState::MIdle => {
                if self.strobe.strobe.val() {
                    self.state.d.next = SPIState::LoadBit;
                }
            }
        }
    }
}

#[test]
fn test_spi_master_is_synthesizable() {
    #[derive(LogicBlock)]
    struct Wrap {
        uut: SPIMaster<64>,
    }

    impl Logic for Wrap {
        fn update(&mut self) {}
    }

    impl Default for Wrap {
        fn default() -> Self {
            let config = SPIConfig {
                clock_speed: 48_000_000,
                cs_off: true,
                mosi_off: false,
                speed_hz: 1_000_000,
                cpha: true,
                cpol: false,
            };
            Self {
                uut: SPIMaster::new(config),
            }
        }
    }

    let mut dev = Wrap::default();
    dev.uut.clock.connect();
    dev.uut.bits_outbound.connect();
    dev.uut.data_outbound.connect();
    dev.uut.start_send.connect();
    dev.uut.continued_transaction.connect();
    dev.uut.wires.miso.connect();
    dev.connect_all();
    println!("{}", generate_verilog(&dev));
    rust_hdl_synth::yosys_validate("spi_master", &generate_verilog(&dev)).unwrap();
}
