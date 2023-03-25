use crate::prelude::*;
use rust_hdl_core::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    Start,
    Reading,
    Waiting,
    Flag,
    Ack,
    AckHold,
    WaitEndTransaction,
    CheckStop,
    Writing,
    WaitSCLHigh,
    WaitSCLLow,
    WaitSCLHighAck,
    CollectAck,
}

#[derive(LogicBlock, Default)]
pub struct I2CTarget {
    pub i2c: I2CBusDriver,
    pub clock: Signal<In, Clock>,
    pub from_bus: Signal<Out, Bits<8>>,
    pub bus_write: Signal<Out, Bit>,
    pub active: Signal<In, Bit>,
    pub stop: Signal<Out, Bit>,
    pub to_bus: Signal<In, Bits<8>>,
    pub write_enable: Signal<In, Bit>,
    pub ack: Signal<Out, Bit>,
    pub nack: Signal<Out, Bit>,
    pub write_ok: Signal<Out, Bit>,
    state: DFF<State>,
    scl_is_high: Signal<Local, Bit>,
    sda_is_high: Signal<Local, Bit>,
    sda_flop: DFF<Bit>,
    clear_sda: Signal<Local, Bit>,
    set_sda: Signal<Local, Bit>,
    read_bit: DFF<Bit>,
    count: DFF<Bits<4>>,
    accum: DFF<Bits<8>>,
}

impl Logic for I2CTarget {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal structures
        dff_setup!(self, clock, state, sda_flop, read_bit, count, accum);
        // Latch prevention
        self.i2c.scl.drive_low.next = false;
        self.i2c.sda.drive_low.next = self.sda_flop.q.val();
        self.sda_is_high.next = self.i2c.sda.line_state.val();
        self.scl_is_high.next = self.i2c.scl.line_state.val();
        self.set_sda.next = false;
        self.clear_sda.next = false;
        self.from_bus.next = self.accum.q.val();
        self.bus_write.next = false;
        self.stop.next = false;
        self.ack.next = false;
        self.nack.next = false;
        self.write_ok.next = false;
        // For now, can only read bits
        match self.state.q.val() {
            State::Idle => {
                self.set_sda.next = true;
                self.count.d.next = 0.into();
                if !self.sda_is_high.val() & self.scl_is_high.val() {
                    self.state.d.next = State::Start;
                }
            }
            State::Start => {
                if self.sda_is_high.val() {
                    self.state.d.next = State::Idle;
                } else if !self.sda_is_high.val() & !self.scl_is_high.val() {
                    self.state.d.next = State::Reading;
                }
            }
            State::Writing => {
                if self.accum.q.val().get_bit(7) {
                    self.set_sda.next = true;
                } else {
                    self.clear_sda.next = true;
                }
                self.count.d.next = self.count.q.val() + 1;
                self.accum.d.next = self.accum.q.val() << 1;
                self.state.d.next = State::WaitSCLHigh;
                if self.count.q.val() == 8 {
                    self.state.d.next = State::WaitSCLHighAck;
                }
            }
            State::WaitSCLHighAck => {
                if self.scl_is_high.val() {
                    self.set_sda.next = true;
                    self.state.d.next = State::CollectAck;
                }
            }
            State::CollectAck => {
                if !self.scl_is_high.val() {
                    if self.sda_is_high.val() {
                        self.nack.next = true;
                    } else {
                        self.ack.next = true;
                    }
                    self.state.d.next = State::Reading;
                }
            }
            State::WaitSCLHigh => {
                if self.scl_is_high.val() {
                    self.state.d.next = State::WaitSCLLow;
                }
            }
            State::WaitSCLLow => {
                if !self.scl_is_high.val() {
                    self.state.d.next = State::Writing;
                }
            }
            State::Reading => {
                self.write_ok.next = true;
                if self.write_enable.val() {
                    self.accum.d.next = self.to_bus.val();
                    self.count.d.next = 0.into();
                    self.state.d.next = State::Writing;
                }
                if self.scl_is_high.val() {
                    self.read_bit.d.next = self.sda_is_high.val();
                    self.state.d.next = State::Waiting;
                }
            }
            State::Waiting => {
                if !self.scl_is_high.val() {
                    self.accum.d.next =
                        (self.accum.q.val() << 1) | bit_cast::<8, 1>(self.read_bit.q.val().into());
                    self.count.d.next = self.count.q.val() + 1;
                    if self.count.q.val() == 7 {
                        self.state.d.next = State::Flag;
                    } else {
                        self.state.d.next = State::Reading;
                    }
                }
                if !self.read_bit.q.val() & self.sda_is_high.val() & self.scl_is_high.val() {
                    self.stop.next = true;
                    self.state.d.next = State::Idle;
                }
                if self.read_bit.q.val() & !self.sda_is_high.val() & self.scl_is_high.val() {
                    self.stop.next = true;
                    self.state.d.next = State::Idle;
                }
            }
            State::Flag => {
                self.bus_write.next = true;
                self.accum.d.next = 0.into();
                self.count.d.next = 0.into();
                self.state.d.next = State::Ack;
            }
            State::Ack => {
                self.clear_sda.next = self.active.val();
                if !self.active.val() {
                    self.state.d.next = State::WaitEndTransaction;
                }
                if self.scl_is_high.val() {
                    self.state.d.next = State::AckHold;
                }
            }
            State::AckHold => {
                if !self.scl_is_high.val() {
                    self.set_sda.next = true;
                    self.state.d.next = State::Reading;
                }
            }
            State::WaitEndTransaction => {
                // A STOP condition is signalled by
                // CLK -> H, SDA -> L
                // CLK -> H, SDA -> H
                if self.scl_is_high.val() & !self.sda_is_high.val() {
                    self.state.d.next = State::CheckStop;
                }
            }
            State::CheckStop => {
                if self.scl_is_high.val() & self.sda_is_high.val() {
                    self.state.d.next = State::Idle;
                    self.stop.next = true;
                }
            }
            _ => {
                self.state.d.next = State::Idle;
            }
        }
        if self.set_sda.val() {
            self.sda_flop.d.next = false;
        }
        if self.clear_sda.val() {
            self.sda_flop.d.next = true;
        }
    }
}
