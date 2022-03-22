use crate::core::prelude::*;
use crate::sim::sdr_sdram::chip::SDRAMSimulator;
use crate::widgets::dff::DFF;
use crate::widgets::prelude::{DelayLine, MemoryTimings, SynchronousFIFO};
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
}

// Limits - can only run a 4 bank SDRAM.  If you need a different
// number of banks, you will need to modify it.
#[derive(LogicBlock)]
pub struct SDRAMFIFOController<const R: usize, const C: usize, const P: usize, const D: usize> {
    pub sdram: SDRAMDriver<D>,
    pub write_enable: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    cmd: Signal<Local, SDRAMCommand>,
    encoder: SDRAMCommandEncoder,
    pub data_in: Signal<In, Bits<D>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub data_out: Signal<Out, Bits<D>>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
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
    write_pointer: DFF<Bits<P>>,
    read_pointer: DFF<Bits<P>>,
    delay_counter: DFF<Bits<32>>,
    refresh_counter: DFF<Bits<32>>,
    transfer_counter: DFF<Bits<8>>,
    active_bank: DFF<Bits<2>>,
    active_row: DFF<Bits<13>>,
    read_valid: DelayLine<Bit, 8, 3>,
    fp: SynchronousFIFO<Bits<D>, 6, 7, 1>,
    bp: SynchronousFIFO<Bits<D>, 6, 7, 8>,
    read_bank: Signal<Local, Bits<2>>,
    read_row: Signal<Local, Bits<13>>,
    read_col: Signal<Local, Bits<13>>,
    write_bank: Signal<Local, Bits<2>>,
    write_row: Signal<Local, Bits<13>>,
    write_col: Signal<Local, Bits<13>>,
    row_bits: Constant<Bits<32>>,
    col_bits: Constant<Bits<32>>,
    can_write: Signal<Local, Bit>,
    can_read: Signal<Local, Bit>,
    is_full: Signal<Local, Bit>,
    is_empty: Signal<Local, Bit>,
}

impl<const R: usize, const C: usize, const P: usize, const D: usize>
    SDRAMFIFOController<R, C, P, D>
{
    pub fn new(cas_delay: u32, timings: MemoryTimings) -> SDRAMFIFOController<R, C, P, D> {
        assert_eq!(P, R + C + 2);
        // mode register definitions
        // A2:A0 are the burst length, this design does not use burst transfers
        // so A2:A0 are 0
        // A3 is 0 because of burst type sequential
        // A6:A4 define the CAS latency in clocks.  We assume 3 clocks of latency.
        // The rest of the bits should all be zero
        // So the mode register is basically just CAS << 4
        let mode_register = cas_delay << 4;
        Self {
            sdram: Default::default(),
            write_enable: Default::default(),
            clock: Default::default(),
            cmd: Default::default(),
            encoder: Default::default(),
            data_in: Default::default(),
            write: Default::default(),
            full: Default::default(),
            data_out: Default::default(),
            read: Default::default(),
            empty: Default::default(),
            boot_delay: Constant::new((timings.t_boot() + 10).into()),
            t_rp: Constant::new((timings.t_rp()).into()),
            t_rfc: Constant::new((timings.t_rfc()).into()),
            t_refresh_max: Constant::new((timings.t_refresh_max() * 9 / 10).into()),
            t_rcd: Constant::new((timings.t_rcd()).into()),
            t_wr: Constant::new((timings.t_wr()).into()),
            max_transfer_size: Constant::new(64_usize.into()),
            mode_register: Constant::new(mode_register.into()),
            cas_delay: Constant::new(cas_delay.into()),
            state: Default::default(),
            write_pointer: Default::default(),
            read_pointer: Default::default(),
            delay_counter: Default::default(),
            refresh_counter: Default::default(),
            transfer_counter: Default::default(),
            active_bank: Default::default(),
            active_row: Default::default(),
            read_valid: Default::default(),
            fp: Default::default(),
            bp: Default::default(),
            read_bank: Default::default(),
            read_row: Default::default(),
            read_col: Default::default(),
            write_bank: Default::default(),
            write_row: Default::default(),
            write_col: Default::default(),
            row_bits: Constant::new(R.into()),
            col_bits: Constant::new(C.into()),
            can_write: Default::default(),
            can_read: Default::default(),
            is_full: Default::default(),
            is_empty: Default::default(),
        }
    }
}

