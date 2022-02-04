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
    pub sda: Signal<InOut, Bit>,
    pub scl: Signal<InOut, Bit>,
    pub clock: Signal<In, Clock>,
    pub cmd: Signal<In, I2CDriverCmd>,
    pub run: Signal<In, Bit>,
    pub busy: Signal<Out, Bit>,
    pub error: Signal<Out, Bit>,
    pub reset: Signal<In, Bit>,
    pub read_bit: Signal<Out, Bit>,
    pub read_valid: Signal<Out, Bit>,
    state: DFF<State>,
    delay: Shot<32>,
    sda_driver: OpenDrainBuffer,
    scl_driver: OpenDrainBuffer,
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
        self.sda.link(&mut self.sda_driver.bus);
        self.scl.link(&mut self.scl_driver.bus);
        // Clock the internal structures
        self.state.clk.next = self.clock.val();
        self.delay.clock.next = self.clock.val();
        self.sda_flop.clk.next = self.clock.val();
        self.scl_flop.clk.next = self.clock.val();
        // Latch avoidance and default conditions
        self.state.d.next = self.state.q.val();
        self.delay.trigger.next = false;
        self.sda_flop.d.next = self.sda_flop.q.val();
        self.scl_flop.d.next = self.scl_flop.q.val();
        self.sda_driver.enable.next = self.sda_flop.q.val();
        self.scl_driver.enable.next = self.scl_flop.q.val();
        self.error.next = false;
        // Helpers to make the code more readable
        self.sda_is_high.next = self.sda_driver.read_data.val();
        self.scl_is_high.next = self.scl_driver.read_data.val();
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
                    if !self.sda_driver.read_data.val() {
                        self.state.d.next = State::Error;
                    } else {
                        self.state.d.next = State::Idle;
                    }
                }
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
        if self.reset.val() {
            self.state.d.next = State::Idle;
        }
        if self.run.val() & self.busy.val() {
            self.state.d.next = State::Error;
        }
    }
}

impl I2CDriver {
    pub fn new(config: I2CConfig) -> I2CDriver {
        I2CDriver {
            sda: Default::default(),
            scl: Default::default(),
            clock: Default::default(),
            cmd: Default::default(),
            run: Default::default(),
            busy: Default::default(),
            delay: Shot::new(config.clock_speed_hz, config.delay_time),
            sda_driver: Default::default(),
            scl_driver: Default::default(),
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
            reset: Default::default(),
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
    let mut uut = TopWrap::new(I2CDriver::new(config));
    uut.uut.scl.connect();
    uut.uut.sda.connect();
    uut.uut.clock.connect();
    uut.uut.cmd.connect();
    uut.uut.run.connect();
    uut.uut.reset.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("i2cdriver", &vlog).unwrap()
}

#[derive(LogicBlock)]
struct I2CDriverTest {
    clock: Signal<In, Clock>,
    driver: I2CDriver,
    target: I2CTarget,
    pullup_sda: TristateBuffer<Bit>,
    pullup_scl: TristateBuffer<Bit>,
}

impl Logic for I2CDriverTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.driver.clock.next = self.clock.val();
        self.target.clock.next = self.clock.val();
        self.pullup_scl.bus.join(&mut self.driver.scl);
        self.pullup_sda.bus.join(&mut self.driver.sda);
        self.pullup_scl.bus.join(&mut self.target.scl);
        self.pullup_sda.bus.join(&mut self.target.sda);
        self.driver.sda.join(&mut self.target.sda);
        self.driver.scl.join(&mut self.target.scl);
        self.pullup_scl.write_data.next = true;
        self.pullup_sda.write_data.next = true;
        self.driver.reset.next = false;
        self.target.active.next = false;
    }
}

impl Default for I2CDriverTest {
    fn default() -> Self {
        let config = I2CConfig {
            delay_time: Duration::from_micros(10),
            clock_speed_hz: 1_000_000,
        };
        Self {
            clock: Default::default(),
            driver: I2CDriver::new(config),
            target: I2CTarget::default(),
            pullup_sda: Default::default(),
            pullup_scl: Default::default(),
        }
    }
}

#[test]
fn test_i2c_driver_operation() {
    let mut uut = I2CDriverTest::default();
    uut.clock.connect();
    uut.driver.cmd.connect();
    uut.driver.run.connect();
    uut.pullup_scl.write_enable.connect();
    uut.pullup_sda.write_enable.connect();
    uut.connect_all();
    //uut.sim_sda.bus.
    let mut sim = Simulation::new();
    sim.add_clock(500_000, |x: &mut Box<I2CDriverTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_custom_logic(|x| {
        x.pullup_sda.write_enable.next =
            !x.driver.sda.is_driving_tristate() & !x.target.sda.is_driving_tristate();
        x.pullup_scl.write_enable.next =
            !x.driver.scl.is_driving_tristate() & !x.target.scl.is_driving_tristate();
    });
    let bits = [true, false, false, true, true, false, true, true];
    sim.add_testbench(move |mut sim: Sim<I2CDriverTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x = sim.watch(|x| x.target.bus_write.val(), x)?;
        sim_assert!(sim, x.target.from_bus.val() == 0b10011011_u8, x);
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| x.target.bus_write.val(), x)?;
        sim_assert!(sim, x.target.from_bus.val() == 0b10011011_u8, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<I2CDriverTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.driver.cmd.next = I2CDriverCmd::SendStart;
        x.driver.run.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.driver.run.next = false;
        x = sim.watch(|x| !x.driver.busy.val(), x)?;
        for bit in &bits {
            wait_clock_true!(sim, clock, x);
            x.driver.cmd.next = if *bit {
                I2CDriverCmd::SendTrue
            } else {
                I2CDriverCmd::SendFalse
            };
            x.driver.run.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.driver.run.next = false;
            x = sim.watch(|x| !x.driver.busy.val(), x)?;
        }
        x.driver.cmd.next = I2CDriverCmd::GetBit;
        x.driver.run.next = true;
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| !x.driver.busy.val(), x)?;
        x.driver.cmd.next = I2CDriverCmd::Restart;
        x.driver.run.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.driver.run.next = false;
        x = sim.watch(|x| !x.driver.busy.val(), x)?;
        for bit in &bits {
            wait_clock_true!(sim, clock, x);
            x.driver.cmd.next = if *bit {
                I2CDriverCmd::SendTrue
            } else {
                I2CDriverCmd::SendFalse
            };
            x.driver.run.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.driver.run.next = false;
            x = sim.watch(|x| !x.driver.busy.val(), x)?;
        }
        x.driver.cmd.next = I2CDriverCmd::GetBit;
        x.driver.run.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.driver.run.next = false;
        x = sim.watch(|x| !x.driver.busy.val(), x)?;
        x.driver.cmd.next = I2CDriverCmd::SendStop;
        x.driver.run.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.driver.run.next = false;
        x = sim.watch(|x| !x.driver.busy.val(), x)?;
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 1_000_000_000, "i2c_driver.vcd")
        .unwrap()
}
