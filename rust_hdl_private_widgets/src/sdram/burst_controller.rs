use crate::dff::DFF;
use crate::dff_setup;
use crate::prelude::DelayLine;
use crate::sdram::cmd::{SDRAMCommand, SDRAMCommandEncoder};
use crate::sdram::{OutputBuffer, SDRAMDriver};
use rust_hdl_private_core::prelude::*;

use super::timings::MemoryTimings;

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
    WritePrep,
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
    // The input interface does not allow flow control.  You must hook this up to a
    // FIFO on the consumer side to send data or risk data loss.  It is your
    // responsibility to ensure that you can provide L values on the input interface
    // on demand when you issue the WRITE command.
    pub data_in: Signal<In, Bits<D>>,
    pub data_strobe: Signal<Out, Bit>,
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
    // These are used to decouple the timing of the controller from the
    // outside world
    data_in_reg: DFF<Bits<D>>,
    data_strobe_reg: DFF<Bit>,
    data_out_reg: DFF<Bits<D>>,
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
            data_strobe: Default::default(),
            write_not_read: Default::default(),
            cmd_strobe: Default::default(),
            cmd_address: Default::default(),
            busy: Default::default(),
            data_out: Default::default(),
            data_valid: Default::default(),
            error: Default::default(),
            boot_delay: Constant::new((timings.t_boot() + 50).to_bits()),
            t_rp: Constant::new((timings.t_rp()).to_bits()),
            t_rfc: Constant::new((timings.t_rfc()).to_bits()),
            t_refresh_max: Constant::new((timings.t_refresh_max() * 7 / 10).to_bits()),
            t_rcd: Constant::new((timings.t_rcd()).to_bits()),
            t_wr: Constant::new((timings.t_wr()).to_bits()),
            max_transfer_size: Constant::new(L.to_bits()),
            mode_register: Constant::new(mode_register.to_bits()),
            /*
             * For a registered buffer, we need to add 2 cycles to the cas delay
             * - we add 1 on the send side because we add 1 on the send side and
             *   1 on the receive side
             */
            cas_delay: Constant::new(
                match buffer {
                    OutputBuffer::Wired => cas_delay + 1,
                    OutputBuffer::DelayOne => cas_delay + 2,
                    OutputBuffer::DelayTwo => cas_delay + 3,
                }
                .to_bits(),
            ),
            state: Default::default(),
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
            row_bits: Constant::new(R.to_bits()),
            col_bits: Constant::new(C.to_bits()),
            data_in_reg: Default::default(),
            read_pending: Default::default(),
            encode: Default::default(),
            refresh_needed: Default::default(),
            data_out_reg: Default::default(),
            data_strobe_reg: Default::default(),
        }
    }
}

