use crate::core::prelude::*;
use crate::sim::sdr_sdram::sdr_sdram_cmd_sim::SDRAMCommand;
use crate::sim::sdr_sdram::sdr_sdram_timings::{nanos_to_clocks, MemoryTimings};
use crate::widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum BankState {
    Idle,
    Active,
    Reading,
    Precharging,
    Writing,
    Error,
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
    mem: RAM<Bits<D>, A>,
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
    row_shift: Constant<Bits<A>>,
}

impl<const R: usize, const C: usize, const A: usize, const D: usize> MemoryBank<R, C, A, D> {
    pub fn new(clock_frequency_hz: f64, timings: MemoryTimings) -> Self {
        assert_eq!(R + C, A);
        let t_ras = nanos_to_clocks(
            timings.t_ras_row_active_min_time_nanoseconds,
            clock_frequency_hz,
        ) - 1;
        let t_rc = nanos_to_clocks(
            timings.t_rc_row_to_row_min_time_nanoseconds,
            clock_frequency_hz,
        ) - 1;
        let t_rcd = nanos_to_clocks(
            timings.t_rcd_row_to_column_min_time_nanoseconds,
            clock_frequency_hz,
        ) - 1;
        let t_rp =
            nanos_to_clocks(timings.t_rp_recharge_period_nanoseconds, clock_frequency_hz) - 1;
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
            mem: Default::default(),
            state: DFF::new(BankState::Idle),
            auto_precharge: Default::default(),
            active_row: Default::default(),
            burst_counter: Default::default(),
            active_col: Default::default(),
            delay_counter: Default::default(),
            t_activate: Default::default(),
            t_ras: Constant::new(t_ras.into()),
            t_rc: Constant::new(t_rc.into()),
            t_rcd: Constant::new(t_rcd.into()),
            t_rp: Constant::new(t_rp.into()),
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
        self.mem.write_data.next = self.write_data.val();
        self.mem.write_enable.next = false;
        self.read_data.next = self.mem.read_data.val();
        // Nned to model the Active->Precharge->Active min time... TODO
        // Start counting cycles for how long the row is active
        self.t_activate.d.next = self.t_activate.q.val() + 1_u32;
        self.busy.next = true;
        match self.state.q.val() {
            BankState::Idle => {
                // Reset the activate timer
                self.t_activate.d.next = 0_usize.into();
                self.busy.next = false;
                match self.cmd.val() {
                    SDRAMCommand::Active => {
                        // Activate the given row.
                        // Load the row into the row register
                        self.active_row.d.next = self.address.val().get_bits::<R>(0_usize);
                        // Reset the delay timer
                        self.delay_counter.d.next = 0_usize.into();
                        // Transition to the activating state.
                        self.state.d.next = BankState::Active;
                    }
                    SDRAMCommand::NOP => {}
                    _ => {
                        self.state.d.next = BankState::Error;
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
                        // Did the read finish?
                        if self.burst_counter.q.val() == self.burst_len.val() {
                            if self.auto_precharge.d.next {
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
            BankState::Writing => {
                self.mem.write_enable.next = true;
                match self.cmd.val() {
                    SDRAMCommand::NOP => {
                        // Process the write command
                        self.burst_counter.d.next = self.burst_counter.q.val() + 1_u32;
                        self.active_col.d.next = self.active_col.q.val() + 1_u32;
                        // Did the write finish?
                        if self.burst_counter.q.val() == self.burst_len.val() {
                            if self.auto_precharge.d.next {
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
    uut.connect_all();
    uut.burst_len.next = 8_usize.into();
    uut.write_burst.next = true;
    uut.cas_delay.next = 3_usize.into();
    uut.cmd.next = SDRAMCommand::NOP;
    uut
}

#[test]
fn test_bank_sim_synthesizes() {
    let uut = mk_bank_sim();
    let vlog = generate_verilog(&uut);
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
        wait_clock_cycle!(sim, clock, x);
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
