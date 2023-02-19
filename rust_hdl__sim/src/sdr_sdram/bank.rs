use rust_hdl__core::prelude::*;
use rust_hdl__widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum BankState {
    Boot,
    Idle,
    Active,
    Reading,
    Precharging,
    Writing,
    Error,
    Autorefreshing,
    WriteRecovery,
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
    pub address: Signal<In, Bits<13>>,
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
    t_wr: Constant<Bits<32>>,  // Write recovery time
    t_refresh_max: Constant<Bits<32>>,
    t_rfc: Constant<Bits<32>>,
    row_shift: Constant<Bits<A>>,
}

impl<const R: usize, const C: usize, const A: usize, const D: usize> MemoryBank<R, C, A, D> {
    pub fn new(timings: MemoryTimings) -> Self {
        assert_eq!(R + C, A);
        let t_ras = timings.t_ras() - 1;
        let t_rc = timings.t_rc() - 1;
        let t_rcd = timings.t_rcd() - 1;
        let t_rp = timings.t_rp() - 1;
        let t_refresh_max = timings.t_refresh_max() - 1;
        let t_rfc = timings.t_rfc() - 1;
        let t_wr = timings.t_wr() - 1;
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
            state: Default::default(),
            auto_precharge: Default::default(),
            active_row: Default::default(),
            burst_counter: Default::default(),
            active_col: Default::default(),
            delay_counter: Default::default(),
            refresh_counter: Default::default(),
            refresh_active: Default::default(),
            t_activate: Default::default(),
            t_ras: Constant::new(t_ras.to_bits()),
            t_rc: Constant::new(t_rc.to_bits()),
            t_rcd: Constant::new(t_rcd.to_bits()),
            t_rp: Constant::new(t_rp.to_bits()),
            t_wr: Constant::new(t_wr.to_bits()),
            t_refresh_max: Constant::new(t_refresh_max.to_bits()),
            t_rfc: Constant::new(t_rfc.to_bits()),
            row_shift: Constant::new(C.to_bits()),
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
        dff_setup!(
            self,
            clock,
            refresh_counter,
            refresh_active,
            write_reg,
            state,
            auto_precharge,
            active_row,
            burst_counter,
            active_col,
            delay_counter,
            t_activate
        );
        clock!(self, clock, delay_line, read_delay_line);
        self.delay_counter.d.next = self.delay_counter.q.val() + 1;
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
        self.delay_line.delay.next = self.cas_delay.val() - 2;
        // Start counting cycles for how long the row is active
        self.t_activate.d.next = self.t_activate.q.val() + 1;
        self.busy.next = true;
        self.read_delay_line.data_in.next = false;
        self.read_delay_line.delay.next = self.cas_delay.val() - 1;
        self.read_valid.next = self.read_delay_line.data_out.val();
        self.refresh_counter.d.next = self.refresh_counter.q.val() + self.refresh_active.q.val();
        match self.state.q.val() {
            BankState::Boot => {
                self.t_activate.d.next = 0xFFFF.into();
                self.state.d.next = BankState::Idle;
            }
            BankState::Idle => {
                self.busy.next = false;
                if self.select.val() {
                    match self.cmd.val() {
                        SDRAMCommand::Active => {
                            // Reset the activate timer
                            if self.t_activate.q.val() < self.t_rc.val() {
                                self.state.d.next = BankState::Error;
                            } else {
                                self.t_activate.d.next = 0.into();
                                // Activate the given row.
                                // Load the row into the row register
                                self.active_row.d.next = self.address.val().get_bits::<R>(0);
                                // Reset the delay timer
                                self.delay_counter.d.next = 0.into();
                                // Transition to the activating state.
                                self.state.d.next = BankState::Active;
                            }
                        }
                        SDRAMCommand::NOP => {}
                        SDRAMCommand::Precharge => {} // See ISSI docs.  Precharging an idle bank is a NOP
                        SDRAMCommand::AutoRefresh => {
                            if self.refresh_active.q.val()
                                & (self.refresh_counter.q.val() < self.t_rc.val())
                            {
                                self.state.d.next = BankState::Error;
                            } else {
                                self.state.d.next = BankState::Autorefreshing;
                                self.refresh_active.d.next = true;
                                self.refresh_counter.d.next = 0.into();
                            }
                        } // Handled at the chip level
                        SDRAMCommand::LoadModeRegister => {} // Ignored by banks
                        _ => {
                            self.state.d.next = BankState::Error;
                        }
                    }
                }
            }
            BankState::Active => {
                if self.select.val() {
                    match self.cmd.val() {
                        SDRAMCommand::NOP => {}
                        SDRAMCommand::Read => {
                            if self.t_activate.q.val() < self.t_rcd.val() {
                                self.state.d.next = BankState::Error;
                            } else {
                                // RCD is met, we want to read
                                self.active_col.d.next = self.address.val().get_bits::<C>(0);
                                self.burst_counter.d.next = 0.into();
                                self.state.d.next = BankState::Reading;
                                // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                                self.auto_precharge.d.next = self.address.val().get_bit(10);
                            }
                        }
                        SDRAMCommand::Write => {
                            if self.t_activate.q.val() < self.t_rcd.val() {
                                self.state.d.next = BankState::Error;
                            } else {
                                // RCD is met, we want to write
                                self.active_col.d.next = self.address.val().get_bits::<C>(0);
                                self.burst_counter.d.next = 0.into();
                                self.state.d.next = BankState::Writing;
                                // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                                self.auto_precharge.d.next = self.address.val().get_bit(10);
                            }
                        }
                        SDRAMCommand::Precharge => {
                            if self.t_activate.q.val() < self.t_ras.val() {
                                self.state.d.next = BankState::Error;
                            } else {
                                // RAS is met, we can close the current row
                                self.delay_counter.d.next = 0.into();
                                self.state.d.next = BankState::Precharging;
                            }
                        }
                        _ => {
                            self.state.d.next = BankState::Error;
                        }
                    }
                }
            }
            BankState::Reading => {
                // Process the read command
                self.burst_counter.d.next = self.burst_counter.q.val() + 1;
                self.active_col.d.next = self.active_col.q.val() + 1;
                self.read_delay_line.data_in.next = true;
                // Did the read finish?
                if self.burst_counter.q.val() == self.burst_len.val() {
                    self.read_delay_line.data_in.next = false;
                    if self.auto_precharge.q.val() {
                        self.delay_counter.d.next = 0.into();
                        self.state.d.next = BankState::Precharging;
                    } else {
                        self.state.d.next = BankState::Active
                    }
                }
                if self.select.val() {
                    match self.cmd.val() {
                        SDRAMCommand::NOP => {}
                        SDRAMCommand::Read => {
                            // RCD is met, we want to read
                            self.active_col.d.next = self.address.val().get_bits::<C>(0);
                            self.burst_counter.d.next = 0.into();
                            // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                            self.auto_precharge.d.next = self.address.val().get_bit(10);
                            self.state.d.next = BankState::Reading;
                        }
                        SDRAMCommand::Precharge => {
                            if self.auto_precharge.q.val() {
                                self.state.d.next = BankState::Error;
                            } else {
                                self.delay_counter.d.next = 0.into();
                                self.state.d.next = BankState::Precharging;
                            }
                        }
                        _ => {
                            self.state.d.next = BankState::Error;
                        }
                    }
                }
            }
            BankState::Precharging => {
                if self.delay_counter.q.val() == self.t_rp.val() {
                    self.state.d.next = BankState::Idle;
                }
                if self.select.val() {
                    match self.cmd.val() {
                        SDRAMCommand::NOP => {}
                        _ => {
                            self.state.d.next = BankState::Error;
                        }
                    }
                }
            }
            BankState::Autorefreshing => {
                if self.refresh_counter.q.val() == self.t_rfc.val() {
                    self.state.d.next = BankState::Idle;
                }
                if self.select.val() {
                    match self.cmd.val() {
                        SDRAMCommand::NOP => {}
                        _ => {
                            self.state.d.next = BankState::Error;
                        }
                    }
                }
            }
            BankState::Writing => {
                self.mem.write_enable.next = true;
                // Process the write command
                self.burst_counter.d.next = self.burst_counter.q.val() + 1;
                self.active_col.d.next = self.active_col.q.val() + 1;
                // Did the write finish?
                if self.burst_counter.q.val() == self.burst_len.val() - 1 {
                    self.delay_counter.d.next = 0.into();
                    if self.auto_precharge.q.val() {
                        self.state.d.next = BankState::Precharging;
                    } else {
                        self.state.d.next = BankState::WriteRecovery
                    }
                }
                if self.select.val() {
                    match self.cmd.val() {
                        SDRAMCommand::NOP => {}
                        SDRAMCommand::Write => {
                            self.active_col.d.next = self.address.val().get_bits::<C>(0);
                            self.burst_counter.d.next = 0.into();
                            // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                            self.auto_precharge.d.next = self.address.val().get_bit(10);
                            self.state.d.next = BankState::Writing;
                        }
                        SDRAMCommand::Precharge => {
                            if self.auto_precharge.q.val() {
                                self.state.d.next = BankState::Error;
                            } else {
                                self.delay_counter.d.next = 0.into();
                                self.state.d.next = BankState::Precharging;
                            }
                        }
                        _ => {
                            self.state.d.next = BankState::Error;
                        }
                    }
                }
            }
            BankState::Error => {
                self.error.next = true;
            }
            BankState::WriteRecovery => {
                if self.delay_counter.q.val() == self.t_wr.val() {
                    self.state.d.next = BankState::Active;
                }
                match self.cmd.val() {
                    SDRAMCommand::NOP => {}
                    SDRAMCommand::Read => {
                        self.active_col.d.next = self.address.val().get_bits::<C>(0);
                        self.burst_counter.d.next = 0.into();
                        self.state.d.next = BankState::Reading;
                        // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                        self.auto_precharge.d.next = self.address.val().get_bit(10);
                    }
                    SDRAMCommand::Write => {
                        self.active_col.d.next = self.address.val().get_bits::<C>(0);
                        self.burst_counter.d.next = 0.into();
                        self.state.d.next = BankState::Writing;
                        // Capture the auto precharge bit (bit 10) - this is the per the JEDEC spec
                        self.auto_precharge.d.next = self.address.val().get_bit(10);
                    }
                    _ => {
                        self.state.d.next = BankState::Error;
                    }
                }
            }
            _ => {
                self.state.d.next = BankState::Boot;
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
    let mut uut = MemoryBank::new(MemoryTimings::mt48lc8m16a2(500e6));
    uut.address.connect();
    uut.cmd.connect();
    uut.clock.connect();
    uut.cas_delay.connect();
    uut.write_burst.connect();
    uut.burst_len.connect();
    uut.write_data.connect();
    uut.select.connect();
    uut.connect_all();
    uut.burst_len.next = 8.into();
    uut.write_burst.next = true;
    uut.cas_delay.next = 3.into();
    uut.cmd.next = SDRAMCommand::NOP;
    uut.select.next = true;
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
        let timing = MemoryTimings::mt48lc8m16a2(500e6);

        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 30);
        x.cmd.next = SDRAMCommand::Active;
        x.address.next = 14.into();
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
        sim_assert_eq!(sim, x.state.q.val(), BankState::Idle, x);
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
        let timing = MemoryTimings::mt48lc8m16a2(500e6);

        wait_clock_true!(sim, clock, x);
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::Active;
        x.address.next = 14.into();
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
    sim.add_clock(clock_period / 2, |x: &mut Box<MemoryBank<5, 5, 10, 16>>| {
        x.clock.next = !x.clock.val();
    });
    let data = [
        0xABCD, 0xDEAD, 0xBEEF, 0x1234, 0xFACE, 0x5EA1, 0xCAFE, 0xBABE,
    ];
    sim.add_testbench(move |mut sim: Sim<MemoryBank<5, 5, 10, 16>>| {
        let mut x = sim.init()?;
        x = sim.watch(
            |x| x.clock.val().clk & (x.cmd.val() == SDRAMCommand::Read),
            x,
        )?;
        let cas_start_time = sim.time();
        x = sim.watch(|x| x.clock.val().clk & x.read_valid.val(), x)?;
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
            x = sim.watch(|x| !x.clock.val().clk & x.read_valid.val(), x)?;
            for val in &data {
                sim_assert!(sim, x.read_data.val() == *val, x);
                wait_clock_cycle!(sim, clock, x);
            }
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MemoryBank<5, 5, 10, 16>>| {
        let mut x = sim.init()?;
        let timing = MemoryTimings::mt48lc8m16a2(500e6);

        wait_clock_true!(sim, clock, x);
        wait_clock_cycles!(sim, clock, x, 30);
        x.cmd.next = SDRAMCommand::Active;
        x.address.next = 14.into();
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
        x.address.next = 0.into();
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
        x.address.next = 0.into();
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 8);
        // Read the data back out - with auto precharge
        x.cmd.next = SDRAMCommand::Read;
        x.address.next = 1024.into();
        wait_clock_cycle!(sim, clock, x);
        x.cmd.next = SDRAMCommand::NOP;
        wait_clock_cycles!(sim, clock, x, 10);
        let precharge_clocks = timing.t_rp();
        wait_clock_cycles!(sim, clock, x, precharge_clocks);
        sim_assert!(sim, !x.busy.val(), x);
        sim_assert!(sim, !x.error.val(), x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 1_000_000, &vcd_path!("sdram_write.vcd"))
        .unwrap();
}
