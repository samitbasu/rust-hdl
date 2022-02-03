use crate::core::prelude::*;
use crate::widgets::i2c::i2c_driver::{I2CDriver, I2CDriverCmd};
use crate::widgets::i2c::i2c_test_target::I2CTestTarget;
use crate::widgets::prelude::*;
use std::time::Duration;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum I2CControllerCmd {
    BeginWrite,
    BeginRead,
    EndTransmission,
}

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    SendStart,
    SendBuffer,
    WaitAck,
    Error,
}

#[derive(LogicBlock)]
pub struct I2CController {
    // The I2C data line.  Must have an external pullup
    pub sda: Signal<InOut, Bit>,
    // The I2C clock line.  Must have an external pullup
    pub scl: Signal<InOut, Bit>,
    pub clock: Signal<In, Clock>,
    pub cmd: Signal<In, I2CControllerCmd>,
    pub run: Signal<In, Bit>,
    pub busy: Signal<Out, Bit>,
    pub error: Signal<Out, Bit>,
    pub write_data_in: Signal<In, Bits<8>>,
    pub read_data_out: Signal<Out, Bits<8>>,
    pub read_valid: Signal<Out, Bit>,
    pub reset: Signal<In, Bit>,
    pub ack: Signal<Out, Bit>,
    pub nack: Signal<Out, Bit>,
    driver: I2CDriver,
    counter: DFF<Bits<4>>,
    read_data: DFF<Bits<8>>,
    write_data: DFF<Bits<8>>,
    state: DFF<State>,
}

impl I2CController {
    pub fn new(config: I2CConfig) -> Self {
        Self {
            sda: Default::default(),
            scl: Default::default(),
            clock: Default::default(),
            cmd: Default::default(),
            run: Default::default(),
            busy: Default::default(),
            error: Default::default(),
            write_data_in: Default::default(),
            read_data_out: Default::default(),
            read_valid: Default::default(),
            reset: Default::default(),
            ack: Default::default(),
            nack: Default::default(),
            driver: I2CDriver::new(config),
            counter: Default::default(),
            read_data: Default::default(),
            write_data: Default::default(),
            state: Default::default(),
        }
    }
}

