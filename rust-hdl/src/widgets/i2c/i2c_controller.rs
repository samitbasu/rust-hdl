use crate::core::prelude::*;
use crate::widgets::i2c::i2c_driver::{I2CDriver, I2CDriverCmd};
use crate::widgets::i2c::i2c_test_target::I2CTestTarget;
use crate::widgets::prelude::*;
use std::time::Duration;

use super::i2c_test_target::I2CTestBus;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum I2CControllerCmd {
    Noop,
    BeginWrite,
    Write,
    BeginRead,
    Read,
    EndTransmission,
    ReadLast,
}

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    SendBuffer,
    GetBuffer,
    WaitAck,
    WaitBit,
    Error,
    WaitDriverIdle,
}

#[derive(LogicBlock)]
pub struct I2CController {
    pub i2c: I2CBusDriver,
    pub clock: Signal<In, Clock>,
    pub cmd: Signal<In, I2CControllerCmd>,
    pub run: Signal<In, Bit>,
    pub busy: Signal<Out, Bit>,
    pub error: Signal<Out, Bit>,
    pub write_data_in: Signal<In, Bits<8>>,
    pub read_data_out: Signal<Out, Bits<8>>,
    pub read_valid: Signal<Out, Bit>,
    pub ack: Signal<Out, Bit>,
    pub nack: Signal<Out, Bit>,
    driver: I2CDriver,
    counter: DFF<Bits<4>>,
    read_data: DFF<Bits<8>>,
    write_data: DFF<Bits<8>>,
    state: DFF<State>,
    started: DFF<Bit>,
    last_read: DFF<Bit>,
}

impl I2CController {
    pub fn new(config: I2CConfig) -> Self {
        Self {
            i2c: Default::default(),
            clock: Default::default(),
            cmd: Default::default(),
            run: Default::default(),
            busy: Default::default(),
            error: Default::default(),
            write_data_in: Default::default(),
            read_data_out: Default::default(),
            read_valid: Default::default(),
            ack: Default::default(),
            nack: Default::default(),
            driver: I2CDriver::new(config),
            counter: Default::default(),
            read_data: Default::default(),
            write_data: Default::default(),
            state: Default::default(),
            started: Default::default(),
            last_read: Default::default(),
        }
    }
}

