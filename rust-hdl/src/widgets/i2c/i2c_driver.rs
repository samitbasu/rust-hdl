use crate::core::prelude::*;
use crate::widgets::prelude::*;
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct I2CConfig {
    pub delay_time: Duration,
    pub clock_speed_hz: u64,
}

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum I2CDriverCmd {
    Noop,
    SendStart,
    SendTrue,
    SendFalse,
    SendStop,
    GetBit,
    Restart,
}

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    Start,
    Send,
    Error,
    Clock,
    Stop,
    StopSetup,
    CheckArbitration,
    Restart,
    RestartDelay,
    Receive,
    ReceiveClock,
}

// Implement the bit-bang I2C interface as reported on Wikipedia
#[derive(LogicBlock)]
pub struct I2CDriver {
    pub i2c: I2CBusDriver,
    pub clock: Signal<In, Clock>,
    pub cmd: Signal<In, I2CDriverCmd>,
    pub run: Signal<In, Bit>,
    pub busy: Signal<Out, Bit>,
    pub error: Signal<Out, Bit>,
    pub read_bit: Signal<Out, Bit>,
    pub read_valid: Signal<Out, Bit>,
    state: DFF<State>,
    delay: Shot<32>,
    sda_is_high: Signal<Local, Bit>,
    scl_is_high: Signal<Local, Bit>,
    sda_flop: DFF<Bit>,
    scl_flop: DFF<Bit>,
    clear_sda: Signal<Local, Bit>,
    clear_scl: Signal<Local, Bit>,
    set_sda: Signal<Local, Bit>,
    set_scl: Signal<Local, Bit>,
}

