use crate::dff::DFF;
use crate::edge_detector::EdgeDetector;
use crate::prelude::BitSynchronizer;
use crate::spi_master::SPIConfig;
use rust_hdl_core::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum SPISlaveState {
    Idle,
    Armed,
    Capture,
    Hold,
    Update,
    Settle,
    Waiting,
    Hangup,
    Disabled,
}

#[derive(LogicBlock)]
pub struct SPISlave<const N: usize> {
    pub clock: Signal<In, Clock>,
    pub mosi: Signal<In, Bit>,
    pub miso: Signal<Out, Bit>,
    pub msel: Signal<In, Bit>,
    pub mclk: Signal<In, Bit>,
    pub disabled: Signal<In, Bit>,
    pub busy: Signal<Out, Bit>,
    pub data_inbound: Signal<Out, Bits<N>>,
    pub start_send: Signal<In, Bit>,
    pub data_outbound: Signal<In, Bits<N>>,
    pub bits: Signal<In, Bits<16>>,
    pub continued_transaction: Signal<In, Bit>,
    pub transfer_done: Signal<Out, Bit>,
    miso_flop: DFF<Bit>,
    done_flop: DFF<Bit>,
    register_out: DFF<Bits<N>>,
    register_in: DFF<Bits<N>>,
    state: DFF<SPISlaveState>,
    pointer: DFF<Bits<16>>,
    bits_saved: DFF<Bits<16>>,
    continued_saved: DFF<Bit>,
    capture_detector: EdgeDetector,
    advance_detector: EdgeDetector,
    edge_detector: EdgeDetector,
    mclk_synchronizer: BitSynchronizer,
    csel_synchronizer: BitSynchronizer,
    escape: DFF<Bits<16>>,
    clocks_per_baud: Constant<Bits<16>>,
    cpha: Constant<Bit>,
    cs_off: Constant<Bit>,
}

///
/// Here is a table of the SPI setup:
/// CPOL  CPHA  EDGE  ACTION
///  0     0     R     Sample
///  0     0     F     Change
///  0     1     R     Change
///  0     1     F     Sample
///  1     0     R     Change
///  1     0     F     Sample
///  1     1     R     Sample
///  1     1     F     Change
///
/// So Sample on Rising edge if CPOL == CPHA
/// Also, CPHA decides if we start in the sample state or in the change state
impl<const N: usize> SPISlave<N> {
    pub fn new(config: SPIConfig) -> Self {
        Self {
            clock: Default::default(),
            mosi: Default::default(),
            miso: Default::default(),
            msel: Default::default(),
            mclk: Default::default(),
            disabled: Default::default(),
            busy: Default::default(),
            data_inbound: Default::default(),
            start_send: Default::default(),
            data_outbound: Default::default(),
            bits: Default::default(),
            continued_transaction: Default::default(),
            transfer_done: Default::default(),
            miso_flop: Default::default(),
            done_flop: Default::default(),
            register_out: Default::default(),
            register_in: Default::default(),
            state: Default::default(),
            pointer: Default::default(),
            bits_saved: Default::default(),
            continued_saved: Default::default(),
            capture_detector: EdgeDetector::new(!(config.cpol ^ config.cpha)),
            advance_detector: EdgeDetector::new(config.cpol ^ config.cpha),
            edge_detector: EdgeDetector::new(!config.cs_off),
            mclk_synchronizer: BitSynchronizer::default(),
            csel_synchronizer: BitSynchronizer::default(),
            escape: Default::default(),
            clocks_per_baud: Constant::new((2 * config.clock_speed / config.speed_hz).into()),
            cpha: Constant::new(config.cpha),
            cs_off: Constant::new(config.cs_off),
        }
    }
}

