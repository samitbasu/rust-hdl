use rust_hdl::prelude::Bits;
use serde::Serialize;

use crate::{
    strobe::{StrobeConfig, StrobeState},
    synchronous::Synchronous,
};

#[derive(Copy, Clone, PartialEq, Debug, Default, Serialize)]
pub enum SPIState {
    #[default]
    Idle,
    Dwell,
    LoadBit,
    MActive,
    SampleMISO,
    MIdle,
    Finish,
}

pub struct SPIMode {
    pub cs_off: bool,
    pub mosi_off: bool,
    pub cpha: bool,
    pub cpol: bool,
}

pub struct SPIControllerConfig<const N: usize> {
    pub clock_speed: u64,
    pub cs_off: bool,
    pub mosi_off: bool,
    pub speed_hz: u64,
    pub cpha: bool,
    pub cpol: bool,
    pub strobe: StrobeConfig,
}

impl<const N: usize> SPIControllerConfig<N> {
    pub fn new(clock_speed: u64, speed_hz: u64, mode: SPIMode) -> Self {
        let strobe = StrobeConfig::new(clock_speed, speed_hz as f64);
        Self {
            clock_speed,
            speed_hz,
            cs_off: mode.cs_off,
            mosi_off: mode.mosi_off,
            cpha: mode.cpha,
            cpol: mode.cpol,
            strobe,
        }
    }
}

#[derive(Default, Clone, Copy, Serialize)]
pub struct SPIControllerState<const N: usize> {
    pub register_out: Bits<N>,
    pub register_in: Bits<N>,
    pub state: SPIState,
    pub strobe: StrobeState,
    pub pointer: u16,
    pub clock_state: bool,
    pub done_flop: bool,
    pub msel_flop: bool,
    pub mosi_flop: bool,
    pub continued_save: bool,
}

#[derive(Copy, Clone, Serialize)]
pub struct SPIInputs<const N: usize> {
    pub bits_outbound: u16,
    pub data_outbound: Bits<N>,
    pub start_send: bool,
    pub continued_transaction: bool,
    pub miso: bool,
}

#[derive(Copy, Clone, Serialize)]
pub struct SPIOutputs<const N: usize> {
    pub mosi: bool,
    pub msel: bool,
    pub mclk: bool,
    pub data_inbound: Bits<N>,
    pub transfer_done: bool,
    pub busy: bool,
}

impl<const N: usize> Synchronous for SPIControllerConfig<N> {
    type Input = SPIInputs<N>;
    type Output = SPIOutputs<N>;
    type State = SPIControllerState<N>;

    fn default_output(&self) -> Self::Output {
        SPIOutputs {
            mosi: self.mosi_off,
            msel: self.cs_off,
            mclk: self.cpol,
            data_inbound: Bits::<N>::default(),
            transfer_done: false,
            busy: false,
        }
    }

    fn update(&self, q: Self::State, i: Self::Input) -> (Self::Output, Self::State) {
        let mut d = q;
        let mut o = self.default_output();
        let strobe_output;
        // Activate the baud strobe
        (strobe_output, d.strobe) = self.strobe.update(q.strobe, true);
        o.mclk = q.clock_state;
        o.mosi = q.mosi_flop;
        o.msel = q.msel_flop;
        // Connect the output signals to the internal registers
        o.data_inbound = q.register_in;
        o.transfer_done = q.done_flop;
        d.done_flop = false;
        let pointerm1 = q.pointer - 1;
        o.busy = true;
        // The main state machine
        match q.state {
            SPIState::Idle => {
                o.busy = false;
                d.clock_state = self.cpol;
                if i.start_send {
                    // Capture the outgoing data in our register
                    d.register_out = i.data_outbound;
                    d.state = SPIState::Dwell; // Transition to the DWELL state
                    d.pointer = i.bits_outbound; // set bit point to number of bit to send (1 based)
                    d.register_in = 0.into(); // Clear out the input store register
                    d.msel_flop = !self.cs_off; // Activate the chip select
                    d.continued_save = i.continued_transaction;
                } else if !q.continued_save {
                    d.msel_flop = self.cs_off; // Set the chip select signal to be "off"
                }
                d.mosi_flop = self.mosi_off; // Set the mosi signal to be "off"
            }
            SPIState::Dwell => {
                if strobe_output {
                    // Dwell timeout has reached zero
                    d.state = SPIState::LoadBit; // Transition to the loadbit state
                }
            }
            SPIState::LoadBit => {
                if q.pointer != 0 {
                    d.mosi_flop = q.register_out.get_bit(pointerm1 as usize); //Fetch the corresponding bit out of the register
                    d.pointer = pointerm1; // Decrement the pointer
                    d.state = SPIState::MActive; // Move to the hold mclock low state
                    d.clock_state = self.cpol ^ self.cpha;
                } else {
                    d.mosi_flop = self.mosi_off; // Set the mosi signal to be "off"
                    d.clock_state = self.cpol;
                    d.state = SPIState::Finish; // No data, go back to idle
                }
            }
            SPIState::MActive => {
                if strobe_output {
                    d.state = SPIState::SampleMISO;
                }
            }
            SPIState::SampleMISO => {
                d.register_in = q.register_in.replace_bit(q.pointer as usize, i.miso);
                d.clock_state = !q.clock_state;
                d.state = SPIState::MIdle;
            }
            SPIState::MIdle => {
                if strobe_output {
                    d.state = SPIState::LoadBit;
                }
            }
            SPIState::Finish => {
                if strobe_output {
                    d.state = SPIState::Idle;
                    d.done_flop = true;
                }
            }
            _ => {
                d.state = SPIState::Idle;
            }
        }
        (o, d)
    }
}

#[test]
fn test_spi_master_basic() {
    let mode = SPIMode {
        cs_off: true,
        mosi_off: true,
        cpha: false,
        cpol: false,
    };
    let mut writer = vcd::Writer::new(std::fs::File::create("spi_master_basic.vcd").unwrap());
    let config = SPIControllerConfig::<64>::new(50_000_000, 1_000_000, mode);
    let mut state = SPIControllerState::<64>::default();
    let mut output = config.default_output();
    let mut inputs = SPIInputs {
        bits_outbound: 0,
        data_outbound: 0.into(),
        start_send: false,
        continued_transaction: false,
        miso: false,
    };
    writer.timescale(1, vcd::TimescaleUnit::PS).unwrap();
    writer.add_module("spi").unwrap();

    for clk in 0..1_000_000 {
        let (o, s) = config.update(state, inputs);
        state = s;
        output = o;
        inputs = SPIInputs {
            bits_outbound: 0,
            data_outbound: 0.into(),
            start_send: false,
            continued_transaction: false,
            miso: false,
        };
        writer.timestamp(clk).unwrap();
    }
}