impl<const R: usize, const C: usize, const P: usize, const D: usize> Logic
    for SDRAMFIFOController<R, C, P, D>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal logic
        self.state.clk.next = self.clock.val();
        self.write_pointer.clk.next = self.clock.val();
        self.read_pointer.clk.next = self.clock.val();
        self.fp.clock.next = self.clock.val();
        self.bp.clock.next = self.clock.val();
        self.delay_counter.clk.next = self.clock.val();
        self.refresh_counter.clk.next = self.clock.val();
        self.read_valid.clock.next = self.clock.val();
        self.transfer_counter.clk.next = self.clock.val();
        self.active_bank.clk.next = self.clock.val();
        self.active_row.clk.next = self.clock.val();
        // Connect the input and output to the fp and bp fifo
        self.fp.write.next = self.write.val();
        self.fp.data_in.next = self.data_in.val();
        self.full.next = self.fp.full.val();
        self.bp.read.next = self.read.val();
        self.data_out.next = self.bp.data_out.val();
        self.empty.next = self.bp.empty.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.write_pointer.d.next = self.write_pointer.q.val();
        self.read_pointer.d.next = self.read_pointer.q.val();
        self.delay_counter.d.next = self.delay_counter.q.val() + 1_usize;
        self.refresh_counter.d.next = self.refresh_counter.q.val() + 1_usize;
        self.cmd.next = SDRAMCommand::NOP;
        self.sdram.address.next = 0_usize.into();
        self.sdram.bank.next = 0_usize.into();
        self.write_enable.next = false;
        self.sdram.write_data.next = self.fp.data_out.val();
        self.bp.data_in.next = self.sdram.read_data.val();
        self.fp.read.next = false;
        self.read_valid.data_in.next = false;
        self.bp.write.next = self.read_valid.data_out.val();
        self.read_valid.delay.next = self.cas_delay.val();
        // Calculate the read and write addresses
        self.read_col.next = bit_cast::<13, C>(self.read_pointer.q.val().get_bits::<C>(0_usize));
        self.read_row.next = bit_cast::<13, R>(
            self.read_pointer
                .q
                .val()
                .get_bits::<R>(self.col_bits.val().index()),
        );
        self.read_bank.next = self
            .read_pointer
            .q
            .val()
            .get_bits::<2>(self.col_bits.val().index() + self.row_bits.val().index());
        self.write_col.next = bit_cast::<13, C>(self.write_pointer.q.val().get_bits::<C>(0_usize));
        self.write_row.next = bit_cast::<13, R>(
            self.write_pointer
                .q
                .val()
                .get_bits::<R>(self.col_bits.val().index()),
        );
        self.write_bank.next = self
            .write_pointer
            .q
            .val()
            .get_bits::<2>(self.col_bits.val().index() + self.row_bits.val().index());
        self.transfer_counter.d.next = self.transfer_counter.q.val();
        self.active_bank.d.next = self.active_bank.q.val();
        self.active_row.d.next = self.active_row.q.val();
        self.is_empty.next = self.write_pointer.q.val() == self.read_pointer.q.val();
        self.is_full.next = self.read_pointer.q.val() == (self.write_pointer.q.val() + 1_usize);
        self.can_write.next = (self.transfer_counter.q.val() < self.max_transfer_size.val())
            & (self.write_bank.val() == self.active_bank.q.val())
            & (self.write_row.val() == self.active_row.q.val())
            & (self.refresh_counter.q.val() < self.t_refresh_max.val())
            & (!self.is_full.val())
            & !self.fp.empty.val();
        self.can_read.next = (self.transfer_counter.q.val() < self.max_transfer_size.val())
            & (self.read_bank.val() == self.active_bank.q.val())
            & (self.read_row.val() == self.active_row.q.val())
            & (self.refresh_counter.q.val() < self.t_refresh_max.val())
            & (!self.is_empty.val())
            & !self.bp.almost_full.val();
        // State machine
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
                    if !self.is_empty.val() & !self.bp.almost_full.val() {
                        self.cmd.next = SDRAMCommand::Active;
                        self.sdram.bank.next = self.read_bank.val();
                        self.sdram.address.next = self.read_row.val();
                        self.state.d.next = State::ReadActivate;
                        self.active_row.d.next = self.read_row.val();
                        self.active_bank.d.next = self.read_bank.val();
                    } else if !self.is_full.val() & !self.fp.empty.val() {
                        self.cmd.next = SDRAMCommand::Active;
                        self.sdram.bank.next = self.write_bank.val();
                        self.sdram.address.next = self.write_row.val();
                        self.state.d.next = State::WriteActivate;
                        self.active_row.d.next = self.write_row.val();
                        self.active_bank.d.next = self.write_bank.val();
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
                if self.can_write.val() {
                    self.sdram.bank.next = self.write_bank.val();
                    self.sdram.address.next = self.write_col.val();
                    self.cmd.next = SDRAMCommand::Write;
                    self.write_enable.next = true;
                    self.transfer_counter.d.next = self.transfer_counter.q.val() + 1_usize;
                    self.fp.read.next = true;
                    self.write_pointer.d.next = self.write_pointer.q.val() + 1_usize;
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
                if self.can_read.val() {
                    self.sdram.bank.next = self.read_bank.val();
                    self.sdram.address.next = self.read_col.val();
                    self.cmd.next = SDRAMCommand::Read;
                    self.transfer_counter.d.next = self.transfer_counter.q.val() + 1_usize;
                    self.read_valid.data_in.next = true;
                    self.read_pointer.d.next = self.read_pointer.q.val() + 1_usize;
                } else {
                    self.delay_counter.d.next = 0_usize.into();
                    self.state.d.next = State::Recovery;
                }
            }
        }
        // Connect the command encoder
        self.encoder.cmd.next = self.cmd.val();
        self.sdram.we_not.next = self.encoder.we_not.val();
        self.sdram.cas_not.next = self.encoder.cas_not.val();
        self.sdram.ras_not.next = self.encoder.ras_not.val();
        self.sdram.cs_not.next = self.encoder.cs_not.val();
        self.sdram.clk.next = self.clock.val();
    }
}

