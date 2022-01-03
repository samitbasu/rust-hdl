use crate::core::prelude::*;
use crate::sim::sdr_sdram::cmd::SDRAMCommand;
use crate::sim::sdr_sdram::timings::{nanos_to_clocks, MemoryTimings};
use crate::widgets::delay_line::DelayLine;
use crate::widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum BankState {
    Idle,
    Active,
    Reading,
    Precharging,
    Writing,
    Error,
    Autorefreshing,
}

// Bank state machine - a bank is simulated using BRAM.
// Tbis can be generalized later.  For now, we set the
// number of rows to 256, and the number of columns to 32
// That yields 8 row addresses, and 5 column addresses, for
// a total of 13 address bits.
#[derive(LogicBlock)]
pub struct MemoryBank<const R: usize, const C: usize, const A: usize, const D: usize> {
    // Constraint - A = R + C
    pub clock: Signal<In, Clock>,
    pub cas_delay: Signal<In, Bits<3>>,
    pub write_burst: Signal<In, Bit>,
    pub address: Signal<In, Bits<12>>,
    pub burst_len: Signal<In, Bits<4>>,
    pub cmd: Signal<In, SDRAMCommand>,
    pub error: Signal<Out, Bit>,
    pub busy: Signal<Out, Bit>,
    pub write_data: Signal<In, Bits<D>>,
    pub read_data: Signal<Out, Bits<D>>,
    pub read_valid: Signal<Out, Bit>,
    pub select: Signal<In, Bit>,
    delay_line: DelayLine<Bits<D>, 7, 3>,
    read_delay_line: DelayLine<Bit, 7, 3>,
    refresh_counter: DFF<Bits<32>>,
    refresh_active: DFF<Bit>,
    mem: RAM<Bits<D>, A>,
    write_reg: DFF<Bits<D>>,
    state: DFF<BankState>,
    auto_precharge: DFF<Bit>,
    active_row: DFF<Bits<R>>,
    burst_counter: DFF<Bits<4>>,
    active_col: DFF<Bits<C>>,
    delay_counter: DFF<Bits<32>>,
    t_activate: DFF<Bits<32>>,
    t_ras: Constant<Bits<32>>, // Min time from activate to precharge
    t_rc: Constant<Bits<32>>,  // Min time from active to activate
    t_rcd: Constant<Bits<32>>, // Min time from active to read/write
    t_rp: Constant<Bits<32>>,  // Precharge command time
    t_refresh_max: Constant<Bits<32>>,
    t_rfc: Constant<Bits<32>>,
    row_shift: Constant<Bits<A>>,
}

impl<const R: usize, const C: usize, const A: usize, const D: usize> MemoryBank<R, C, A, D> {
    pub fn new(clock_speed_hz: f64, timings: MemoryTimings) -> Self {
        assert_eq!(R + C, A);
        let t_ras = nanos_to_clocks(
            timings.t_ras_row_active_min_time_nanoseconds,
            clock_speed_hz,
        ) - 1;
        let t_rc =
            nanos_to_clocks(timings.t_rc_row_to_row_min_time_nanoseconds, clock_speed_hz) - 1;
        let t_rcd = nanos_to_clocks(
            timings.t_rcd_row_to_column_min_time_nanoseconds,
            clock_speed_hz,
        ) - 1;
        let t_rp = nanos_to_clocks(timings.t_rp_recharge_period_nanoseconds, clock_speed_hz) - 1;
        let t_refresh_max =
            nanos_to_clocks(timings.t_refresh_max_interval_nanoseconds, clock_speed_hz) - 1;
        let t_rfc =
            nanos_to_clocks(timings.t_rfc_autorefresh_period_nanoseconds, clock_speed_hz) - 1;
        Self {
            clock: Default::default(),
            cas_delay: Default::default(),
            write_burst: Default::default(),
            address: Default::default(),
            burst_len: Default::default(),
            cmd: Default::default(),
            error: Default::default(),
            busy: Default::default(),
            write_data: Default::default(),
            read_data: Default::default(),
            read_valid: Default::default(),
            select: Default::default(),
            delay_line: Default::default(),
            read_delay_line: Default::default(),
            mem: Default::default(),
            write_reg: Default::default(),
            state: DFF::new(BankState::Idle),
            auto_precharge: Default::default(),
            active_row: Default::default(),
            burst_counter: Default::default(),
            active_col: Default::default(),
            delay_counter: Default::default(),
            refresh_counter: Default::default(),
            refresh_active: Default::default(),
            t_activate: Default::default(),
            t_ras: Constant::new(t_ras.into()),
            t_rc: Constant::new(t_rc.into()),
            t_rcd: Constant::new(t_rcd.into()),
            t_rp: Constant::new(t_rp.into()),
            t_refresh_max: Constant::new(t_refresh_max.into()),
            t_rfc: Constant::new(t_rfc.into()),
            row_shift: Constant::new(C.into()),
        }
    }
}

