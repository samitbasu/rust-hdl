use crate::core::prelude::*;
use crate::hls::fifo::SyncFIFO;
use crate::widgets::dff::DFF;
use crate::widgets::prelude::{DelayLine, MemoryTimings, SynchronousFIFO};
use crate::widgets::sdram::cmd::{SDRAMCommand, SDRAMCommandEncoder};
use crate::widgets::sdram::{OutputBuffer, SDRAMDriver};

// Controller states...
#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Boot,
    Precharge1,
    AutoRefresh1,
    AutoRefresh2,
    LoadModeRegister,
    Idle,
    IssueRead,
    IssueWrite,
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
//  L - Burst size (< 32)
#[derive(LogicBlock)]
pub struct SDRAMBurstController<const R: usize, const C: usize, const L: u32, const D: usize> {
    pub clock: Signal<In, Clock>,
    pub sdram: SDRAMDriver<D>,
    // We have a couple of small FIFOs that are used to stage data for the writes and reads
    // We forward these interfaces to the user (this is the MIG pattern used in the Spartan6).
    // So to use this controller, you write L words to the input fifo and then trigger
    // the write command, or you trigger a read command and then consume words out of the
    // output fifo.
    pub data_in: Signal<In, Bits<D>>,
    pub data_write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    // The output interface does not allow flow control.  You must hook this up to a
    // FIFO on the consumer side to receive data or risk data loss.  It is your
    // responsibility to ensure that you can accept L values on the output interface
    // when you issue the READ command.
    pub data_out: Signal<Out, Bits<D>>,
    pub data_valid: Signal<Out, Bit>,
    // Command interface
    pub write_not_read: Signal<In, Bit>,
    pub cmd_strobe: Signal<In, Bit>,
    pub cmd_address: Signal<In, Bits<32>>,
    pub busy: Signal<Out, Bit>,
    pub error: Signal<Out, Bit>,
    cmd: Signal<Local, SDRAMCommand>,
    encode: SDRAMCommandEncoder,
    boot_delay: Constant<Bits<16>>,
    t_rp: Constant<Bits<16>>,
    t_rfc: Constant<Bits<16>>,
    t_refresh_max: Constant<Bits<16>>,
    t_rcd: Constant<Bits<16>>,
    t_wr: Constant<Bits<16>>,
    max_transfer_size: Constant<Bits<6>>,
    mode_register: Constant<Bits<13>>,
    cas_delay: Constant<Bits<3>>,
    state: DFF<State>,
    fp: SynchronousFIFO<Bits<D>, 5, 6, L>,
    reg_address: DFF<Bits<32>>,
    reg_cmd_address: DFF<Bits<32>>,
    delay_counter: DFF<Bits<16>>,
    refresh_counter: DFF<Bits<16>>,
    transfer_counter: DFF<Bits<6>>,
    read_valid: DelayLine<Bit, 8, 3>,
    addr_bank: Signal<Local, Bits<2>>,
    addr_row: Signal<Local, Bits<13>>,
    addr_col: Signal<Local, Bits<13>>,
    write_pending: DFF<Bit>,
    read_pending: DFF<Bit>,
    refresh_needed: DFF<Bit>,
    row_bits: Constant<Bits<32>>,
    col_bits: Constant<Bits<32>>,
}

impl<const R: usize, const C: usize, const L: u32, const D: usize>
    SDRAMBurstController<R, C, L, D>
{
    pub fn new(
        cas_delay: u32,
        timings: MemoryTimings,
        buffer: OutputBuffer,
    ) -> SDRAMBurstController<R, C, L, D> {
        assert!(L < 64);
        assert_eq!((1 << C) % L, 0);
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
            data_write: Default::default(),
            write_not_read: Default::default(),
            cmd_strobe: Default::default(),
            cmd_address: Default::default(),
            busy: Default::default(),
            data_out: Default::default(),
            data_valid: Default::default(),
            error: Default::default(),
            boot_delay: Constant::new((timings.t_boot() + 10).into()),
            t_rp: Constant::new((timings.t_rp()).into()),
            t_rfc: Constant::new((timings.t_rfc()).into()),
            t_refresh_max: Constant::new((timings.t_refresh_max() * 9 / 10).into()),
            t_rcd: Constant::new((timings.t_rcd()).into()),
            t_wr: Constant::new((timings.t_wr()).into()),
            max_transfer_size: Constant::new(L.into()),
            mode_register: Constant::new(mode_register.into()),
            /*
             * For a registered buffer, we need to add 2 cycles to the cas delay
             * - we add 1 on the send side because we add 1 on the send side and
             *   1 on the receive side
             */
            cas_delay: Constant::new(
                match buffer {
                    OutputBuffer::Wired => cas_delay,
                    OutputBuffer::DelayOne => cas_delay + 1,
                    OutputBuffer::DelayTwo => cas_delay + 2,
                }
                .into(),
            ),
            state: Default::default(),
            fp: Default::default(),
            reg_address: Default::default(),
            reg_cmd_address: Default::default(),
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
            read_pending: Default::default(),
            encode: Default::default(),
            refresh_needed: Default::default(),
            full: Default::default(),
        }
    }
}

