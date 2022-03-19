use crate::core::prelude::*;
use crate::sim::sdr_sdram::chip::SDRAMSimulator;
use crate::widgets::dff::DFF;
use crate::widgets::prelude::{DelayLine, MemoryTimings, TristateBuffer};
use crate::widgets::sdram::cmd::{SDRAMCommand, SDRAMCommandEncoder};
use crate::widgets::sdram::SDRAMDriver;

// Controller states...
#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Boot,
    Precharge1,
    AutoRefresh1,
    AutoRefresh2,
    LoadModeRegister,
    Idle,
    Refresh,
    ReadActivate,
    ReadCycle,
    WriteActivate,
    WriteCycle,
    Recovery,
    Precharge,
    Error,
}

// Limits - can only run a 4 bank SDRAM.  If you need a different
// number of banks, you will need to modify it.
//
// Constants:
//  R - Row bits in the address
//  C - Col bits in the address
//  D - Data bus width
//  L - Line width (multiple of D)
#[derive(LogicBlock)]
pub struct SDRAMBaseController<
    const R: usize,
    const C: usize,
    const L: usize,
    const D: usize,
> {
    pub clock: Signal<In, Clock>,
    pub sdram: SDRAMDriver<D>,
    // Command interface
    pub data_in: Signal<In, Bits<L>>,
    pub write_not_read: Signal<In, Bit>,
    pub cmd_strobe: Signal<In, Bit>,
    pub cmd_address: Signal<In, Bits<32>>,
    pub busy: Signal<Out, Bit>,
    pub data_out: Signal<Out, Bits<L>>,
    pub data_valid: Signal<Out, Bit>,
    pub error: Signal<Out, Bit>,
    cmd: Signal<Local, SDRAMCommand>,
    encode: SDRAMCommandEncoder,
    bufz: TristateBuffer<Bits<D>>,
    boot_delay: Constant<Bits<32>>,
    t_rp: Constant<Bits<32>>,
    t_rfc: Constant<Bits<32>>,
    t_refresh_max: Constant<Bits<32>>,
    t_rcd: Constant<Bits<32>>,
    t_wr: Constant<Bits<32>>,
    max_transfer_size: Constant<Bits<8>>,
    mode_register: Constant<Bits<13>>,
    cas_delay: Constant<Bits<3>>,
    state: DFF<State>,
    reg_data_write: DFF<Bits<L>>,
    reg_data_read: DFF<Bits<L>>,
    reg_address: DFF<Bits<32>>,
    delay_counter: DFF<Bits<32>>,
    refresh_counter: DFF<Bits<32>>,
    transfer_counter: DFF<Bits<8>>,
    read_valid: DelayLine<Bit, 8, 3>,
    addr_bank: Signal<Local, Bits<2>>,
    addr_row: Signal<Local, Bits<13>>,
    addr_col: Signal<Local, Bits<13>>,
    write_pending: DFF<Bit>,
    read_pending: DFF<Bit>,
    read_ready: DFF<Bit>,
    row_bits: Constant<Bits<32>>,
    col_bits: Constant<Bits<32>>,
    data_bits: Constant<Bits<L>>,
    data_shift_in: Constant<Bits<L>>,
    data_out_counter: DFF<Bits<8>>,
}