impl<const R: usize, const C: usize, const A: usize, const D: usize> Logic
    for MemoryBank<R, C, A, D>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal logic
        self.mem.read_clock.next = self.clock.val();
        self.mem.write_clock.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.active_row.clk.next = self.clock.val();
        self.burst_counter.clk.next = self.clock.val();
        self.active_col.clk.next = self.clock.val();
        self.delay_counter.clk.next = self.clock.val();
        self.t_activate.clk.next = self.clock.val();
        self.auto_precharge.clk.next = self.clock.val();
        self.write_reg.clk.next = self.clock.val();
        self.delay_line.clock.next = self.clock.val();
        self.read_delay_line.clock.next = self.clock.val();
        self.refresh_counter.clk.next = self.clock.val();
        self.refresh_active.clk.next = self.clock.val();
        // Add latch prevention
        self.state.d.next = self.state.q.val();
        self.active_row.d.next = self.active_row.q.val();
        self.burst_counter.d.next = self.burst_counter.q.val();
        self.active_col.d.next = self.active_col.q.val();
        self.delay_counter.d.next = self.delay_counter.q.val();
        self.t_activate.d.next = self.t_activate.q.val();
        self.auto_precharge.d.next = self.auto_precharge.q.val();
        self.error.next = false;
        // Model the row-column multiplexing
        self.mem.read_address.next = (bit_cast::<A, R>(self.active_row.q.val())
            << self.row_shift.val())
            | bit_cast::<A, C>(self.active_col.q.val());
        self.mem.write_address.next = (bit_cast::<A, R>(self.active_row.q.val())
            << self.row_shift.val())
            | bit_cast::<A, C>(self.active_col.q.val());
        self.write_reg.d.next = self.write_data.val();
        self.mem.write_data.next = self.write_reg.q.val();
        self.mem.write_enable.next = false;
        self.delay_line.data_in.next = self.mem.read_data.val();
        self.read_data.next = self.delay_line.data_out.val();
        self.delay_line.delay.next = self.cas_delay.val() - 2_usize;
        // Start counting cycles for how long the row is active
        self.t_activate.d.next = self.t_activate.q.val() + 1_u32;
        self.busy.next = true;
        self.read_delay_line.data_in.next = false;
        self.read_delay_line.delay.next = self.cas_delay.val() - 1_usize;
        self.read_valid.next = self.read_delay_line.data_out.val();
        self.refresh_active.d.next = self.refresh_active.q.val();
        self.refresh_counter.d.next = self.refresh_counter.q.val() + self.refresh_active.q.val();
        match self.state.q.val() {
            BankState::Idle => {
                self.busy.next = false;
                if self.select.val() {
                    match self.cmd.val() {
                        SDRAMCommand::Active => {
                            // Reset the activate timer
                            if self.t_activate.q.val() < self.t_rcd.val() {
                                self.state.d.next = BankState::Error;
                            } else {
                                self.t_activate.d.next = 0_usize.into();
                                // Activate the given row.
                                // Load the row into the row register
                                self.active_row.d.next = self.address.val().get_bits::<R>(0_usize);
                                // Reset the delay timer
                                self.delay_counter.d.next = 0_usize.into();
                                // Transition to the activating state.
                                self.state.d.next = BankState::Active;
                            }
                        }
                        SDRAMCommand::NOP => {}
                        SDRAMCommand::Precharge => {} // See ISSI docs.  Precharging an idle bank is a NOP
                        SDRAMCommand::AutoRefresh => {
                            self.state.d.next = BankState::Autorefreshing;
                            self.refresh_active.d.next = true;
                            self.refresh_counter.d.next = 0_usize.into();
                        } // Handled at the chip level
                        SDRAMCommand::LoadModeRegister => {} // Ignored by banks
                        _ => {
                            self.state.d.next = BankState::Error;
                        }
                    }
                }
            }
            BankState::Active => {
                match self.cmd.val() {
                    SDRAMCommand::NOP => {}
                    SDRAMCommand::Read => {
                        if self.t_activate.q.val() < self.t_rcd.val() {
                            self.state.d.next = BankState::Error;
                        } else {
                            // RCD is met, we want to read
                            self.active_col.d.next = self.address.val().get_bits::<C>(0_usize);
                            self.burst_counter.d.next = 0_usize.into();
                            self.state.d.next = BankState::Reading;
                            // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                            self.auto_precharge.d.next = self.address.val().get_bit(10_usize);
                        }
                    }
                    SDRAMCommand::Write => {
                        if self.t_activate.q.val() < self.t_rcd.val() {
                            self.state.d.next = BankState::Error;
                        } else {
                            // RCD is met, we want to write
                            self.active_col.d.next = self.address.val().get_bits::<C>(0_usize);
                            self.burst_counter.d.next = 0_usize.into();
                            self.state.d.next = BankState::Writing;
                        }
                    }
                    SDRAMCommand::Precharge => {
                        if self.t_activate.q.val() < self.t_ras.val() {
                            self.state.d.next = BankState::Error;
                        } else {
                            // RAS is met, we can close the current row
                            self.delay_counter.d.next = 0_usize.into();
                            self.state.d.next = BankState::Precharging;
                        }
                    }
                    _ => {
                        self.state.d.next = BankState::Error;
                    }
                }
            }
            BankState::Reading => {
                match self.cmd.val() {
                    SDRAMCommand::NOP => {
                        // Process the read command
                        self.burst_counter.d.next = self.burst_counter.q.val() + 1_u32;
                        self.active_col.d.next = self.active_col.q.val() + 1_u32;
                        self.read_delay_line.data_in.next = true;
                        // Did the read finish?
                        if self.burst_counter.q.val() == self.burst_len.val() {
                            self.read_delay_line.data_in.next = false;
                            if self.auto_precharge.q.val() {
                                self.delay_counter.d.next = 0_usize.into();
                                self.state.d.next = BankState::Precharging;
                            } else {
                                self.state.d.next = BankState::Active
                            }
                        }
                    }
                    SDRAMCommand::Read => {
                        // RCD is met, we want to read
                        self.active_col.d.next = self.address.val().get_bits::<C>(0_usize);
                        self.burst_counter.d.next = 0_usize.into();
                        // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                        self.auto_precharge.d.next = self.address.val().get_bit(10_usize);
                    }
                    SDRAMCommand::Precharge => {
                        if self.auto_precharge.q.val() {
                            self.state.d.next = BankState::Error;
                        } else {
                            self.delay_counter.d.next = 0_usize.into();
                            self.state.d.next = BankState::Precharging;
                        }
                    }
                    _ => {
                        self.state.d.next = BankState::Error;
                    }
                }
            }
            BankState::Precharging => match self.cmd.val() {
                SDRAMCommand::NOP => {
                    self.delay_counter.d.next = self.delay_counter.q.val() + 1_usize;
                    if self.delay_counter.q.val() == self.t_rp.val() {
                        self.state.d.next = BankState::Idle;
                    }
                }
                _ => {
                    self.state.d.next = BankState::Error;
                }
            },
            BankState::Autorefreshing => match self.cmd.val() {
                SDRAMCommand::NOP => {
                    if self.refresh_counter.q.val() == self.t_rfc.val() {
                        self.state.d.next = BankState::Idle;
                    }
                }
                _ => {
                    self.state.d.next = BankState::Error;
                }
            },
            BankState::Writing => {
                self.mem.write_enable.next = true;
                match self.cmd.val() {
                    SDRAMCommand::NOP => {
                        // Process the write command
                        self.burst_counter.d.next = self.burst_counter.q.val() + 1_u32;
                        self.active_col.d.next = self.active_col.q.val() + 1_u32;
                        // Did the write finish?
                        if self.burst_counter.q.val() == self.burst_len.val() - 1_u32 {
                            if self.auto_precharge.q.val() {
                                self.delay_counter.d.next = 0_usize.into();
                                self.state.d.next = BankState::Precharging;
                            } else {
                                self.state.d.next = BankState::Active
                            }
                        }
                    }
                    SDRAMCommand::Write => {
                        self.active_col.d.next = self.address.val().get_bits::<C>(0_usize);
                        self.burst_counter.d.next = 0_usize.into();
                        // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                        self.auto_precharge.d.next = self.address.val().get_bit(10_usize);
                    }
                    SDRAMCommand::Precharge => {
                        if self.auto_precharge.q.val() {
                            self.state.d.next = BankState::Error;
                        } else {
                            self.delay_counter.d.next = 0_usize.into();
                            self.state.d.next = BankState::Precharging;
                        }
                    }
                    _ => {
                        self.state.d.next = BankState::Error;
                    }
                }
            }
            BankState::Error => {
                self.error.next = true;
            }
        }
        if self.refresh_counter.q.val() >= self.t_refresh_max.val() {
            self.state.d.next = BankState::Error;
        }
    }
}

