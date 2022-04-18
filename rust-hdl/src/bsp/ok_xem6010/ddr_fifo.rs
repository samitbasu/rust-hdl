use super::mcb_if::MCBInterface1GDDR2;
use super::mig::MemoryInterfaceGenerator;
use crate::bsp::ok_xem6010::mig::MIGInstruction;
use crate::core::prelude::*;
use crate::widgets::prelude::*;

#[derive(LogicState, Debug, Copy, Clone, PartialEq)]
pub enum DDRFIFOState {
    Booting,
    Idle,
    Read,
    Write,
    ReadComplete,
    WriteComplete,
    UpdateWriteAddress,
    Busy,
}

#[derive(LogicBlock, Default)]
pub struct DDRFIFO {
    // Clocks.  Raw sys clock must come
    // from the PLL at 100 MHz.  o_clock is
    // a rebuffered version used to clock the
    // interface.
    pub o_clock: Signal<Out, Clock>,
    pub raw_sys_clock: Signal<In, Clock>,
    // Reset signal
    pub reset: Signal<In, ResetN>,
    // Read interface
    pub read: Signal<In, Bit>,
    pub data_out: Signal<Out, Bits<32>>,
    pub empty: Signal<Out, Bit>,
    pub almost_empty: Signal<Out, Bit>,
    pub read_clock: Signal<In, Clock>,
    // Write interface
    pub write: Signal<In, Bit>,
    pub data_in: Signal<In, Bits<32>>,
    pub almost_full: Signal<Out, Bit>,
    pub full: Signal<Out, Bit>,
    pub write_clock: Signal<In, Clock>,
    // DRAM interface
    pub mcb: MCBInterface1GDDR2,
    // Internal MIG
    mig: MemoryInterfaceGenerator,
    write_address: DFF<Bits<27>>,
    read_address: DFF<Bits<27>>,
    state: DFF<DDRFIFOState>,
    have_data: Signal<Local, Bit>,
    // Front porch FIFO
    front_porch: AsynchronousFIFO<Bits<32>, 10, 11, 32>,
    // Back porch FIFO
    back_porch: AsynchronousFIFO<Bits<32>, 8, 9, 32>,
    // Will accept data from front porch FIFO
    will_transfer_in: Signal<Local, Bit>,
    // Will push data to the back porch FIFO
    will_transfer_out: Signal<Local, Bit>,
    // transfer counter
    transfer_in_count: DFF<Bits<7>>,
    transfer_out_count: DFF<Bits<7>>,
    // Status byte
    pub status: Signal<Out, Bits<8>>,
    mig_clock: Signal<Local, Clock>,
    mig_reset: Signal<Local, ResetN>,
}

