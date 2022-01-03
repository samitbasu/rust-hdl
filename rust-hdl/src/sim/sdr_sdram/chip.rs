use crate::core::prelude::*;
use crate::sim::sdr_sdram::bank::MemoryBank;
use crate::sim::sdr_sdram::cmd::SDRAMCommand;
use crate::sim::sdr_sdram::timings::{nanos_to_clocks, MemoryTimings};
use crate::widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum MasterState {
    Boot,
    WaitPrecharge,
    Precharge,
    WaitAutorefresh,
    LoadModeRegister,
    Ready,
    Error,
}

// Clock enable, and DQM are ignored.
#[derive(LogicBlock)]
pub struct SDRAMSimulator<const D: usize> {
    pub clock: Signal<In, Clock>,
    pub cmd: Signal<In, SDRAMCommand>,
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
    bufz: TristateBuffer<Bits<D>>,
    banks: [MemoryBank<5, 5, 10, D>; 4],
    // Timings
    // Number of clocks to delay for boot initialization
    boot_delay: Constant<Bits<32>>,
    t_rp: Constant<Bits<32>>,
    load_mode_timing: Constant<Bits<32>>,
    t_rrd: Constant<Bits<32>>,
    banks_busy: Signal<Local, Bit>,
}

impl<const D: usize> Logic for SDRAMSimulator<D> {
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
        self.data.link(&mut self.bufz.bus);
        // Connect up the banks to the I/O buffer
        self.bufz.write_enable.next = false;
        self.bufz.write_data.next = 0_usize.into();
        for i in 0..4 {
            self.banks[i].clock.next = self.clock.val();
            self.banks[i].write_data.next = self.bufz.read_data.val();
            if self.banks[i].read_valid.val() {
                self.bufz.write_data.next = self.banks[i].read_data.val();
                self.bufz.write_enable.next = true;
            }
            self.banks[i].address.next = self.address.val();
            self.banks[i].cmd.next = self.cmd.val();
            self.banks[i].write_burst.next = self.write_burst_mode.q.val();
            self.banks[i].burst_len.next = 1_usize.into();
            self.banks[i].select.next = true;
            match self.burst_len.q.val().index() {
                0 => self.banks[i].burst_len.next = 1_usize.into(),
                1 => self.banks[i].burst_len.next = 2_usize.into(),
                2 => self.banks[i].burst_len.next = 4_usize.into(),
                3 => self.banks[i].burst_len.next = 8_usize.into(),
                _ => self.state.d.next = MasterState::Error,
            }
            self.banks[i].cas_delay.next = 2_usize.into();
            match self.cas_latency.q.val().index() {
                0 => self.banks[i].cas_delay.next = 0_usize.into(),
                2 => self.banks[i].cas_delay.next = 2_usize.into(),
                3 => self.banks[i].cas_delay.next = 3_usize.into(),
                _ => self.state.d.next = MasterState::Error,
            }
        }
        self.banks_busy.next = self.banks[0].busy.val()
            | self.banks[1].busy.val()
            | self.banks[2].busy.val()
            | self.banks[3].busy.val();
        match self.state.q.val() {
            MasterState::Boot => {
                if self.cmd.val() != SDRAMCommand::NOP {
                    self.state.d.next = MasterState::Error;
                }
                self.counter.d.next = self.counter.q.val() + 1_usize;
                if self.counter.q.val() == self.boot_delay.val() {
                    self.state.d.next = MasterState::WaitPrecharge;
                }
            }
            MasterState::WaitPrecharge => {
                match self.cmd.val() {
                    SDRAMCommand::NOP => {}
                    SDRAMCommand::Precharge => {
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
                if self.counter.q.val() == self.t_rp.val() {
                    self.state.d.next = MasterState::WaitAutorefresh;
                }
                if self.cmd.val() != SDRAMCommand::NOP {
                    self.state.d.next = MasterState::Error;
                }
            }
            MasterState::WaitAutorefresh => match self.cmd.val() {
                SDRAMCommand::NOP => {}
                SDRAMCommand::AutoRefresh => {
                    if self.banks_busy.val() {
                        self.state.d.next = MasterState::Error;
                    } else {
                        self.auto_refresh_init_counter.d.next =
                            self.auto_refresh_init_counter.q.val() + 1_usize;
                    }
                }
                SDRAMCommand::LoadModeRegister => {
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
                if self.cmd.val() != SDRAMCommand::NOP {
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
            MasterState::Error => {
                self.test_error.next = true;
            }
            MasterState::Ready => {
                self.test_ready.next = true;
            }
        }
        // Any banks that are in error mean the chip is in error.
        for i in 0_usize..4 {
            if self.banks[i].error.val() {
                self.state.d.next = MasterState::Error;
            }
        }
    }
}

impl<const D: usize> SDRAMSimulator<D> {
    pub fn new(clock_speed_hz: f64, timings: MemoryTimings) -> Self {
        // Calculate the number of picoseconds per clock cycle
        let boot_delay = nanos_to_clocks(timings.initial_delay_in_nanoseconds, clock_speed_hz) - 1;
        let precharge_delay =
            nanos_to_clocks(timings.t_rp_recharge_period_nanoseconds, clock_speed_hz) - 1;
        let bank_bank_delay = nanos_to_clocks(
            timings.t_rrd_bank_to_bank_activate_min_time_nanoseconds,
            clock_speed_hz,
        ) - 1;
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
            bufz: Default::default(),
            banks: array_init::array_init(|_| MemoryBank::new(clock_speed_hz, timings)),
            boot_delay: Constant::new(boot_delay.into()),
            t_rp: Constant::new(precharge_delay.into()),
            t_rrd: Constant::new(bank_bank_delay.into()),
            load_mode_timing: Constant::new(timings.load_mode_command_timing_clocks.into()),
            banks_busy: Default::default(),
        }
    }
}

#[cfg(test)]
fn mk_sdr_sim() -> SDRAMSimulator<16> {
    let mut uut = SDRAMSimulator::new(125_000_000.0, MemoryTimings::mt48lc8m16a2());
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
    sim.add_clock(4000, |x: &mut Box<SDRAMSimulator<16>>| {
        x.clock.next = !x.clock.val();
    });
    sim.add_testbench(move |mut sim: Sim<SDRAMSimulator<16>>| {
        let mut x = sim.init()?;
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_true!(sim, clock, x);
        // Wait for 100 microseconds
        // 100 microseconds = 100 * 1_000_000
        x = sim.wait(100_000_000, x)?;
        wait_clock_true!(sim, clock, x);
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::Precharge;
        x.address.next = 0xFFF_usize.into();
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 3);
        x.cmd.next = SDRAMCommand::AutoRefresh;
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 9);
        x.cmd.next = SDRAMCommand::AutoRefresh;
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 9);
        x.cmd.next = SDRAMCommand::LoadModeRegister;
        x.address.next = 0b000_0_00_011_0_011_u32.into();
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 20);
        sim_assert!(sim, x.state.q.val() == MasterState::Ready, x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 200_000_000, "sdr_init.vcd")
        .unwrap()
}