// For test purposes, we run the clock a lot faster...
#[cfg(test)]
fn mk_bank_sim() -> MemoryBank<5, 5, 10, 16> {
    let mut uut = MemoryBank::new(500_000_000.0, MemoryTimings::mt48lc8m16a2());
    uut.address.connect();
    uut.cmd.connect();
    uut.clock.connect();
    uut.cas_delay.connect();
    uut.write_burst.connect();
    uut.burst_len.connect();
    uut.write_data.connect();
    uut.select.connect();
    uut.connect_all();
    uut.burst_len.next = 8_usize.into();
    uut.write_burst.next = true;
    uut.cas_delay.next = 3_usize.into();
    uut.cmd.next = SDRAMCommand::NOP;
    uut.select.next = true;
    uut
}

#[test]
fn test_bank_sim_synthesizes() {
    let uut = mk_bank_sim();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("sdram_bank", &vlog).unwrap();
}

#[test]
fn test_bank_activation_immediate_close_is_ok_with_delay() {
    let uut = mk_bank_sim();
    let mut sim = Simulation::new();
    // Clock period is 500 MHz or 2000ps
    let clock_period = 2000;
    sim.add_clock(clock_period / 2, |x: &mut Box<MemoryBank<5, 5, 10, 16>>| {
        x.clock.next = !x.clock.val();
    });
    sim.add_testbench(move |mut sim: Sim<MemoryBank<5, 5, 10, 16>>| {
        let mut x = sim.init()?;
        let timing = MemoryTimings::mt48lc8m16a2();
        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 30);
        x.cmd.next = SDRAMCommand::Active;
        x.address.next = 14_usize.into();
        wait_clock_cycle!(sim, clock, x);
        let start_time = sim.time();
        // Insert enough NOPS to meet the Active-to-precharge-time
        // Allow for 1 clock delay while loading the precharge time
        let wait_for_precharge =
            timing.t_ras_row_active_min_time_nanoseconds * 1000.0 - clock_period as f64;
        while sim.time() - start_time < wait_for_precharge as u64 {
            x.cmd.next = SDRAMCommand::NOP;
            wait_clock_cycle!(sim, clock, x);
        }
        x.cmd.next = SDRAMCommand::Precharge;
        wait_clock_cycle!(sim, clock, x);
        let start_time = sim.time();
        let precharge_time = timing.t_rp_recharge_period_nanoseconds * 1000.0 - clock_period as f64;
        while sim.time() - start_time < precharge_time as u64 {
            x.cmd.next = SDRAMCommand::NOP;
            wait_clock_cycle!(sim, clock, x);
            sim_assert!(sim, x.state.q.val() != BankState::Idle, x);
        }
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.state.q.val() == BankState::Idle, x);
        wait_clock_cycle!(sim, clock, x, 10);
        sim_assert!(sim, !x.error.val(), x);
        sim.done(x)
    });
    sim.run(Box::new(uut), 1_000_000).unwrap();
}

