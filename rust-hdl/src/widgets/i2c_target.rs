use crate::core::prelude::*;
use crate::widgets::prelude::*;
use std::time::Duration;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum I2CTargetState {
    Idle,
    Start,
    Reading,
    Waiting,
}

#[derive(LogicBlock, Default)]
pub struct I2CTarget {
    pub sda: Signal<InOut, Bit>,
    pub scl: Signal<InOut, Bit>,
    pub clock: Signal<In, Clock>,
    pub from_bus: Signal<Out, Bit>,
    pub bus_write: Signal<Out, Bit>,
    pub stop: Signal<Out, Bit>,
    state: DFF<I2CTargetState>,
    sda_driver: OpenDrainBuffer,
    scl_driver: OpenDrainBuffer,
    scl_is_high: Signal<Local, Bit>,
    sda_is_high: Signal<Local, Bit>,
    sda_flop: DFF<Bit>,
    clear_sda: Signal<Local, Bit>,
    set_sda: Signal<Local, Bit>,
    read_bit: DFF<Bit>,
}

impl Logic for I2CTarget {
    #[hdl_gen]
    fn update(&mut self) {
        self.sda.link(&mut self.sda_driver.bus);
        self.scl.link(&mut self.scl_driver.bus);
        // Clock the internal structures
        self.state.clk.next = self.clock.val();
        self.sda_flop.clk.next = self.clock.val();
        self.read_bit.clk.next = self.clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.sda_flop.d.next = self.sda_flop.q.val();
        self.read_bit.d.next = self.read_bit.q.val();
        self.scl_driver.enable.next = false;
        self.sda_driver.enable.next = self.sda_flop.q.val();
        self.sda_is_high.next = self.sda_driver.read_data.val();
        self.scl_is_high.next = self.scl_driver.read_data.val();
        self.set_sda.next = false;
        self.clear_sda.next = false;
        self.from_bus.next = self.read_bit.q.val();
        self.bus_write.next = false;
        self.stop.next = false;
        // For now, can only read bits
        match self.state.q.val() {
            I2CTargetState::Idle => {
                if !self.sda_is_high.val() & self.scl_is_high.val() {
                    self.state.d.next = I2CTargetState::Start;
                }
            }
            I2CTargetState::Start => {
                if self.sda_is_high.val() {
                    self.state.d.next = I2CTargetState::Idle;
                } else if !self.sda_is_high.val() & !self.scl_is_high.val() {
                    self.state.d.next = I2CTargetState::Reading;
                }
            }
            I2CTargetState::Reading => {
                if self.scl_is_high.val() {
                    self.read_bit.d.next = self.sda_is_high.val();
                    self.state.d.next = I2CTargetState::Waiting;
                }
            }
            I2CTargetState::Waiting => {
                if !self.scl_is_high.val() {
                    self.bus_write.next = true;
                    self.state.d.next = I2CTargetState::Reading;
                }
                if !self.read_bit.q.val() & self.sda_is_high.val() & self.scl_is_high.val() {
                    self.stop.next = true;
                    self.state.d.next = I2CTargetState::Idle;
                }
            }
        }
    }
}