impl Logic for DDRFIFO {
    #[hdl_gen]
    fn update(&mut self) {
        // Link the mcb interface
        MCBInterface1GDDR2::link(&mut self.mcb, &mut self.mig.mcb);
        // Forward the raw clock
        self.mig.raw_sys_clk.next = self.raw_sys_clock.val();
        self.mig_clock.next = self.mig.clk_out.val();
        self.mig_reset.next = self.mig.reset_out.val();
        // Update the output clock with the generated (buffered) clock
        self.o_clock.next = self.mig.clk_out.val();
        // Connect the flops and the interfaces to that buffered clock
        self.mig.p0_rd.clock.next = self.mig.clk_out.val();
        self.mig.p0_wr.clock.next = self.mig.clk_out.val();
        self.mig.p0_cmd.clock.next = self.mig.clk_out.val();
        self.front_porch.read_clock.next = self.mig.clk_out.val();
        self.front_porch.read_reset.next = self.mig.reset_out.val();
        self.back_porch.write_clock.next = self.mig.clk_out.val();
        self.back_porch.write_reset.next = self.mig.reset_out.val();
        dff_setup!(
            self,
            mig_clock,
            mig_reset,
            write_address,
            read_address,
            state,
            transfer_in_count,
            transfer_out_count
        );
        // Connect the data signals from the front and back porch
        // FIFOs to the MIG FIFOs
        self.mig.p0_wr.data.next.data = self.front_porch.data_out.val();
        self.mig.p0_wr.data.next.mask = 0_usize.into();
        self.back_porch.data_in.next = self.mig.p0_rd.data.val();
        // Connect the front porch fifo to our published
        // interfaces
        self.front_porch.data_in.next = self.data_in.val();
        self.front_porch.write.next = self.write.val();
        self.almost_full.next = self.front_porch.almost_full.val();
        self.full.next = self.front_porch.full.val();
        self.front_porch.write_clock.next = self.write_clock.val();
        self.front_porch.write_reset.next = self.mig_reset.val();
        // Connect the back porch fifo to our published
        // interface
        self.data_out.next = self.back_porch.data_out.val();
        self.back_porch.read.next = self.read.val();
        self.almost_empty.next = self.back_porch.almost_empty.val();
        self.empty.next = self.back_porch.empty.val();
        self.back_porch.read_clock.next = self.read_clock.val();
        self.back_porch.read_reset.next = self.reset.val();
        // By default, do nothing.
        self.mig.p0_cmd.cmd.next.instruction = MIGInstruction::Refresh;
        self.mig.p0_cmd.cmd.next.byte_address = 0_usize.into();
        self.mig.p0_cmd.cmd.next.burst_len = 31_usize.into(); // Always work with 32 word packets
        self.mig.p0_cmd.enable.next = false;
        // The DDR FIFO contains data if the write address is not equal to the
        // read address.  NOTE! There should be some protection for the DDR FIFO
        // filling up.  TODO - Add DDR overrun protection.
        self.have_data.next = self.write_address.q.val() != self.read_address.q.val();
        // Decide when we will transfer in
        self.will_transfer_in.next = self.transfer_in_count.q.val().any()
            & !self.mig.p0_wr.full.val()
            & !self.front_porch.empty.val();
        self.transfer_in_count.d.next =
            self.transfer_in_count.q.val() - bit_cast::<7, 1>(self.will_transfer_in.val().into());
        self.mig.p0_wr.enable.next = self.will_transfer_in.val();
        self.front_porch.read.next = self.will_transfer_in.val();
        // Decide when we will transfer out
        self.will_transfer_out.next = self.transfer_out_count.q.val().any()
            & !self.mig.p0_rd.empty.val()
            & !self.back_porch.full.val();
        self.transfer_out_count.d.next =
            self.transfer_out_count.q.val() - bit_cast::<7, 1>(self.will_transfer_out.val().into());
        self.mig.p0_rd.enable.next = self.will_transfer_out.val();
        self.back_porch.write.next = self.will_transfer_out.val();
        match self.state.q.val() {
            DDRFIFOState::Booting => {
                if self.mig.calib_done.val() {
                    self.state.d.next = DDRFIFOState::Busy;
                }
            }
            DDRFIFOState::Idle => {
                if self.have_data.val() & !self.back_porch.almost_full.val() {
                    self.mig.p0_cmd.cmd.next.instruction = MIGInstruction::Read;
                    self.mig.p0_cmd.cmd.next.byte_address =
                        bit_cast::<30, 27>(self.read_address.q.val());
                    self.mig.p0_cmd.enable.next = true;
                    self.transfer_out_count.d.next = 32_usize.into();
                    self.state.d.next = DDRFIFOState::Read;
                } else if !self.front_porch.almost_empty.val() {
                    self.transfer_in_count.d.next = 32_usize.into();
                    self.state.d.next = DDRFIFOState::Write;
                }
            }
            DDRFIFOState::Read => {
                if !self.transfer_out_count.q.val().any() {
                    self.state.d.next = DDRFIFOState::ReadComplete;
                }
            }
            DDRFIFOState::ReadComplete => {
                self.read_address.d.next = self.read_address.q.val() + 128_usize;
                self.state.d.next = DDRFIFOState::Busy;
            }
            DDRFIFOState::Write => {
                if !self.transfer_in_count.q.val().any() {
                    self.state.d.next = DDRFIFOState::WriteComplete;
                }
            }
            DDRFIFOState::WriteComplete => {
                self.mig.p0_cmd.cmd.next.instruction = MIGInstruction::Write;
                self.mig.p0_cmd.cmd.next.byte_address =
                    bit_cast::<30, 27>(self.write_address.q.val());
                self.mig.p0_cmd.enable.next = true;
                self.state.d.next = DDRFIFOState::UpdateWriteAddress;
            }
            DDRFIFOState::UpdateWriteAddress => {
                self.write_address.d.next = self.write_address.q.val() + 128_usize;
                self.state.d.next = DDRFIFOState::Busy;
            }
            DDRFIFOState::Busy => {
                if !self.mig.p0_cmd.full.val() {
                    self.state.d.next = DDRFIFOState::Idle;
                }
            }
        }
        // Wire up the reset
        self.mig.reset.next = self.reset.val();
        // Set the status byte
        self.status.next = bit_cast::<8, 1>(self.mig.p0_wr.error.val().into())
            | (bit_cast::<8, 1>(self.mig.p0_wr.underrun.val().into()) << 1_usize)
            | (bit_cast::<8, 1>(self.mig.p0_cmd.full.val().into()) << 2_usize)
            | (bit_cast::<8, 1>(self.mig.p0_rd.error.val().into()) << 3_usize)
            | (bit_cast::<8, 1>(self.mig.p0_rd.overflow.val().into()) << 4_usize)
            | (bit_cast::<8, 1>(self.have_data.val().into()) << 5_usize);
    }
}

#[test]
fn test_ddr_fifo_gen() {
    let ddr = DDRFIFO::default();
    let _vlog = generate_verilog_unchecked(&ddr);
}