impl<const R: usize, const C: usize, const L: u32, const D: usize> Logic
    for SDRAMBurstController<R, C, L, D>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal logic
        dff_setup!(
            self,
            clock,
            state,
            reg_address,
            reg_cmd_address,
            delay_counter,
            refresh_counter,
            transfer_counter,
            write_pending,
            read_pending,
            refresh_needed,
            data_in_reg,
            data_strobe_reg,
            data_out_reg
        );
        clock!(self, clock, read_valid);
        // Latch prevention
        self.delay_counter.d.next = self.delay_counter.q.val() + 1;
        self.refresh_counter.d.next = self.refresh_counter.q.val() + 1;
        self.cmd.next = SDRAMCommand::NOP;
        self.sdram.address.next = 0.into();
        self.sdram.bank.next = 0.into();
        // Insert registers to decouple the DRAM bus from the external bus
        self.data_out.next = self.data_out_reg.q.val();
        self.data_out_reg.d.next = self.sdram.read_data.val();
        self.data_valid.next = self.read_valid.data_out.val();
        self.sdram.write_enable.next = false;
        // Connect the DRAM to the staging register to decouple timing
        self.sdram.write_data.next = self.data_in_reg.q.val();
        self.data_strobe.next = self.data_strobe_reg.q.val();
        self.read_valid.data_in.next = false;
        self.read_valid.delay.next = self.cas_delay.val();
        // Calculate the read and write addresses
        self.addr_col.next = bit_cast::<13, C>(self.reg_address.q.val().get_bits::<C>(0));
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
        self.data_strobe_reg.d.next = false;
        match self.state.q.val() {
            State::Boot => {
                if self.delay_counter.q.val() == self.boot_delay.val() {
                    self.state.d.next = State::Precharge1;
                    self.cmd.next = SDRAMCommand::Precharge;
                    self.sdram.address.next = 0xFFF.into();
                    self.delay_counter.d.next = 0.into();
                }
            }
            State::Precharge1 => {
                if self.delay_counter.q.val() == self.t_rp.val() {
                    self.state.d.next = State::AutoRefresh1;
                    self.cmd.next = SDRAMCommand::AutoRefresh;
                    self.delay_counter.d.next = 0.into();
                }
            }
            State::AutoRefresh1 => {
                if self.delay_counter.q.val() == self.t_rfc.val() {
                    self.state.d.next = State::AutoRefresh2;
                    self.cmd.next = SDRAMCommand::AutoRefresh;
                    self.delay_counter.d.next = 0.into();
                }
            }
            State::AutoRefresh2 => {
                if self.delay_counter.q.val() == self.t_rfc.val() {
                    self.state.d.next = State::LoadModeRegister;
                    self.cmd.next = SDRAMCommand::LoadModeRegister;
                    self.sdram.address.next = self.mode_register.val();
                    self.delay_counter.d.next = 0.into();
                }
            }
            State::LoadModeRegister => {
                if self.delay_counter.q.val() == 4 {
                    self.state.d.next = State::Idle;
                }
            }
            State::Idle => {
                self.delay_counter.d.next = 0.into();
                self.transfer_counter.d.next = 0.into();
                if self.refresh_needed.q.val() {
                    // Refresh takes the highest priority
                    self.cmd.next = SDRAMCommand::AutoRefresh;
                    self.state.d.next = State::Refresh;
                    self.refresh_counter.d.next = 0.into();
                    self.refresh_needed.d.next = false;
                } else if self.read_pending.q.val() {
                    self.reg_address.d.next = self.reg_cmd_address.q.val();
                    self.state.d.next = State::IssueRead;
                } else if self.write_pending.q.val() {
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
                    self.transfer_counter.d.next = 0.into();
                }
            }
            State::WriteActivate => {
                self.sdram.write_enable.next = true;
                if self.delay_counter.q.val() == self.t_rcd.val() {
                    self.state.d.next = State::WritePrep;
                    self.transfer_counter.d.next = 0.into();
                    self.data_strobe_reg.d.next = true;
                }
            }
            State::WritePrep => {
                self.state.d.next = State::WriteCycle;
                self.data_strobe_reg.d.next = true;
                self.sdram.write_enable.next = true;
            }
            State::WriteCycle => {
                self.sdram.write_enable.next = true;
                if self.transfer_counter.q.val() < self.max_transfer_size.val() {
                    self.sdram.bank.next = self.addr_bank.val();
                    self.sdram.address.next = self.addr_col.val();
                    self.cmd.next = SDRAMCommand::Write;
                    self.transfer_counter.d.next = self.transfer_counter.q.val() + 1;
                    self.reg_address.d.next = self.reg_address.q.val() + 1;
                } else {
                    self.delay_counter.d.next = 0.into();
                    self.state.d.next = State::Recovery;
                }
                if self.transfer_counter.q.val() < self.max_transfer_size.val() - 2 {
                    self.data_strobe_reg.d.next = true;
                }
            }
            State::Recovery => {
                self.sdram.write_enable.next = true;
                if self.delay_counter.q.val() == self.t_wr.val() {
                    self.cmd.next = SDRAMCommand::Precharge;
                    // 13 bits is 0001_1111_1111_1111 0x1FFF or
                    self.sdram.address.next = 0x1FFF.into();
                    self.delay_counter.d.next = 0.into();
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
                    self.transfer_counter.d.next = self.transfer_counter.q.val() + 1;
                    self.read_valid.data_in.next = true;
                    self.reg_address.d.next = self.reg_address.q.val() + 1;
                } else {
                    self.delay_counter.d.next = 0.into();
                    self.state.d.next = State::Recovery;
                }
            }
            State::Error => {}
            _ => {
                self.state.d.next = State::Boot;
            }
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
        self.data_in_reg.d.next = self.data_in.val();
    }
}
