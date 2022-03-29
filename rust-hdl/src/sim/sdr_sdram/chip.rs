use crate::core::prelude::*;
use crate::sim::sdr_sdram::bank::MemoryBank;
use crate::widgets::prelude::*;
use crate::widgets::sdram::cmd::SDRAMCommandDecoder;
use crate::widgets::sdram::SDRAMDevice;

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
    pub sdram: SDRAMDevice<D>,
    pub write_enable: Signal<Out, Bit>,
    pub test_error: Signal<Out, Bit>,
    pub test_ready: Signal<Out, Bit>,
    decode: SDRAMCommandDecoder,
    clock: Signal<Local, Clock>,
    cmd: Signal<Local, SDRAMCommand>,
    state: DFF<MasterState>,
    counter: DFF<Bits<32>>,
    auto_refresh_init_counter: DFF<Bits<32>>,
    write_burst_mode: DFF<Bit>,
    cas_latency: DFF<Bits<3>>,
    burst_type: DFF<Bit>,
    burst_len: DFF<Bits<3>>,
    op_mode: DFF<Bits<2>>,
    banks: [MemoryBank<6, 4, 10, D>; 4],
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
        self.clock.next = self.sdram.clk.val();
        self.state.clk.next = self.clock.val();
        self.counter.clk.next = self.clock.val();
        self.auto_refresh_init_counter.clk.next = self.clock.val();
        self.write_burst_mode.clk.next = self.clock.val();
        self.cas_latency.clk.next = self.clock.val();
        self.burst_type.clk.next = self.clock.val();
        self.burst_len.clk.next = self.clock.val();
        self.op_mode.clk.next = self.clock.val();
        // Connect the command decoder to the bus
        self.decode.we_not.next = self.sdram.we_not.val();
        self.decode.cas_not.next = self.sdram.cas_not.val();
        self.decode.ras_not.next = self.sdram.ras_not.val();
        self.decode.cs_not.next = self.sdram.cs_not.val();
        self.cmd.next = self.decode.cmd.val();
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
        // Connect up the banks to the I/O buffer
        self.write_enable.next = false;
        self.sdram.read_data.next = 0_usize.into();
        for i in 0..4 {
            self.banks[i].clock.next = self.clock.val();
            self.banks[i].write_data.next = self.sdram.write_data.val();
            if self.banks[i].read_valid.val() {
                self.sdram.read_data.next = self.banks[i].read_data.val();
                self.write_enable.next = self.banks[i].read_valid.val();
            }
            self.banks[i].address.next = self.sdram.address.val();
            self.banks[i].cmd.next = self.cmd.val();
            self.banks[i].write_burst.next = self.write_burst_mode.q.val();
            self.banks[i].burst_len.next = 1_usize.into();
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
            if self.sdram.bank.val().index() == i {
                self.banks[i].select.next = true;
            } else {
                self.banks[i].select.next = false;
            }
            if self.cmd.val() == SDRAMCommand::AutoRefresh {
                self.banks[i].select.next = true;
            }
            if (self.cmd.val() == SDRAMCommand::Precharge)
                & self.sdram.address.val().get_bit(10_usize)
            {
                self.banks[i].select.next = true;
            }
        }
        self.banks_busy.next = self.banks[0].busy.val()
            | self.banks[1].busy.val()
            | self.banks[2].busy.val()
            | self.banks[3].busy.val();
        match self.state.q.val() {
            MasterState::Boot => {
                if (self.cmd.val() != SDRAMCommand::NOP) & (self.counter.q.val().any()) {
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
                        if self.sdram.address.val().get_bit(10_usize) != true {
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
                        self.burst_len.d.next = self.sdram.address.val().get_bits::<3>(0_usize);
                        self.burst_type.d.next = self.sdram.address.val().get_bit(3_usize);
                        self.cas_latency.d.next = self.sdram.address.val().get_bits::<3>(4_usize);
                        self.op_mode.d.next = self.sdram.address.val().get_bits::<2>(7_usize);
                        self.write_burst_mode.d.next = self.sdram.address.val().get_bit(9_usize);
                        if self.sdram.address.val().get_bits::<2>(10_usize) != 0_usize {
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
    pub fn new(timings: MemoryTimings) -> Self {
        // Calculate the number of picoseconds per clock cycle
        let boot_delay = timings.t_boot();
        let precharge_delay = timings.t_rp() - 1;
        let bank_bank_delay = timings.t_rrd() - 1;
        Self {
            clock: Default::default(),
            cmd: Signal::default(),
            sdram: Default::default(),
            write_enable: Default::default(),
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
            banks: array_init::array_init(|_| MemoryBank::new(timings)),
            boot_delay: Constant::new(boot_delay.into()),
            t_rp: Constant::new(precharge_delay.into()),
            t_rrd: Constant::new(bank_bank_delay.into()),
            load_mode_timing: Constant::new((timings.load_mode_command_timing_clocks - 1).into()),
            banks_busy: Default::default(),
            decode: Default::default(),
        }
    }
}

#[cfg(test)]
fn mk_sdr_sim() -> SDRAMSimulator<16> {
    let mut uut = SDRAMSimulator::new(MemoryTimings::fast_boot_sim(125e6));
    uut.sdram.link_connect_dest();
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

#[macro_export]
macro_rules! sdram_cmd {
    ($uut: ident, $cmd: expr) => {
        match $cmd {
            SDRAMCommand::NOP => {
                $uut.sdram.ras_not.next = true;
                $uut.sdram.cas_not.next = true;
                $uut.sdram.we_not.next = true;
            }
            SDRAMCommand::BurstTerminate => {
                $uut.sdram.ras_not.next = true;
                $uut.sdram.cas_not.next = true;
                $uut.sdram.we_not.next = false;
            }
            SDRAMCommand::Read => {
                $uut.sdram.ras_not.next = true;
                $uut.sdram.cas_not.next = false;
                $uut.sdram.we_not.next = true;
            }
            SDRAMCommand::Write => {
                $uut.sdram.ras_not.next = true;
                $uut.sdram.cas_not.next = false;
                $uut.sdram.we_not.next = false;
            }
            SDRAMCommand::Active => {
                $uut.sdram.ras_not.next = false;
                $uut.sdram.cas_not.next = true;
                $uut.sdram.we_not.next = true;
            }
            SDRAMCommand::Precharge => {
                $uut.sdram.ras_not.next = false;
                $uut.sdram.cas_not.next = true;
                $uut.sdram.we_not.next = false;
            }
            SDRAMCommand::AutoRefresh => {
                $uut.sdram.ras_not.next = false;
                $uut.sdram.cas_not.next = false;
                $uut.sdram.we_not.next = true;
            }
            SDRAMCommand::LoadModeRegister => {
                $uut.sdram.ras_not.next = false;
                $uut.sdram.cas_not.next = false;
                $uut.sdram.we_not.next = false;
            }
        }
    };
}

#[macro_export]
macro_rules! sdram_activate {
    ($sim: ident, $clock: ident, $uut: ident, $bank: expr, $row: expr) => {
        sdram_cmd!($uut, SDRAMCommand::Active);
        $uut.sdram.address.next = ($row as u32).into();
        $uut.sdram.bank.next = ($bank as u32).into();
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::NOP);
    };
}

#[macro_export]
macro_rules! sdram_write {
    ($sim: ident, $clock: ident, $uut: ident, $bank: expr, $addr: expr, $data: expr) => {
        sdram_cmd!($uut, SDRAMCommand::Write);
        $uut.sdram.bank.next = ($bank as u32).into();
        $uut.sdram.write_data.next = ($data[0] as u32).into();
        $uut.sdram.address.next = ($addr as u32).into();
        wait_clock_cycle!($sim, $clock, $uut);
        for i in 1..($data).len() {
            sdram_cmd!($uut, SDRAMCommand::NOP);
            $uut.sdram.write_data.next = ($data[i] as u32).into();
            $uut.sdram.address.next = 0_u32.into();
            wait_clock_cycle!($sim, $clock, $uut);
        }
    };
}

#[macro_export]
macro_rules! sdram_read {
    ($sim: ident, $clock: ident, $uut: ident, $bank: expr, $addr: expr, $data: expr) => {
        sdram_cmd!($uut, SDRAMCommand::Read);
        $uut.sdram.bank.next = ($bank as u32).into();
        $uut.sdram.address.next = ($addr as u32).into();
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::NOP);
        wait_clock_cycles!($sim, $clock, $uut, 2); // Programmed CAS delay - 1
        for datum in $data {
            sdram_cmd!($uut, SDRAMCommand::NOP);
            sim_assert!($sim, $uut.sdram.read_data.val() == (datum as u32), $uut);
            wait_clock_cycle!($sim, $clock, $uut);
        }
    };
}

#[macro_export]
macro_rules! sdram_reada {
    ($sim: ident, $clock: ident, $uut: ident, $bank: expr, $addr: expr, $data: expr) => {
        sdram_cmd!($uut, SDRAMCommand::Read);
        $uut.sdram.bank.next = ($bank as u32).into();
        $uut.sdram.address.next = ($addr as u32 | 1024_u32).into(); // Signal autoprecharge
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::NOP);
        wait_clock_cycles!($sim, $clock, $uut, 2); // Programmed CAS delay
        for datum in $data {
            sdram_cmd!($uut, SDRAMCommand::NOP);
            sim_assert!($sim, $uut.sdram.read_data.val() == (datum as u32), $uut);
            wait_clock_cycle!($sim, $clock, $uut);
        }
    };
}

#[macro_export]
macro_rules! sdram_precharge_one {
    ($sim: ident, $clock: ident, $uut: ident, $bank: expr) => {
        sdram_cmd!($uut, SDRAMCommand::Precharge);
        $uut.sdram.bank.next = ($bank as u32).into();
        $uut.sdram.address.next = (0 as u32).into();
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::NOP);
    };
}

#[macro_export]
macro_rules! sdram_refresh {
    ($sim: ident, $clock: ident, $uut: ident, $timings: expr) => {
        sdram_cmd!($uut, SDRAMCommand::AutoRefresh);
        $uut.sdram.bank.next = (0 as u32).into();
        $uut.sdram.address.next = (0 as u32).into();
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::NOP);
        wait_clock_cycles!($sim, $clock, $uut, $timings.t_rfc());
    };
}