impl Logic for I2CDriver {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, state, sda_flop, scl_flop);
        clock!(self, clock, delay);
        // Latch avoidance and default conditions
        self.delay.trigger.next = false;
        self.i2c.sda.drive_low.next = self.sda_flop.q.val();
        self.i2c.scl.drive_low.next = self.scl_flop.q.val();
        self.error.next = false;
        // Helpers to make the code more readable
        self.sda_is_high.next = self.i2c.sda.line_state.val();
        self.scl_is_high.next = self.i2c.scl.line_state.val();
        self.set_scl.next = false;
        self.clear_scl.next = false;
        self.set_sda.next = false;
        self.clear_sda.next = false;
        self.read_bit.next = self.sda_is_high.val();
        self.read_valid.next = false;
        self.busy.next = (self.state.q.val() != State::Idle) & (self.state.q.val() != State::Error);
        match self.state.q.val() {
            State::Idle => {
                if self.run.val() {
                    match self.cmd.val() {
                        I2CDriverCmd::SendStart => {
                            // if (read_SDA() == 0) {
                            //     arbitration_lost();
                            // }
                            if !self.sda_is_high.val() {
                                self.state.d.next = State::Error
                            } else {
                                //  clear_SDA();
                                //  I2C_delay();
                                self.clear_sda.next = true;
                                self.delay.trigger.next = true;
                                self.state.d.next = State::Start
                            }
                        }
                        I2CDriverCmd::SendTrue => {
                            // set_SDA()
                            self.set_sda.next = true;
                            // I2C_delay()
                            self.delay.trigger.next = true;
                            self.state.d.next = State::Send
                        }
                        I2CDriverCmd::GetBit => {
                            // set_SDA() - let target drive the SDA line
                            self.set_sda.next = true;
                            // I2C_delay()
                            self.delay.trigger.next = true;
                            self.state.d.next = State::Receive;
                        }
                        I2CDriverCmd::SendFalse => {
                            // clear_SDA()
                            self.clear_sda.next = true;
                            // I2C_delay()
                            self.delay.trigger.next = true;
                            self.state.d.next = State::Send
                        }
                        I2CDriverCmd::SendStop => {
                            // clear_SDA();
                            // I2C_delay();
                            self.clear_sda.next = true;
                            self.delay.trigger.next = true;
                            self.state.d.next = State::Stop
                        }
                        I2CDriverCmd::Restart => {
                            //   set_SDA();
                            //   I2C_delay();
                            self.set_sda.next = true;
                            self.delay.trigger.next = true;
                            self.state.d.next = State::Restart
                        }
                        I2CDriverCmd::Noop => {}
                        _ => {}
                    }
                }
            }
            State::Start => {
                if self.delay.fired.val() {
                    //  clear_SCL();
                    self.clear_scl.next = true;
                    self.state.d.next = State::Idle;
                }
            }
            State::Stop => {
                if self.delay.fired.val() {
                    // set_SCL()
                    self.set_scl.next = true;
                    self.delay.trigger.next = true;
                    self.state.d.next = State::StopSetup;
                }
            }
            State::Error => {
                self.error.next = true;
            }
            State::Send => {
                if self.delay.fired.val() {
                    // set_SCL()
                    self.set_scl.next = true;
                    self.state.d.next = State::Clock;
                    // I2C_delay()
                    self.delay.trigger.next = true;
                }
            }
            State::Receive => {
                if self.delay.fired.val() {
                    // set SCL()
                    self.set_scl.next = true;
                    self.state.d.next = State::ReceiveClock;
                    // I2C_delay()
                    self.delay.trigger.next = true;
                }
            }
            State::Clock => {
                if self.delay.fired.val() {
                    self.clear_scl.next = true;
                    self.state.d.next = State::Idle;
                }
            }
            State::ReceiveClock => {
                if self.delay.fired.val() {
                    self.read_valid.next = true;
                    self.clear_scl.next = true;
                    self.state.d.next = State::Idle;
                }
            }
            State::StopSetup => {
                if self.delay.fired.val() {
                    // set_SDA()
                    self.set_sda.next = true;
                    // I2C_delay()
                    self.delay.trigger.next = true;
                    self.state.d.next = State::CheckArbitration;
                }
            }
            State::Restart => {
                //   set_SCL();
                if self.delay.fired.val() {
                    self.set_scl.next = true;
                    self.state.d.next = State::RestartDelay;
                    self.delay.trigger.next = true;
                }
            }
            State::RestartDelay => {
                if self.delay.fired.val() {
                    self.clear_sda.next = true;
                    self.delay.trigger.next = true;
                    self.state.d.next = State::Start
                }
            }
            State::CheckArbitration => {
                if self.delay.fired.val() {
                    // if (read_SDA() == 0) {
                    //    arbitration_lost();
                    // }
                    if !self.i2c.sda.line_state.val() {
                        self.state.d.next = State::Error;
                    } else {
                        self.state.d.next = State::Idle;
                    }
                }
            }
            _ => {
                self.state.d.next = State::Idle;
            }
        }
        if self.set_scl.val() {
            self.scl_flop.d.next = false;
        }
        if self.clear_scl.val() {
            self.scl_flop.d.next = true;
        }
        if self.set_sda.val() {
            self.sda_flop.d.next = false;
        }
        if self.clear_sda.val() {
            self.sda_flop.d.next = true;
        }
        if self.run.val() & self.busy.val() {
            self.state.d.next = State::Error;
        }
    }
}

impl I2CDriver {
    pub fn new(config: I2CConfig) -> I2CDriver {
        I2CDriver {
            i2c: Default::default(),
            clock: Default::default(),
            cmd: Default::default(),
            run: Default::default(),
            busy: Default::default(),
            delay: Shot::new(config.clock_speed_hz, config.delay_time),
            sda_is_high: Default::default(),
            state: Default::default(),
            error: Default::default(),
            scl_is_high: Default::default(),
            sda_flop: Default::default(),
            scl_flop: Default::default(),
            clear_sda: Default::default(),
            clear_scl: Default::default(),
            set_sda: Default::default(),
            set_scl: Default::default(),
            read_bit: Default::default(),
            read_valid: Default::default(),
        }
    }
}

#[test]
fn test_i2c_driver_synthesizes() {
    let config = I2CConfig {
        delay_time: Duration::from_nanos(1500),
        clock_speed_hz: 48_000_000,
    };
    let mut uut = I2CDriver::new(config);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("i2cdriver", &vlog).unwrap()
}