#[derive(LogicBlock)]
struct TestSDRAMDevice {
    dram: SDRAMSimulator<16>,
    cntrl: SDRAMFIFOController<5, 5, 12, 16>,
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
    let timings = MemoryTimings::fast_boot_sim(125e6);
    let mut uut = TestSDRAMDevice {
        dram: SDRAMSimulator::new(timings),
        cntrl: SDRAMFIFOController::new(3, timings),
        clock: Default::default(),
    };
    uut.clock.connect();
    uut.cntrl.data_in.connect();
    uut.cntrl.write.connect();
    uut.cntrl.read.connect();
    uut.connect_all();
    uut
}

#[cfg(test)]
fn make_test_controller() -> SDRAMFIFOController<5, 5, 12, 16> {
    let timings = MemoryTimings::fast_boot_sim(125e6);
    let mut uut = SDRAMFIFOController::new(3, timings);
    uut.write.connect();
    uut.read.connect();
    uut.data_in.connect();
    uut.clock.connect();
    uut.sdram.link_connect_dest();
    uut.connect_all();
    uut
}

#[test]
fn test_sdram_controller_is_synthesizable() {
    let uut = make_test_controller();
    let vlog = generate_verilog(&uut);
    //println!("{}", vlog);
    yosys_validate("sdram_fifo_controller", &vlog).unwrap();
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
    sim.run_to_file(Box::new(uut), 12_000_000, "fifo_sdram_boot.vcd")
        .unwrap()
}

#[test]
fn test_unit_writes() {
    use rand::Rng;
    let uut = make_test_device();
    let mut sim = Simulation::new();
    let test_data = (0..2048)
        .map(|_| rand::thread_rng().gen::<u16>())
        .collect::<Vec<_>>();
    sim.add_clock(4000, |x: &mut Box<TestSDRAMDevice>| {
        x.clock.next = !x.clock.val()
    });
    let send = test_data.clone();
    let recv = test_data.clone();
    sim.add_testbench(move |mut sim: Sim<TestSDRAMDevice>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for data in &send {
            x.cntrl.data_in.next = (*data as u16).into();
            x.cntrl.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.cntrl.write.next = false;
            while x.cntrl.full.val() {
                wait_clock_cycle!(sim, clock, x);
            }
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<TestSDRAMDevice>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for data in &recv {
            while x.cntrl.empty.val() {
                wait_clock_cycle!(sim, clock, x);
            }
            sim_assert!(sim, x.cntrl.data_out.val() == (*data as u16), x);
            x.cntrl.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.cntrl.read.next = false;
        }
        sim_assert!(sim, !x.dram.test_error.val(), x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 80_000_000, "fifo_sdram_writes.vcd")
        .unwrap()
}