impl Logic for I2CController {
    #[hdl_gen]
    fn update(&mut self) {
        I2CBusDriver::link(&mut self.i2c, &mut self.driver.i2c);
        clock!(self, clock, driver);
        dff_setup!(self, clock, counter, read_data, write_data, state, started, last_read);
        self.driver.run.next = false;
        self.driver.cmd.next = I2CDriverCmd::Noop;
        // Default values
        self.busy.next = (self.state.q.val() != State::Idle) | self.driver.busy.val();
        self.error.next = false;
        self.read_data_out.next = self.read_data.q.val();
        self.read_valid.next = false;
        self.ack.next = false;
        self.nack.next = false;
        match self.state.q.val() {
            State::Idle => {
                if self.run.val() {
                    match self.cmd.val() {
                        I2CControllerCmd::BeginWrite => {
                            // Latch the write data as the address
                            // Only the lower 7 bits are used.
                            // The last bit is set to 0 to indicate a write
                            self.write_data.d.next = self.write_data_in.val() << 1;
                            if !self.started.q.val() {
                                self.driver.cmd.next = I2CDriverCmd::SendStart;
                            } else {
                                self.driver.cmd.next = I2CDriverCmd::Restart;
                            }
                            self.driver.run.next = true;
                            self.counter.d.next = 8.into();
                            self.state.d.next = State::SendBuffer;
                            self.started.d.next = true;
                        }
                        I2CControllerCmd::BeginRead => {
                            // Set the lowest bit to indicate a read
                            self.write_data.d.next = (self.write_data_in.val() << 1) | 1;
                            if !self.started.q.val() {
                                self.driver.cmd.next = I2CDriverCmd::SendStart;
                            } else {
                                self.driver.cmd.next = I2CDriverCmd::Restart;
                            }
                            self.driver.run.next = true;
                            self.counter.d.next = 8.into();
                            self.state.d.next = State::SendBuffer;
                            self.started.d.next = true;
                        }
                        I2CControllerCmd::EndTransmission => {
                            self.driver.cmd.next = I2CDriverCmd::SendStop;
                            self.driver.run.next = true;
                            self.state.d.next = State::WaitDriverIdle;
                            self.started.d.next = false;
                        }
                        I2CControllerCmd::Write => {
                            self.write_data.d.next = self.write_data_in.val();
                            self.counter.d.next = 8.into();
                            self.state.d.next = State::SendBuffer;
                        }
                        I2CControllerCmd::Read => {
                            self.counter.d.next = 8.into();
                            self.state.d.next = State::GetBuffer;
                            self.last_read.d.next = false;
                        }
                        I2CControllerCmd::ReadLast => {
                            self.counter.d.next = 8.into();
                            self.state.d.next = State::GetBuffer;
                            self.last_read.d.next = true;
                        }
                        I2CControllerCmd::Noop => {}
                        _ => {}
                    }
                    if self.driver.busy.val() {
                        self.state.d.next = State::Error;
                    }
                }
            }
            State::SendBuffer => {
                if !self.driver.busy.val() {
                    if self.counter.q.val() == 0 {
                        self.driver.cmd.next = I2CDriverCmd::GetBit;
                        self.driver.run.next = true;
                        self.state.d.next = State::WaitAck;
                    } else {
                        if self.write_data.q.val().get_bit(7) {
                            self.driver.cmd.next = I2CDriverCmd::SendTrue;
                        } else {
                            self.driver.cmd.next = I2CDriverCmd::SendFalse;
                        }
                        self.write_data.d.next = self.write_data.q.val() << 1;
                        self.driver.run.next = true;
                        self.counter.d.next = self.counter.q.val() - 1;
                    }
                }
            }
            State::GetBuffer => {
                if !self.driver.busy.val() {
                    if self.counter.q.val() == 0 {
                        if self.last_read.q.val() {
                            self.driver.cmd.next = I2CDriverCmd::SendTrue;
                        } else {
                            self.driver.cmd.next = I2CDriverCmd::SendFalse;
                        }
                        self.driver.run.next = true;
                        self.state.d.next = State::Idle;
                        self.read_valid.next = true;
                    } else {
                        self.driver.cmd.next = I2CDriverCmd::GetBit;
                        self.driver.run.next = true;
                        self.state.d.next = State::WaitBit;
                    }
                }
            }
            State::WaitBit => {
                if self.driver.read_valid.val() {
                    self.read_data.d.next = (self.read_data.q.val() << 1)
                        | bit_cast::<8, 1>(self.driver.read_bit.val().into());
                    self.counter.d.next = self.counter.q.val() - 1;
                    self.state.d.next = State::GetBuffer;
                }
            }
            State::WaitAck => {
                if self.driver.read_valid.val() {
                    if self.driver.read_bit.val() {
                        self.nack.next = true;
                    } else {
                        self.ack.next = true;
                    }
                    self.state.d.next = State::Idle;
                }
            }
            State::WaitDriverIdle => {
                if !self.driver.busy.val() {
                    self.state.d.next = State::Idle;
                }
            }
            State::Error => {
                self.error.next = true;
            }
            _ => {
                self.state.d.next = State::Idle;
            }
        }
        if self.driver.error.val() {
            self.state.d.next = State::Error;
        }
    }
}

#[test]
fn test_i2c_controller_synthesizes() {
    let config = I2CConfig {
        delay_time: Duration::from_nanos(1500),
        clock_speed_hz: 48_000_000,
    };
    let mut uut = I2CController::new(config);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("i2ccontroller", &vlog).unwrap()
}

