use rust_hdl::prelude::Bits;

use crate::{
    strobe::{StrobeConfig, StrobeState},
    synchronous::Synchronous,
};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
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

pub struct SPIControllerConfig<const N: usize> {
    pub clock_speed: u64,
    pub cs_off: bool,
    pub mosi_off: bool,
    pub speed_hz: u64,
    pub cpha: bool,
    pub cpol: bool,
    pub strobe: StrobeConfig,
}

#[derive(Default, Clone, Copy)]
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

pub struct SPIInputs<const N: usize> {
    pub bits_outbound: u16,
    pub data_outbound: Bits<N>,
    pub start_send: bool,
    pub continued_transaction: bool,
    pub miso: bool,
}

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
