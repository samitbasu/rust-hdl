use crate::core::prelude::*;
use crate::widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum MasterState {
    Boot,
    WaitPrecharge,
    Precharge,
    WaitAutorefresh,
    Autorefresh,
    LoadModeRegister,
    Ready,
    Error,
}

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum MicronCommand {
    LoadModeRegister,
    AutoRefresh,
    Precharge,
    BurstTerminate,
    Write,
    Read,
    Active,
    NOP,
}

// Clock enable, and DQM are ignored.
#[derive(LogicBlock)]
pub struct MicronSDRSimulator<const D: usize> {
    pub clock: Signal<In, Clock>,
    pub cmd: Signal<In, MicronCommand>,
    pub bank: Signal<In, Bits<2>>,
    pub address: Signal<In, Bits<12>>,
    pub data: Signal<InOut, Bits<D>>,
    pub test_error: Signal<Out, Bit>,
    pub test_ready: Signal<Out, Bit>,
    state: DFF<MasterState>,
    counter: DFF<Bits<32>>,
    auto_refresh_init_counter: DFF<Bits<32>>,
    write_burst_mode: DFF<Bit>,
    cas_latency: DFF<Bits<3>>,
    burst_type: DFF<Bit>,
    burst_len: DFF<Bits<3>>,
    op_mode: DFF<Bits<2>>,
    // Timings
    // Number of clocks to delay for boot initialization
    boot_delay: Constant<Bits<32>>,
    precharge_delay: Constant<Bits<32>>,
    autorefresh_delay: Constant<Bits<32>>,
    load_mode_timing: Constant<Bits<32>>,
}

impl<const D: usize> Logic for MicronSDRSimulator<D> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock logic
        self.state.clk.next = self.clock.val();
        self.counter.clk.next = self.clock.val();
        self.auto_refresh_init_counter.clk.next = self.clock.val();
        self.write_burst_mode.clk.next = self.clock.val();
        self.cas_latency.clk.next = self.clock.val();
        self.burst_type.clk.next = self.clock.val();
        self.burst_len.clk.next = self.clock.val();
        self.op_mode.clk.next = self.clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.counter.d.next = self.counter.q.val();
        self.auto_refresh_init_counter.d.next = self.auto_refresh_init_counter.q.val();
        self.write_burst_mode.d.next = self.write_burst_mode.q.val();
        self.cas_latency.d.next = self.cas_latency.q.val();
        self.burst_type.d.next = self.burst_type.q.val();
        self.burst_len.d.next = self.burst_len.q.val();
        self.op_mode.d.next = self.op_mode.q.val();
        //
        self.test_error.next = false;
        self.test_ready.next = false;
        match self.state.q.val() {
            MasterState::Boot => {
                if self.cmd.val() != MicronCommand::NOP {
                    self.state.d.next = MasterState::Error;
                }
                self.counter.d.next = self.counter.q.val() + 1_usize;
                if self.counter.q.val() == self.boot_delay.val() {
                    self.state.d.next = MasterState::WaitPrecharge;
                }
            }
            MasterState::WaitPrecharge => {
                match self.cmd.val() {
                    MicronCommand::NOP => {}
                    MicronCommand::Precharge => {
                        // make sure the ALL bit is set
                        if self.address.val().get_bit(10_usize) != true {
                            self.state.d.next = MasterState::Error;
                        } else {
                            self.counter.d.next = 0_u32.into();
                            self.state.d.next = MasterState::Precharge;
                        }
                    }
                    _ => {
                        self.state.d.next = MasterState::Error;
                    }
                }
            }
            MasterState::Precharge => {
                self.counter.d.next = self.counter.q.val() + 1_usize;
                if self.counter.q.val() == self.precharge_delay.val() {
                    self.state.d.next = MasterState::WaitAutorefresh;
                }
                if self.cmd.val() != MicronCommand::NOP {
                    self.state.d.next = MasterState::Error;
                }
            }
            MasterState::WaitAutorefresh => match self.cmd.val() {
                MicronCommand::NOP => {}
                MicronCommand::AutoRefresh => {
                    self.counter.d.next = 0_usize.into();
                    self.state.d.next = MasterState::Autorefresh;
                }
                MicronCommand::LoadModeRegister => {
                    if self.auto_refresh_init_counter.q.val() < 2_u32.into() {
                        self.state.d.next = MasterState::Error;
                    } else {
                        self.counter.d.next = 0_usize.into();
                        self.state.d.next = MasterState::LoadModeRegister;
                        self.burst_len.d.next = self.address.val().get_bits::<3>(0_usize);
                        self.burst_type.d.next = self.address.val().get_bit(3_usize);
                        self.cas_latency.d.next = self.address.val().get_bits::<3>(4_usize);
                        self.op_mode.d.next = self.address.val().get_bits::<2>(7_usize);
                        self.write_burst_mode.d.next = self.address.val().get_bit(9_usize);
                        if self.address.val().get_bits::<3>(10_usize) != 0_usize {
                            self.state.d.next = MasterState::Error;
                        }
                    }
                }
                _ => {
                    self.state.d.next = MasterState::Error;
                }
            },
            MasterState::LoadModeRegister => {
                self.counter.d.next = self.counter.q.val() + 1_usize;
                if self.counter.q.val() == self.load_mode_timing.val() {
                    self.state.d.next = MasterState::Ready;
                }
                if self.cmd.val() != MicronCommand::NOP {
                    self.state.d.next = MasterState::Error;
                }
                if self.burst_len.q.val() > 3_u32.into() {
                    self.state.d.next = MasterState::Error;
                }
                if (self.cas_latency.q.val() > 3_u32.into()) | (self.cas_latency.q.val() == 0_usize)
                {
                    self.state.d.next = MasterState::Error;
                }
                if self.op_mode.q.val() != 0_u32 {
                    self.state.d.next = MasterState::Error;
                }
            }
            MasterState::Autorefresh => {
                self.counter.d.next = self.counter.q.val() + 1_usize;
                if self.counter.q.val() == self.autorefresh_delay.val() {
                    self.state.d.next = MasterState::WaitAutorefresh;
                    self.auto_refresh_init_counter.d.next =
                        self.auto_refresh_init_counter.q.val() + 1_usize;
                }
                if self.cmd.val() != MicronCommand::NOP {
                    self.state.d.next = MasterState::Error;
                }
            }
            MasterState::Error => {
                self.test_error.next = true;
            }
            MasterState::Ready => {
                self.test_ready.next = true;
            }
        }
    }
}