impl Logic for I2CController {
    #[hdl_gen]
    fn update(&mut self) {
        self.sda.link(&mut self.driver.sda);
        self.scl.link(&mut self.driver.scl);
        self.driver.clock.next = self.clock.val();
        self.driver.run.next = false;
        self.driver.cmd.next = I2CDriverCmd::Noop;
        // Clock the flops
        self.counter.clk.next = self.clock.val();
        self.read_data.clk.next = self.clock.val();
        self.write_data.clk.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        // Latch prevention
        self.counter.d.next = self.counter.q.val();
        self.read_data.d.next = self.read_data.q.val();
        self.write_data.d.next = self.write_data.q.val();
        self.state.d.next = self.state.q.val();
        // Default values
        self.busy.next = false;
        self.error.next = false;
        self.read_data_out.next = self.read_data.q.val();
        self.read_valid.next = false;
        self.ack.next = false;
        match self.state.q.val() {
            State::Idle => {
                if self.run.val() {
                    match self.cmd.val() {
                        I2CControllerCmd::BeginWrite => {
                            // Latch the write data as the address
                            // Only the lower 7 bits are used.
                            // The last bit is set to 0 to indicate a write
                            self.write_data.d.next = self.write_data_in.val() << 1_usize;
                            self.driver.cmd.next = I2CDriverCmd::SendStart;
                            self.driver.run.next = true;
                            self.counter.d.next = 8_usize.into();
                            self.state.d.next = State::SendBuffer;
                        }
                        _ => {}
                    }
                }
            }
            State::SendBuffer => {
                if !self.driver.busy.val() {
                    if self.counter.q.val() == 0_usize {
                        self.driver.cmd.next = I2CDriverCmd::GetBit;
                        self.driver.run.next = true;
                        self.state.d.next = State::WaitAck;
                    } else {
                        if self.write_data.q.val().get_bit(7) {
                            self.driver.cmd.next = I2CDriverCmd::SendTrue;
                        } else {
                            self.driver.cmd.next = I2CDriverCmd::SendFalse;
                        }
                        self.write_data.d.next = self.write_data.q.val() << 1_usize;
                        self.driver.run.next = true;
                        self.counter.d.next = self.counter.q.val() - 1_usize;
                    }
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
            State::Error => {
                self.error.next = true;
            }
            _ => {}
        }
        if self.reset.val() {
            self.state.d.next = State::Idle;
        }
        self.driver.reset.next = self.reset.val();
    }
}

#[test]
fn test_i2c_controller_synthesizes() {
    let config = I2CConfig {
        delay_time: Duration::from_nanos(1500),
        clock_speed_hz: 48_000_000,
    };
    let mut uut = TopWrap::new(I2CController::new(config));
    uut.uut.scl.connect();
    uut.uut.sda.connect();
    uut.uut.clock.connect();
    uut.uut.write_data_in.connect();
    uut.uut.cmd.connect();
    uut.uut.run.connect();
    uut.uut.reset.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("i2ccontroller", &vlog).unwrap()
}

#[derive(LogicBlock)]
struct I2CControllerTest {
    clock: Signal<In, Clock>,
    controller: I2CController,
    target_1: I2CTestTarget,
    target_2: I2CTestTarget,
    pullup_sda: TristateBuffer<Bit>,
    pullup_scl: TristateBuffer<Bit>,
}

impl Logic for I2CControllerTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.controller.clock.next = self.clock.val();
        self.target_1.clock.next = self.clock.val();
        self.target_2.clock.next = self.clock.val();
        self.pullup_scl.bus.join(&mut self.controller.scl);
        self.pullup_sda.bus.join(&mut self.controller.sda);
        self.pullup_scl.bus.join(&mut self.target_1.scl);
        self.pullup_sda.bus.join(&mut self.target_1.sda);
        self.controller.sda.join(&mut self.target_1.sda);
        self.controller.scl.join(&mut self.target_1.scl);
        self.pullup_scl.bus.join(&mut self.target_2.scl);
        self.pullup_sda.bus.join(&mut self.target_2.sda);
        self.controller.sda.join(&mut self.target_2.sda);
        self.controller.scl.join(&mut self.target_2.scl);
        self.pullup_scl.write_data.next = true;
        self.pullup_sda.write_data.next = true;
        self.controller.reset.next = false;
    }
}

impl Default for I2CControllerTest {
    fn default() -> Self {
        let config = I2CConfig {
            delay_time: Duration::from_micros(10),
            clock_speed_hz: 1_000_000,
        };
        Self {
            clock: Default::default(),
            controller: I2CController::new(config),
            target_1: I2CTestTarget::new(0x53),
            target_2: I2CTestTarget::new(0x57),
            pullup_sda: Default::default(),
            pullup_scl: Default::default(),
        }
    }
}

#[test]
fn test_i2c_controller_operation() {
    let mut uut = I2CControllerTest::default();
    uut.clock.connect();
    uut.controller.cmd.connect();
    uut.controller.run.connect();
    uut.controller.write_data_in.connect();
    uut.pullup_scl.write_enable.connect();
    uut.pullup_sda.write_enable.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(500_000, |x: &mut Box<I2CControllerTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_custom_logic(|x| {
        x.pullup_sda.write_enable.next = !x.controller.sda.is_driving_tristate()
            & !x.target_1.sda.is_driving_tristate()
            & !x.target_2.sda.is_driving_tristate();
        x.pullup_scl.write_enable.next = !x.controller.scl.is_driving_tristate()
            & !x.target_1.scl.is_driving_tristate()
            & !x.target_2.scl.is_driving_tristate();
    });
    sim.add_testbench(move |mut sim: Sim<I2CControllerTest>| {
        let mut x = sim.init()?;
        // Check that a write to an invalid address is NACKed.
        wait_clock_true!(sim, clock, x);
        x.controller.cmd.next = I2CControllerCmd::BeginWrite;
        x.controller.write_data_in.next = 0x54_u32.into();
        x.controller.run.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.controller.run.next = false;
        x = sim.watch(|x| x.controller.nack.val(), x)?;
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 1_000_000_000, "i2c_controller.vcd")
        .unwrap()
}