impl<const R: usize, const C: usize, const L: usize, const D: usize>
SDRAMBaseController<R, C, L, D>
{
    pub fn new(cas_delay: u32, timings: MemoryTimings) -> SDRAMBaseController<R, C, L, D> {
        assert_eq!(L % D, 0);
        // mode register definitions
        // A2:A0 are the burst length, this design does not use burst transfers
        // so A2:A0 are 0
        // A3 is 0 because of burst type sequential
        // A6:A4 define the CAS latency in clocks.  We assume 3 clocks of latency.
        // The rest of the bits should all be zero
        // So the mode register is basically just CAS << 4
        let mode_register = cas_delay << 4;
        Self {
            clock: Default::default(),
            sdram: Default::default(),
            cmd: Default::default(),
            data_in: Default::default(),
            write_not_read: Default::default(),
            cmd_strobe: Default::default(),
            cmd_address: Default::default(),
            busy: Default::default(),
            data_out: Default::default(),
            data_valid: Default::default(),
            error: Default::default(),
            bufz: Default::default(),
            boot_delay: Constant::new((timings.t_boot() + 10).into()),
            t_rp: Constant::new((timings.t_rp()).into()),
            t_rfc: Constant::new((timings.t_rfc()).into()),
            t_refresh_max: Constant::new((timings.t_refresh_max() * 9 / 10).into()),
            t_rcd: Constant::new((timings.t_rcd()).into()),
            t_wr: Constant::new((timings.t_wr()).into()),
            max_transfer_size: Constant::new({L/D}.into()),
            mode_register: Constant::new(mode_register.into()),
            cas_delay: Constant::new(cas_delay.into()),
            state: Default::default(),
            reg_data_write: Default::default(),
            reg_data_read: Default::default(),
            reg_address: Default::default(),
            delay_counter: Default::default(),
            refresh_counter: Default::default(),
            transfer_counter: Default::default(),
            read_valid: Default::default(),
            addr_bank: Default::default(),
            addr_row: Default::default(),
            addr_col: Default::default(),
            write_pending: Default::default(),
            row_bits: Constant::new(R.into()),
            col_bits: Constant::new(C.into()),
            data_bits: Constant::new(D.into()),
            data_shift_in: Constant::new({L - D}.into()),
            read_pending: Default::default(),
            data_out_counter: Default::default(),
            read_ready: Default::default(),
            encode: Default::default()
        }
    }
}