pub struct MemoryTimings {
    initial_delay_in_nanoseconds: f64,
    precharge_command_timing_nanoseconds: f64,
    autorefresh_command_timing_nanoseconds: f64,
    load_mode_command_timing_clocks: u32,
}

impl MemoryTimings {
    pub fn MT48LC8M16A2() -> Self {
        Self {
            initial_delay_in_nanoseconds: 100.0e3,
            precharge_command_timing_nanoseconds: 20.0,
            autorefresh_command_timing_nanoseconds: 66.0,
            load_mode_command_timing_clocks: 2,
        }
    }
}

impl<const D: usize> MicronSDRSimulator<D> {
    pub fn new(timings: MemoryTimings, clock_speed_hz: f64) -> Self {
        // Calculate the number of picoseconds per clock cycle
        let clock_period_in_nanos = 1.0e9 / clock_speed_hz;
        let boot_delay =
            (timings.initial_delay_in_nanoseconds / clock_period_in_nanos).ceil() as u32;
        let precharge_delay =
            (timings.precharge_command_timing_nanoseconds / clock_period_in_nanos).ceil() as u32;
        let autorefresh_delay =
            (timings.autorefresh_command_timing_nanoseconds / clock_period_in_nanos).ceil() as u32;
        Self {
            clock: Default::default(),
            cmd: Signal::default(),
            bank: Default::default(),
            address: Default::default(),
            data: Default::default(),
            test_error: Default::default(),
            test_ready: Default::default(),
            state: Default::default(),
            counter: Default::default(),
            auto_refresh_init_counter: Default::default(),
            write_burst_mode: Default::default(),
            cas_latency: Default::default(),
            burst_type: Default::default(),
            burst_len: Default::default(),
            op_mode: Default::default(),
            boot_delay: Constant::new(boot_delay.into()),
            precharge_delay: Constant::new(precharge_delay.into()),
            autorefresh_delay: Constant::new(autorefresh_delay.into()),
            load_mode_timing: Constant::new(timings.load_mode_command_timing_clocks.into()),
        }
    }
}

#[cfg(test)]
fn mk_sdr_sim() -> MicronSDRSimulator<16> {
    let mut uut = MicronSDRSimulator::new(MemoryTimings::MT48LC8M16A2(), 125_000_000.0);
    uut.clock.connect();
    uut.cmd.connect();
    uut.bank.connect();
    uut.address.connect();
    uut.data.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_sdram_sim_synthesizes() {
    let uut = mk_sdr_sim();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("sdram", &vlog).unwrap();
}

#[test]
fn test_sdram_init_works() {
    let uut = mk_sdr_sim();
    let mut sim = Simulation::new();
    // Clock period at 125 MHz is 8000ps
    sim.add_clock(4000, |x: &mut Box<MicronSDRSimulator<16>>| {
        x.clock.next = !x.clock.val();
    });
    sim.add_testbench(move |mut sim: Sim<MicronSDRSimulator<16>>| {
        let mut x = sim.init()?;
        x.cmd.next = MicronCommand::NOP;
        wait_clock_true!(sim, clock, x);
        // Wait for 100 microseconds
        // 100 microseconds = 100 * 1_000_000
        x = sim.wait(100_000_000, x)?;
        wait_clock_true!(sim, clock, x);
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = MicronCommand::Precharge;
        x.address.next = 0xFFF_usize.into();
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = MicronCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 4);
        x.cmd.next = MicronCommand::AutoRefresh;
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = MicronCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 10);
        x.cmd.next = MicronCommand::AutoRefresh;
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = MicronCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 10);
        x.cmd.next = MicronCommand::LoadModeRegister;
        x.address.next = 0b000_0_00_011_0_011_u32.into();
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = MicronCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 20);
        sim_assert!(sim, x.state.q.val() == MasterState::Ready, x);
        sim.done(x)
    });
    let mut vcd = vec![];
    let ret = sim.run_traced(Box::new(uut), 200_000_000, &mut vcd);
    std::fs::write("sdr_init.vcd", vcd).unwrap();
    ret.unwrap();
}