impl<const R: usize, const C: usize, const L: u32, const D: usize> Logic
    for SDRAMBurstController<R, C, L, D>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal logic
        self.state.clk.next = self.clock.val();
        self.fp.clock.next = self.clock.val();
        self.reg_address.clk.next = self.clock.val();
        self.delay_counter.clk.next = self.clock.val();
        self.refresh_counter.clk.next = self.clock.val();
        self.read_valid.clock.next = self.clock.val();
        self.transfer_counter.clk.next = self.clock.val();
        self.write_pending.clk.next = self.clock.val();
        self.read_pending.clk.next = self.clock.val();
        self.refresh_needed.clk.next = self.clock.val();
        self.reg_cmd_address.clk.next = self.clock.val();
        self.reg_cmd_address.d.next = self.reg_cmd_address.q.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.delay_counter.d.next = self.delay_counter.q.val() + 1_usize;
        self.refresh_counter.d.next = self.refresh_counter.q.val() + 1_usize;
        self.cmd.next = SDRAMCommand::NOP;
        self.sdram.address.next = 0_usize.into();
        self.sdram.bank.next = 0_usize.into();
        // Connect the input FIFO to the write interface
        self.fp.data_in.next = self.data_in.val();
        self.fp.write.next = self.data_write.val();
        self.full.next = self.fp.full.val();
        // Connect the read bus directly to the DRAM
        self.data_out.next = self.sdram.read_data.val();
        self.data_valid.next = self.read_valid.data_out.val();
        self.reg_address.d.next = self.reg_address.q.val();
        self.write_pending.d.next = self.write_pending.q.val();
        self.read_pending.d.next = self.read_pending.q.val();
        self.refresh_needed.d.next = self.refresh_needed.q.val();
        self.sdram.write_enable.next = false;
        self.sdram.write_data.next = self.fp.data_out.val();
        self.read_valid.data_in.next = false;
        self.read_valid.delay.next = self.cas_delay.val();
        // Calculate the read and write addresses
        self.addr_col.next = bit_cast::<13, C>(self.reg_address.q.val().get_bits::<C>(0_usize));
        self.addr_row.next = bit_cast::<13, R>(
            self.reg_address
                .q
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
        self.fp.read.next = false;
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
                if self.refresh_needed.q.val() {
                    // Refresh takes the highest priority
                    self.cmd.next = SDRAMCommand::AutoRefresh;
                    self.state.d.next = State::Refresh;
                    self.refresh_counter.d.next = 0_usize.into();
                    self.refresh_needed.d.next = false;
                } else if self.read_pending.q.val() {
                    self.reg_address.d.next = self.reg_cmd_address.q.val();
                    self.state.d.next = State::IssueRead;
                } else if self.write_pending.q.val() & !self.fp.almost_empty.val() {
                    self.reg_address.d.next = self.reg_cmd_address.q.val();
                    self.state.d.next = State::IssueWrite;
                }
            }
            State::IssueRead => {
                self.cmd.next = SDRAMCommand::Active;
                self.sdram.bank.next = self.addr_bank.val();
                self.sdram.address.next = self.addr_row.val();
                self.state.d.next = State::ReadActivate;
                self.read_pending.d.next = false;
            }
            State::IssueWrite => {
                self.cmd.next = SDRAMCommand::Active;
                self.sdram.bank.next = self.addr_bank.val();
                self.sdram.address.next = self.addr_row.val();
                self.state.d.next = State::WriteActivate;
                self.write_pending.d.next = false;
                self.sdram.write_enable.next = true;
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
                self.sdram.write_enable.next = true;
                if self.delay_counter.q.val() == self.t_rcd.val() {
                    self.state.d.next = State::WriteCycle;
                    self.transfer_counter.d.next = 0_usize.into();
                }
            }
            State::WriteCycle => {
                self.sdram.write_enable.next = true;
                if self.transfer_counter.q.val() < self.max_transfer_size.val() {
                    self.sdram.bank.next = self.addr_bank.val();
                    self.sdram.address.next = self.addr_col.val();
                    self.cmd.next = SDRAMCommand::Write;
                    self.transfer_counter.d.next = self.transfer_counter.q.val() + 1_usize;
                    self.fp.read.next = true;
                    self.reg_address.d.next = self.reg_address.q.val() + 1_usize;
                } else {
                    self.delay_counter.d.next = 0_usize.into();
                    self.state.d.next = State::Recovery;
                }
            }
            State::Recovery => {
                self.sdram.write_enable.next = true;
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
            self.reg_cmd_address.d.next = self.cmd_address.val();
            if self.write_not_read.val() {
                self.write_pending.d.next = true;
            } else {
                self.read_pending.d.next = true;
            }
        }
        if self.refresh_counter.q.val() >= self.t_refresh_max.val() {
            self.refresh_needed.d.next = true;
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