#[macro_export]
macro_rules! sdram_boot {
    ($sim: ident, $clock: ident, $uut: ident, $timings: ident) => {
        sdram_cmd!($uut, SDRAMCommand::NOP);
        wait_clock_true!($sim, $clock, $uut);
        // Wait for 100 microseconds
        // 100 microseconds = 100 * 1_000_000
        $uut = $sim.wait(
            ($timings.initial_delay_in_nanoseconds * 1000.0) as u64,
            $uut,
        )?;
        wait_clock_true!($sim, $clock, $uut);
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::Precharge);
        $uut.sdram.address.next = 0xFFF_usize.into();
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::NOP);
        wait_clock_cycles!($sim, $clock, $uut, $timings.t_rp());
        sdram_cmd!($uut, SDRAMCommand::AutoRefresh);
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::NOP);
        wait_clock_cycles!($sim, $clock, $uut, $timings.t_rfc());
        sdram_cmd!($uut, SDRAMCommand::AutoRefresh);
        wait_clock_cycle!($sim, $clock, $uut);
        sdram_cmd!($uut, SDRAMCommand::NOP);
        wait_clock_cycles!($sim, $clock, $uut, $timings.t_rfc());
    };
}

#[test]
fn test_sdram_init_works() {
    let uut = mk_sdr_sim();
    let mut sim = Simulation::new();
    // Clock period at 125 MHz is 8000ps
    sim.add_clock(4000, |x: &mut Box<SDRAMSimulator<16>>| {
        x.sdram.clk.next = !x.sdram.clk.val();
    });
    sim.add_testbench(move |mut sim: Sim<SDRAMSimulator<16>>| {
        let mut x = sim.init()?;
        let timings = MemoryTimings::fast_boot_sim(125e6);
        sdram_boot!(sim, clock, x, timings);
        sdram_cmd!(x, SDRAMCommand::LoadModeRegister);
        x.sdram.address.next = 0b000_0_00_011_0_011_u32.into();
        wait_clock_cycle!(sim, clock, x);
        sdram_cmd!(x, SDRAMCommand::NOP);
        wait_clock_cycles!(sim, clock, x, 5);
        sim_assert!(sim, x.state.q.val() == MasterState::Ready, x);
        // Activate row 14 on bank 2
        sdram_activate!(sim, clock, x, 2, 14);
        // Activate row 7 on bank 1
        wait_clock_cycles!(sim, clock, x, timings.t_rrd());
        sdram_activate!(sim, clock, x, 1, 7);
        wait_clock_cycles!(sim, clock, x, timings.t_ras());
        sdram_write!(
            sim,
            clock,
            x,
            2,
            16,
            [0xABCD, 0xDEAD, 0xBEEF, 0x1234, 0xFACE, 0x5EA1, 0xCAFE, 0xBABE]
        );
        sdram_precharge_one!(sim, clock, x, 2);
        sdram_write!(
            sim,
            clock,
            x,
            1,
            24,
            [0xABCE, 0xDEAE, 0xBEE0, 0x1235, 0xFACF, 0x5EA2, 0xCAFF, 0xBABF]
        );
        sdram_precharge_one!(sim, clock, x, 1);
        wait_clock_cycles!(sim, clock, x, timings.t_rp() + 1);
        sim_assert!(sim, !x.banks_busy.val(), x);
        sim_assert!(sim, x.state.q.val() == MasterState::Ready, x);
        sdram_activate!(sim, clock, x, 1, 7);
        wait_clock_cycles!(sim, clock, x, timings.t_rcd());
        sdram_read!(
            sim,
            clock,
            x,
            1,
            24,
            [0xABCE, 0xDEAE, 0xBEE0, 0x1235, 0xFACF, 0x5EA2, 0xCAFF, 0xBABF]
        );
        sdram_precharge_one!(sim, clock, x, 1);
        sdram_activate!(sim, clock, x, 2, 14);
        wait_clock_cycles!(sim, clock, x, timings.t_rcd());
        sdram_reada!(
            sim,
            clock,
            x,
            2,
            16,
            [0xABCD, 0xDEAD, 0xBEEF, 0x1234, 0xFACE, 0x5EA1, 0xCAFE, 0xBABE]
        );
        wait_clock_cycles!(sim, clock, x, timings.t_rp() + 1);
        sim_assert!(sim, !x.banks_busy.val(), x);
        sim_assert!(sim, x.state.q.val() == MasterState::Ready, x);
        sdram_refresh!(sim, clock, x, timings);
        sim_assert!(sim, !x.banks_busy.val(), x);
        sim_assert!(sim, x.state.q.val() == MasterState::Ready, x);
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 200_000_000, "sdr_init.vcd")
        .unwrap()
}