// TODO - this probably needs some clean up
#[derive(LogicBlock)]
struct I2CControllerTest {
    clock: Signal<In, Clock>,
    controller: I2CController,
    target_1: I2CTestTarget,
    target_2: I2CTestTarget,
    test_bus: I2CTestBus<3>,
}

impl Logic for I2CControllerTest {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, controller, target_1, target_2);
        I2CBusDriver::join(&mut self.controller.i2c, &mut self.test_bus.endpoints[0]);
        I2CBusDriver::join(&mut self.target_1.i2c, &mut self.test_bus.endpoints[1]);
        I2CBusDriver::join(&mut self.target_2.i2c, &mut self.test_bus.endpoints[2]);
    }
}

impl Default for I2CControllerTest {
    fn default() -> Self {
        let config = I2CConfig {
            delay_time: Duration::from_micros(5),
            clock_speed_hz: 1_000_000,
        };
        Self {
            clock: Default::default(),
            controller: I2CController::new(config),
            target_1: I2CTestTarget::new(0x53),
            target_2: I2CTestTarget::new(0x57),
            test_bus: Default::default(),
        }
    }
}

#[test]
fn test_i2c_controller_operation() {
    use rand::Rng;
    struct TestCase {
        address: u8,
        reg_index: u8,
        val_msb: u8,
        val_lsb: u8,
    }

    let test_cases = (0..12)
        .map(|ndx| TestCase {
            address: if rand::thread_rng().gen::<bool>() {
                0x53_u8
            } else {
                0x57_u8
            },
            reg_index: ndx,
            val_msb: rand::thread_rng().gen::<u8>(),
            val_lsb: rand::thread_rng().gen::<u8>(),
        })
        .collect::<Vec<_>>();
    let mut uut = I2CControllerTest::default();
    uut.clock.connect();
    uut.controller.cmd.connect();
    uut.controller.run.connect();
    uut.controller.write_data_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    //println!("{}", vlog);
    let foo = I2CBusDriver::default();
    dbg!(I2CBusDriver::join_hdl("me", "foo", "bar"));
    yosys_validate("i2c_controller", &vlog).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(500_000, |x: &mut Box<I2CControllerTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<I2CControllerTest>| {
        let mut x = sim.init()?;
        // Check that a write to an invalid address is NACKed.
        i2c_begin_write!(sim, clock, x, 0x54_u32);
        sim_assert!(sim, x.controller.nack.val() & !x.controller.ack.val(), x);
        i2c_end_transmission!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 10);
        for test in &test_cases {
            i2c_begin_write!(sim, clock, x, test.address);
            sim_assert!(sim, x.controller.ack.val() & !x.controller.nack.val(), x);
            i2c_write!(sim, clock, x, test.reg_index);
            i2c_write!(sim, clock, x, test.val_msb);
            i2c_write!(sim, clock, x, test.val_lsb);
            i2c_end_transmission!(sim, clock, x);
        }
        wait_clock_cycles!(sim, clock, x, 10);
        for test in &test_cases {
            i2c_begin_write!(sim, clock, x, test.address);
            sim_assert!(sim, x.controller.ack.val() & !x.controller.nack.val(), x);
            i2c_write!(sim, clock, x, test.reg_index);
            sim_assert!(sim, x.controller.ack.val() & !x.controller.nack.val(), x);
            i2c_end_transmission!(sim, clock, x);
            i2c_begin_read!(sim, clock, x, test.address);
            sim_assert!(sim, x.controller.ack.val() & !x.controller.nack.val(), x);
            let byte = i2c_read!(sim, clock, x);
            sim_assert_eq!(sim, byte, test.val_msb.to_bits::<8>(), x);
            let byte = i2c_read_last!(sim, clock, x);
            sim_assert_eq!(sim, byte, test.val_lsb.to_bits::<8>(), x);
            i2c_end_transmission!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.run_to_file(
        Box::new(uut),
        160_000_000_000,
        &vcd_path!("i2c_controller.vcd"),
    )
    .unwrap()
}