#[test]
fn test_bank_activation_immediate_close_fails_for_timing() {
    let uut = mk_bank_sim();
    let mut sim = Simulation::new();
    // Clock period is 500 MHz or 2000ps
    let clock_period = 2000;
    sim.add_clock(clock_period / 2, |x: &mut Box<MemoryBank<5, 5, 10, 16>>| {
        x.clock.next = !x.clock.val();
    });
    sim.add_testbench(move |mut sim: Sim<MemoryBank<5, 5, 10, 16>>| {
        let mut x = sim.init()?;
        let timing = MemoryTimings::mt48lc8m16a2();
        wait_clock_true!(sim, clock, x);
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::Active;
        x.address.next = 14_usize.into();
        wait_clock_cycle!(sim, clock, x);
        let start_time = sim.time();
        // Insert enough NOPS to meet the Active-to-precharge-time
        // Allow for 1 clock delay while loading the precharge time
        // Advance by 1 more clock so it fails
        let wait_for_precharge =
            (timing.t_ras_row_active_min_time_nanoseconds * 1000.0) as u64 - clock_period * 2;
        while sim.time() - start_time < wait_for_precharge as u64 {
            x.cmd.next = SDRAMCommand::NOP;
            wait_clock_cycle!(sim, clock, x);
        }
        x.cmd.next = SDRAMCommand::Precharge;
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycle!(sim, clock, x, 10);
        sim_assert!(sim, x.error.val(), x);
        sim.done(x)
    });
    sim.run(Box::new(uut), 1_000_000).unwrap();
}