impl<const R: usize, const C: usize, const L: usize, const D: usize> Logic
for SDRAMBaseController<R, C, L, D>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal logic
        self.state.clk.next = self.clock.val();
        self.reg_data_write.clk.next = self.clock.val();
        self.reg_data_read.clk.next = self.clock.val();
        self.reg_address.clk.next = self.clock.val();
        self.delay_counter.clk.next = self.clock.val();
        self.refresh_counter.clk.next = self.clock.val();
        self.read_valid.clock.next = self.clock.val();
        self.transfer_counter.clk.next = self.clock.val();
        self.write_pending.clk.next = self.clock.val();
        self.read_pending.clk.next = self.clock.val();
        self.data_out_counter.clk.next = self.clock.val();
        self.read_ready.clk.next = self.clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.delay_counter.d.next = self.delay_counter.q.val() + 1_usize;
        self.refresh_counter.d.next = self.refresh_counter.q.val() + 1_usize;
        self.cmd.next = SDRAMCommand::NOP;
        self.sdram.address.next = 0_usize.into();
        self.sdram.bank.next = 0_usize.into();
        self.data_out.next = self.reg_data_read.q.val();
        self.reg_data_write.d.next = self.reg_data_write.q.val();
        self.reg_data_read.d.next = self.reg_data_read.q.val();
        self.reg_address.d.next = self.reg_address.q.val();
        self.write_pending.d.next = self.write_pending.q.val();
        self.read_pending.d.next = self.read_pending.q.val();
        self.data_out_counter.d.next = self.data_out_counter.q.val() + self.read_valid.data_out.val();
        self.data_valid.next = self.read_ready.q.val();
        // SDRAM signal drivers
        Signal::<InOut, Bits<D>>::link(&mut self.sdram.data, &mut self.bufz.bus);
        self.bufz.write_enable.next = false;
        self.bufz.write_data.next = self.reg_data_write.q.val().get_bits::<D>(0_usize);
        self.read_valid.data_in.next = false;
        self.read_valid.delay.next = self.cas_delay.val();
        // Calculate the read and write addresses
        self.addr_col.next = bit_cast::<13, C>(self.reg_address.q.val().get_bits::<C>(0_usize));
        self.addr_row.next = bit_cast::<13, R>(
            self.reg_address.q
                .val()
                .get_bits::<R>(self.col_bits.val().index()),
        );
        self.addr_bank.next = self
            .reg_address
            .q
            .val()
            .get_bits::<2>(self.col_bits.val().index() + self.row_bits.val().index());
        self.transfer_counter.d.next = self.transfer_counter.q.val();
        // State machine
        self.busy.next = (self.state.q.val() != State::Idle)
            | self.write_pending.q.val()
            | self.read_pending.q.val();
        match self.state.q.val() {
            State::Boot => {
                if self.delay_counter.q.val() == self.boot_delay.val() {
                    self.state.d.next = State::Precharge1;
                    self.cmd.next = SDRAMCommand::Precharge;
                    self.sdram.address.next = 0xFFF_usize.into();
                    self.delay_counter.d.next = 0_usize.into();
                }
            }
            State::Precharge1 => {
                if self.delay_counter.q.val() == self.t_rp.val() {
                    self.state.d.next = State::AutoRefresh1;
                    self.cmd.next = SDRAMCommand::AutoRefresh;
                    self.delay_counter.d.next = 0_usize.into();
                }
            }
            State::AutoRefresh1 => {
                if self.delay_counter.q.val() == self.t_rfc.val() {
                    self.state.d.next = State::AutoRefresh2;
                    self.cmd.next = SDRAMCommand::AutoRefresh;
                    self.delay_counter.d.next = 0_usize.into();
                }
            }
            State::AutoRefresh2 => {
                if self.delay_counter.q.val() == self.t_rfc.val() {
                    self.state.d.next = State::LoadModeRegister;
                    self.cmd.next = SDRAMCommand::LoadModeRegister;
                    self.sdram.address.next = self.mode_register.val();
                    self.delay_counter.d.next = 0_usize.into();
                }
            }
            State::LoadModeRegister => {
                if self.delay_counter.q.val() == 4_usize {
                    self.state.d.next = State::Idle;
                }
            }
            State::Idle => {
                self.delay_counter.d.next = 0_usize.into();
                self.transfer_counter.d.next = 0_usize.into();
                if self.refresh_counter.q.val() >= self.t_refresh_max.val() {
                    // Refresh takes the highest priority
                    self.cmd.next = SDRAMCommand::AutoRefresh;
                    self.state.d.next = State::Refresh;
                    self.refresh_counter.d.next = 0_usize.into();
                } else {
                    if self.read_pending.q.val() {
                        self.cmd.next = SDRAMCommand::Active;
                        self.sdram.bank.next = self.addr_bank.val();
                        self.sdram.address.next = self.addr_row.val();
                        self.state.d.next = State::ReadActivate;
                        self.read_pending.d.next = false;
                        self.data_out_counter.d.next = 0_u8.into();
                    } else if self.write_pending.q.val() {
                        self.cmd.next = SDRAMCommand::Active;
                        self.sdram.bank.next = self.addr_bank.val();
                        self.sdram.address.next = self.addr_row.val();
                        self.state.d.next = State::WriteActivate;
                        self.write_pending.d.next = false;
                    }
                }
            }
            State::Refresh => {
                if self.delay_counter.q.val() == self.t_rfc.val() {
                    self.state.d.next = State::Idle;
                }
            }
            State::ReadActivate => {
                if self.delay_counter.q.val() == self.t_rcd.val() {
                    self.state.d.next = State::ReadCycle;
                    self.transfer_counter.d.next = 0_usize.into();
                }
            }
            State::WriteActivate => {
                if self.delay_counter.q.val() == self.t_rcd.val() {
                    self.state.d.next = State::WriteCycle;
                    self.transfer_counter.d.next = 0_usize.into();
                }
            }
            State::WriteCycle => {
                if self.transfer_counter.q.val() < self.max_transfer_size.val() {
                    self.sdram.bank.next = self.addr_bank.val();
                    self.sdram.address.next = self.addr_col.val();
                    self.cmd.next = SDRAMCommand::Write;
                    self.bufz.write_enable.next = true;
                    self.transfer_counter.d.next = self.transfer_counter.q.val() + 1_usize;
                    self.reg_data_write.d.next = self.reg_data_write.q.val() >> self.data_bits.val();
                    self.reg_address.d.next = self.reg_address.q.val() + 1_usize;
                } else {
                    self.delay_counter.d.next = 0_usize.into();
                    self.state.d.next = State::Recovery;
                }
            }
            State::Recovery => {
                if self.delay_counter.q.val() == self.t_wr.val() {
                    self.cmd.next = SDRAMCommand::Precharge;
                    self.sdram.address.next = 0xFFFF_u32.into();
                    self.delay_counter.d.next = 0_usize.into();
                    self.state.d.next = State::Precharge;
                }
            }
            State::Precharge => {
                if self.delay_counter.q.val() == self.t_rp.val() {
                    self.state.d.next = State::Idle;
                }
            }
            State::ReadCycle => {
                if self.transfer_counter.q.val() < self.max_transfer_size.val() {
                    self.sdram.bank.next = self.addr_bank.val();
                    self.sdram.address.next = self.addr_col.val();
                    self.cmd.next = SDRAMCommand::Read;
                    self.transfer_counter.d.next = self.transfer_counter.q.val() + 1_usize;
                    self.read_valid.data_in.next = true;
                    self.reg_address.d.next = self.reg_address.q.val() + 1_usize;
                } else {
                    self.delay_counter.d.next = 0_usize.into();
                    self.state.d.next = State::Recovery;
                }
            }
            State::Error => {}
        }
        self.error.next = self.state.q.val() == State::Error;
        // Handle the input command latching
        if self.cmd_strobe.val() & !self.read_pending.q.val() & !self.write_pending.q.val() {
            self.reg_address.d.next = self.cmd_address.val();
            if self.write_not_read.val() {
                self.write_pending.d.next = true;
                self.reg_data_write.d.next = self.data_in.val();
            } else {
                self.read_pending.d.next = true;
            }
        }
        if self.read_valid.data_out.val() {
            self.reg_data_read.d.next =
                bit_cast::<L, D>(self.bufz.read_data.val()) << self.data_shift_in.val() |
                    (self.reg_data_read.q.val() >> self.data_bits.val());
        }
        self.read_ready.d.next = !self.read_ready.q.val() & (self.data_out_counter.q.val() == self.max_transfer_size.val());
        if self.read_ready.q.val() {
            self.data_out_counter.d.next = 0_usize.into();
        }
        // Connect up the command encoder
        self.sdram.cs_not.next = self.encode.cs_not.val();
        self.sdram.cas_not.next = self.encode.cas_not.val();
        self.sdram.ras_not.next = self.encode.ras_not.val();
        self.sdram.we_not.next = self.encode.we_not.val();
        self.encode.cmd.next = self.cmd.val();
        self.sdram.clk.next = self.clock.val();
    }
}

