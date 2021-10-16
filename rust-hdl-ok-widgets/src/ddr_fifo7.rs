use crate::ddr_fifo::DDRFIFOState;
use crate::mcb_if::MCBInterface4GDDR3;
use crate::mig7::MemoryInterfaceGenerator7Series;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::cross_fifo::CrossWidenFIFO;
use rust_hdl_widgets::dff::DFF;
use rust_hdl_widgets::prelude::CrossNarrowFIFO;

#[derive(LogicState, Debug, Copy, Clone, PartialEq)]
pub enum DDR7FIFOState {
    Booting,
    Idle,
    Read,
    Write,
}

#[derive(LogicBlock, Default)]
pub struct DDR7FIFO<const N: usize> {
    // Reset - required
    pub reset: Signal<In, Bit>,
    // System clock
    pub sys_clock_p: Signal<In, Clock>,
    pub sys_clock_n: Signal<In, Clock>,
    // Interface to DDR3
    pub mcb: MCBInterface4GDDR3,
    // The write interface
    pub data_in: Signal<In, Bits<N>>,
    pub full: Signal<Out, Bit>,
    pub write: Signal<In, Bit>,
    pub write_clock: Signal<In, Clock>,
    // The read interface
    pub data_out: Signal<Out, Bits<N>>,
    pub empty: Signal<Out, Bit>,
    pub read: Signal<In, Bit>,
    pub read_clock: Signal<In, Clock>,
    // The mig
    mig: MemoryInterfaceGenerator7Series,
    // Front end fifo
    front_porch: CrossWidenFIFO<{ N }, 10, 11, 128, 5, 6>,
    // The write address of the FIFO
    write_address: DFF<Bits<28>>,
    // The read address of the FIFO
    read_address: DFF<Bits<28>>,
    // Back end fifo
    back_porch: CrossNarrowFIFO<128, 8, 9, { N }, 10, 11>,
    // Current state
    state: DFF<DDR7FIFOState>,
    // Is DDR FIFO empty
    is_empty: Signal<Local, Bit>,
    pub status: Signal<Out, Bits<8>>,
}

impl<const N: usize> Logic for DDR7FIFO<N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Link the mcb interface
        self.mcb.link(&mut self.mig.mcb);
        // Forward the raw clocks
        self.mig.raw_pos_clock.next = self.sys_clock_p.val();
        self.mig.raw_neg_clock.next = self.sys_clock_n.val();
        // Connect the write interface to the front porch
        self.front_porch.write_clock.next = self.write_clock.val();
        self.front_porch.data_in.next = self.data_in.val();
        self.front_porch.write.next = self.write.val();
        self.full.next = self.front_porch.full.val();
        // Connect the read interface to the back porch
        self.back_porch.read_clock.next = self.read_clock.val();
        self.data_out.next = self.back_porch.data_out.val();
        self.back_porch.read.next = self.read.val();
        self.empty.next = self.back_porch.empty.val();
        // Connect the front porch to the MIG
        self.mig.write_data_in.next = self.front_porch.data_out.val();
        self.front_porch.read_clock.next = self.mig.clock.val();
        self.front_porch.read.next = false;
        // Connect the back porch to the MIG
        self.back_porch.data_in.next = self.mig.read_data_out.val();
        self.back_porch.write_clock.next = self.mig.clock.val();
        self.back_porch.write.next = false;
        // Clock the flops
        self.write_address.clk.next = self.mig.clock.val();
        self.read_address.clk.next = self.mig.clock.val();
        self.state.clk.next = self.mig.clock.val();
        // Latch prevention for the flops
        self.write_address.d.next = self.write_address.q.val();
        self.read_address.d.next = self.read_address.q.val();
        self.state.d.next = self.state.q.val();
        // Set default inputs for the mig
        self.mig.address.next = 0_u32.into();
        self.mig.enable.next = false;
        self.mig.write_enable.next = false;
        self.mig.reset.next = self.reset.val();
        self.mig.write_data_mask.next = 0_u32.into();
        self.mig.write_data_end.next = false;
        self.mig.command.next = 0_u8.into();
        // Compute the empty flag
        self.is_empty.next = self.read_address.q.val() == self.write_address.q.val();
        // State machine update
        match self.state.q.val() {
            DDR7FIFOState::Booting => {
                if self.mig.calib_done.val() & !self.reset.val() {
                    self.state.d.next = DDR7FIFOState::Idle;
                }
            }
            DDR7FIFOState::Idle => {
                if !self.is_empty.val() & !self.back_porch.full.val() & self.mig.ready.val() {
                    // We can read...
                    self.mig.command.next = 1_u8.into();
                    self.mig.enable.next = true;
                    self.mig.address.next = bit_cast::<29, 28>(self.read_address.q.val());
                    self.state.d.next = DDR7FIFOState::Read;
                } else if !self.front_porch.empty.val()
                    & self.mig.ready.val()
                    & self.mig.write_fifo_not_full.val()
                {
                    self.mig.write_enable.next = true;
                    self.mig.write_data_end.next = true;
                    self.front_porch.read.next = true;
                    self.state.d.next = DDR7FIFOState::Write;
                }
            }
            DDR7FIFOState::Write => {
                self.mig.command.next = 0_u8.into(); // Write command
                self.mig.enable.next = true;
                self.mig.address.next = bit_cast::<29, 28>(self.write_address.q.val());
                if self.mig.ready.val() {
                    self.write_address.d.next = self.write_address.q.val() + 8_u32; // This is the number of 16 bit words
                    self.state.d.next = DDR7FIFOState::Idle;
                }
            }
            DDR7FIFOState::Read => {
                self.back_porch.write.next = self.mig.read_data_valid.val();
                if self.mig.read_data_valid.val() {
                    self.read_address.d.next = self.read_address.q.val() + 8_u32;
                    self.state.d.next = DDR7FIFOState::Idle;
                }
            }
        }
        self.status.next = bit_cast::<8, 1>(self.front_porch.empty.val().into())
            | (bit_cast::<8, 1>(self.front_porch.full.val().into()) << 1_usize)
            | (bit_cast::<8, 1>(self.back_porch.empty.val().into()) << 2_usize)
            | (bit_cast::<8, 1>(self.back_porch.full.val().into()) << 3_usize)
            | (bit_cast::<8, 1>(self.is_empty.val().into()) << 4_usize)
            | (bit_cast::<8, 1>((self.state.q.val() == DDR7FIFOState::Idle).into()) << 5_usize)
            | (bit_cast::<8, 1>(self.mig.ready.val().into()) << 6_usize);
    }
}

#[test]
fn test_ddr7_fifo_gen() {
    use rust_hdl_synth::TopWrap;
    let mut ddr = TopWrap::new(DDR7FIFO::<32>::default());
    ddr.uut.sys_clock_n.connect();
    ddr.uut.sys_clock_p.connect();
    ddr.uut.mcb.link_connect_dest();
    ddr.uut.data_in.connect();
    ddr.uut.write.connect();
    ddr.uut.read.connect();
    ddr.uut.write_clock.connect();
    ddr.uut.read_clock.connect();
    ddr.uut.reset.connect();
    ddr.connect_all();
    yosys_validate("ddr7", &generate_verilog(&ddr));
}