impl<const N: usize> Logic for SPISlave<N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clocks
        self.miso_flop.clk.next = self.clock.val();
        self.done_flop.clk.next = self.clock.val();
        self.register_out.clk.next = self.clock.val();
        self.register_in.clk.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.pointer.clk.next = self.clock.val();
        self.bits_saved.clk.next = self.clock.val();
        self.continued_saved.clk.next = self.clock.val();
        self.capture_detector.clock.next = self.clock.val();
        self.advance_detector.clock.next = self.clock.val();
        self.edge_detector.clock.next = self.clock.val();
        self.mclk_synchronizer.clock.next = self.clock.val();
        self.csel_synchronizer.clock.next = self.clock.val();
        self.escape.clk.next = self.clock.val();
        // Connect the detectors
        self.capture_detector.input_signal.next = self.mclk_synchronizer.sig_out.val();
        self.advance_detector.input_signal.next = self.mclk_synchronizer.sig_out.val();
        self.edge_detector.input_signal.next = self.csel_synchronizer.sig_out.val();
        // Connect the synchronizers
        self.mclk_synchronizer.sig_in.next = self.mclk.val();
        self.csel_synchronizer.sig_in.next = self.msel.val();
        // Logic
        self.busy.next = (self.state.q.val() != SPISlaveState::Idle)
            | (self.csel_synchronizer.sig_out.val() != self.cs_off.val());
        if self.state.q.val() != SPISlaveState::Disabled {
            self.miso.next = self.miso_flop.q.val();
        } else {
            self.miso.next = true;
        }
        self.data_inbound.next = self.register_in.q.val();
        self.transfer_done.next = self.done_flop.q.val();
        self.done_flop.d.next = false;
        self.miso_flop.d.next = self
            .register_out
            .q
            .val()
            .get_bit(self.pointer.q.val().into());
        // Latch prevention
        self.register_in.d.next = self.register_in.q.val();
        self.state.d.next = self.state.q.val();
        self.pointer.d.next = self.pointer.q.val();
        self.register_out.d.next = self.register_out.q.val();
        self.bits_saved.d.next = self.bits_saved.q.val();
        self.continued_saved.d.next = self.continued_saved.q.val();
        self.escape.d.next = self.escape.q.val();
        match self.state.q.val() {
            SPISlaveState::Idle => {
                if self.edge_detector.edge_signal.val() {
                    self.register_in.d.next = 0_u32.into();
                    self.state.d.next = SPISlaveState::Waiting;
                    self.pointer.d.next = 0_u16.into();
                } else if self.start_send.val() {
                    self.register_out.d.next = self.data_outbound.val();
                    self.bits_saved.d.next = self.bits.val();
                    self.continued_saved.d.next = self.continued_transaction.val();
                    self.pointer.d.next = self.bits.val() - 1_usize;
                    self.register_in.d.next = 0_u32.into();
                    self.state.d.next = SPISlaveState::Armed;
                } else if self.disabled.val() {
                    self.state.d.next = SPISlaveState::Disabled;
                }
            }
            SPISlaveState::Armed => {
                if self.csel_synchronizer.sig_out.val() != self.cs_off.val() {
                    if self.cpha.val() & !self.continued_saved.q.val() {
                        self.state.d.next = SPISlaveState::Waiting;
                    } else {
                        self.state.d.next = SPISlaveState::Settle;
                    }
                }
            }
            SPISlaveState::Waiting => {
                if self.advance_detector.edge_signal.val() {
                    self.state.d.next = SPISlaveState::Settle;
                }
            }
            SPISlaveState::Settle => {
                if self.capture_detector.edge_signal.val() {
                    self.state.d.next = SPISlaveState::Capture;
                }
            }
            SPISlaveState::Capture => {
                self.register_in.d.next = (self.register_in.q.val() << 1_usize)
                    | bit_cast::<N, 1>(self.mosi.val().into());
                self.state.d.next = SPISlaveState::Hold;
            }
            SPISlaveState::Hold => {
                if self.advance_detector.edge_signal.val() {
                    if self.pointer.q.val().any() {
                        self.state.d.next = SPISlaveState::Update;
                    } else {
                        if self.continued_saved.q.val() {
                            self.done_flop.d.next = true;
                            self.state.d.next = SPISlaveState::Idle;
                        } else {
                            self.state.d.next = SPISlaveState::Hangup;
                        }
                    }
                    self.escape.d.next = 0_u16.into();
                } else if self.csel_synchronizer.sig_out.val() == self.cs_off.val() {
                    self.done_flop.d.next = true;
                    self.state.d.next = SPISlaveState::Idle;
                } else {
                    self.escape.d.next = self.escape.q.val() + 1_usize;
                }
                if self.escape.q.val() == self.clocks_per_baud.val() {
                    self.done_flop.d.next = true;
                    self.state.d.next = SPISlaveState::Idle;
                }
            }
            SPISlaveState::Update => {
                if self.pointer.q.val().any() {
                    self.pointer.d.next = self.pointer.q.val() - 1_usize;
                }
                self.state.d.next = SPISlaveState::Settle;
            }
            SPISlaveState::Hangup => {
                if self.csel_synchronizer.sig_out.val() == self.cs_off.val() {
                    self.done_flop.d.next = true;
                    self.state.d.next = SPISlaveState::Idle;
                }
                if self.disabled.val() {
                    self.state.d.next = SPISlaveState::Disabled;
                }
            }
            SPISlaveState::Disabled => {
                if !self.disabled.val() {
                    self.state.d.next = SPISlaveState::Idle;
                    self.register_out.d.next = 0_u32.into();
                }
            }
        }
    }
}

#[test]
fn test_spi_slave_synthesizes() {
    let config = SPIConfig {
        clock_speed: 48_000_000,
        cs_off: true,
        mosi_off: false,
        speed_hz: 1_000_000,
        cpha: true,
        cpol: false,
    };
    let mut uut: SPISlave<64> = SPISlave::new(config);
    uut.clock.connect();
    uut.bits.connect();
    uut.mosi.connect();
    uut.msel.connect();
    uut.mclk.connect();
    uut.disabled.connect();
    uut.start_send.connect();
    uut.data_outbound.connect();
    uut.bits.connect();
    uut.continued_transaction.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("spi_slave", &generate_verilog(&uut)).unwrap();
    println!("{}", generate_verilog(&uut));
}