#[derive(LogicBlock)]
struct TestSDRAMDevice {
    dram: SDRAMSimulator<16>,
    cntrl: SDRAMBaseController<5, 5, 64, 16>,
    clock: Signal<In, Clock>,
}

impl Logic for TestSDRAMDevice {
    #[hdl_gen]
    fn update(&mut self) {
        SDRAMDriver::<16>::join(&mut self.cntrl.sdram, &mut self.dram.sdram);
        self.cntrl.clock.next = self.clock.val();
    }
}

#[cfg(test)]
fn make_test_device() -> TestSDRAMDevice {
    let timings = MemoryTimings::fast_boot_sim(100e6);
    let mut uut = TestSDRAMDevice {
        dram: SDRAMSimulator::new(timings),
        cntrl: SDRAMBaseController::new(3, timings),
        clock: Default::default(),
    };
    uut.clock.connect();
    uut.cntrl.data_in.connect();
    uut.cntrl.cmd_strobe.connect();
    uut.cntrl.cmd_address.connect();
    uut.cntrl.write_not_read.connect();
    uut.connect_all();
    uut
}

#[cfg(test)]
fn make_test_controller() -> SDRAMBaseController<5, 5, 64, 16> {
    let timings = MemoryTimings::fast_boot_sim(100e6);
    let mut uut = SDRAMBaseController::new(3, timings);
    uut.cmd.connect();
    uut.cmd_strobe.connect();
    uut.cmd_address.connect();
    uut.data_in.connect();
    uut.clock.connect();
    uut.sdram.link_connect_dest();
    uut.write_not_read.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_sdram_controller_is_synthesizable() {
    let uut = make_test_controller();
    let vlog = generate_verilog(&uut);
    yosys_validate("sdram_base_controller", &vlog).unwrap();
}

#[test]
fn test_unit_is_synthesizable() {
    let uut = make_test_device();
    let vlog = generate_verilog(&uut);
    yosys_validate("sdram_test_unit", &vlog).unwrap();
}

#[test]
fn test_unit_boots() {
    let uut = make_test_device();
    let mut sim = Simulation::new();
    sim.add_clock(4000, |x: &mut Box<TestSDRAMDevice>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<TestSDRAMDevice>| {
        let mut x = sim.init()?;
        x = sim.wait(10_000_000, x)?;
        sim_assert!(sim, !x.dram.test_error.val(), x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 12_000_000, "base_sdram_boot.vcd")
        .unwrap()
}

#[macro_export]
macro_rules! sdram_basic_write {
    ($sim: ident, $uut: ident, $cntrl: ident, $addr: expr, $data: expr) => {
        $uut = $sim.watch(|x| !x.$cntrl.busy.val(), $uut)?;
        $uut.$cntrl.cmd_address.next = ($addr).into();
        $uut.$cntrl.write_not_read.next = true;
        $uut.$cntrl.data_in.next = ($data).into();
        $uut.$cntrl.cmd_strobe.next = true;
        wait_clock_cycle!($sim, clock, $uut);
        $uut.$cntrl.cmd_strobe.next = false;
        $uut.$cntrl.cmd_address.next = 0_usize.into();
        $uut.$cntrl.write_not_read.next = false;
        $uut.$cntrl.data_in.next = 0_usize.into();
    }
}

#[macro_export]
macro_rules! sdram_basic_read {
    ($sim: ident, $uut: ident, $cntrl: ident, $addr: expr) => {
        {
            $uut = $sim.watch(|x| !x.$cntrl.busy.val(), $uut)?;
            $uut.$cntrl.cmd_address.next = ($addr).into();
            $uut.$cntrl.write_not_read.next = false;
            $uut.$cntrl.cmd_strobe.next = true;
            wait_clock_cycle!($sim, clock, $uut);
            $uut.$cntrl.cmd_strobe.next = false;
            $uut.$cntrl.cmd_address.next = 0_usize.into();
            $uut = $sim.watch(|x| x.$cntrl.data_valid.val(), $uut)?;
            $uut.$cntrl.data_out.val()
        }
    }
}


#[test]
fn test_unit_writes() {
    use rand::Rng;
    let uut = make_test_device();
    let mut sim = Simulation::new();
    let test_data = (0..256)
        .map(|_| rand::thread_rng().gen::<u64>())
        .collect::<Vec<_>>();
    sim.add_clock(5000, |x: &mut Box<TestSDRAMDevice>| {
        x.clock.next = !x.clock.val()
    });
    let send = test_data.clone();
    let recv = test_data.clone();
    sim.add_testbench(move |mut sim: Sim<TestSDRAMDevice>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        sdram_basic_write!(sim, x, cntrl, 0_usize, 0xDEAD_BEEF_CAFE_BABE_u64);
        sdram_basic_write!(sim, x, cntrl, 4_usize, 0x1234_ABCD_5678_EFFE_u64);
        let read = sdram_basic_read!(sim, x, cntrl, 2_usize);
        sim_assert_eq!(sim, read, 0x5678_EFFE_DEAD_BEEF_u64, x);
        let read = sdram_basic_read!(sim, x, cntrl, 4_usize);
        sim_assert_eq!(sim, read, 0x1234_ABCD_5678_EFFE_u64, x);
        for (ndx, val) in send.iter().enumerate() {
            sdram_basic_write!(sim, x, cntrl, ndx*4+8, *val);
        }
        for (ndx, val) in recv.iter().enumerate() {
            let read = sdram_basic_read!(sim, x, cntrl, ndx*4+8);
            sim_assert_eq!(sim, read, *val, x);
        }
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 100_000_000, "base_sdram_writes.vcd")
        .unwrap()
}