#[test]
fn test_bank_write() {
    let uut = mk_bank_sim();
    let mut sim = Simulation::new();
    // Clock period is 500 MHz or 2000ps
    let clock_period = 2000;
    let clock_speed = 500_000_000;
    sim.add_clock(clock_period / 2, |x: &mut Box<MemoryBank<5, 5, 10, 16>>| {
        x.clock.next = !x.clock.val();
    });
    let data = [
        0xABCD_u16, 0xDEAD_u16, 0xBEEF, 0x1234, 0xFACE, 0x5EA1, 0xCAFE, 0xBABE,
    ];
    sim.add_testbench(move |mut sim: Sim<MemoryBank<5, 5, 10, 16>>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.clock.val().0 & (x.cmd.val() == SDRAMCommand::Read), x)?;
        let cas_start_time = sim.time();
        x = sim.watch(|x| x.clock.val().0 & x.read_valid.val(), x)?;
        let cas_end_time = sim.time();
        sim_assert!(
            sim,
            (cas_end_time - cas_start_time) == (x.cas_delay.val().index() as u64) * clock_period,
            x
        );
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MemoryBank<5, 5, 10, 16>>| {
        let mut x = sim.init()?;
        for _ in 0..2 {
            x = sim.watch(|x| !x.clock.val().0 & x.read_valid.val(), x)?;
            for val in &data {
                sim_assert!(sim, x.read_data.val() == *val, x);
                wait_clock_cycle!(sim, clock, x);
            }
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MemoryBank<5, 5, 10, 16>>| {
        let mut x = sim.init()?;
        let timing = MemoryTimings::mt48lc8m16a2();
        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 30);
        x.cmd.next = SDRAMCommand::Active;
        x.address.next = 14_usize.into();
        wait_clock_cycle!(sim, clock, x);
        let start_time = sim.time();
        // Insert enough NOPS to meet the Active-to-write-time
        // Allow for 1 clock delay while loading the write command
        let wait_for_active =
            (timing.t_rcd_row_to_column_min_time_nanoseconds * 1000.0) as u64 - clock_period;
        while sim.time() - start_time < wait_for_active as u64 {
            x.cmd.next = SDRAMCommand::NOP;
            wait_clock_cycle!(sim, clock, x);
        }
        x.cmd.next = SDRAMCommand::Write;
        x.write_data.next = data[0].into();
        x.address.next = 0_usize.into();
        wait_clock_cycle!(sim, clock, x);
        for datum in data.iter().skip(1) {
            x.cmd.next = SDRAMCommand::NOP;
            x.write_data.next = (*datum).into();
            wait_clock_cycle!(sim, clock, x);
        }
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 10);
        // Read the data back out
        x.cmd.next = SDRAMCommand::Read;
        x.address.next = 0_usize.into();
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 8);
        // Read the data back out - with auto precharge
        x.cmd.next = SDRAMCommand::Read;
        x.address.next = 1024_usize.into();
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 10);
        let precharge_clocks =
            nanos_to_clocks(timing.t_rp_recharge_period_nanoseconds, clock_speed as f64);
        wait_clock_cycles!(sim, clock, x, precharge_clocks);
        sim_assert!(sim, !x.busy.val(), x);
        sim_assert!(sim, !x.error.val(), x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 1_000_000, "sdram_write.vcd")
        .unwrap();
}
