use rust_hdl_core::prelude::*;

use crate::dff::DFF;
use crate::fifo_if::{FIFOReadIF, FIFOWriteIF};
use crate::ram::{RAMRead, RAMWrite};
use rust_hdl_core::bits::bit_cast;

// The read side of the circuitry for the FIFO.  Manages the read
// address
#[derive(LogicBlock)]
pub struct FIFOReadLogic<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    pub clock: Signal<In, Clock>,
    pub sig: FIFOReadIF<D>,
    pub write_address_delayed: Signal<In, Bits<NP1>>,
    pub ram_read: RAMRead<D, N>,
    pub read_address_out: Signal<Out, Bits<NP1>>,
    read_address: DFF<Bits<NP1>>,
    is_empty: Signal<Local, Bit>,
    fill_level: Signal<Local, Bits<NP1>>,
    dff_underflow: DFF<Bit>,
    fifo_address_mask: Constant<Bits<NP1>>,
    fifo_size: Constant<Bits<NP1>>,
    block_size: Constant<Bits<NP1>>,
}

impl<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Logic
    for FIFOReadLogic<D, N, NP1, BLOCK_SIZE>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clocks.
        self.read_address.clk.next = self.clock.val();
        self.dff_underflow.clk.next = self.clock.val();
        self.ram_read.clock.next = self.clock.val();
        // Compute the is empty flag
        self.is_empty.next = (self.read_address.q.val() == self.write_address_delayed.val()).into();
        // Estimate the fill level
        self.fill_level.next = ((self.write_address_delayed.val() & self.fifo_address_mask.val())
            + self.fifo_size.val()
            - (self.read_address.q.val() & self.fifo_address_mask.val()))
            & self.fifo_address_mask.val();
        // Compute the almost empty signal
        self.sig.almost_empty.next = (self.fill_level.val() < self.block_size.val()).into();
        // Propagate the empty signal.
        self.sig.empty.next = self.is_empty.val();
        // Set the RAM read address by masking off the lower N bits of the pointer.
        self.ram_read.address.next =
            bit_cast::<N, NP1>(self.read_address.q.val() & self.fifo_address_mask.val());
        // Forward the output of the RAM read to the FIFO interface.
        self.sig.data_out.next = self.ram_read.data.val();
        // Assign the read advance based on the outside request
        // and our availability to read.
        if self.sig.read.val() & !self.is_empty.val() {
            self.read_address.d.next = self.read_address.q.val() + 1_u32;
            // We "forward" by a cycle so that we don't loose a cycle waiting for the
            // update to propagate through the flip flop.
            self.ram_read.address.next = bit_cast::<N, NP1>(
                (self.read_address.q.val() + 1_u32) & self.fifo_address_mask.val(),
            );
        } else {
            self.read_address.d.next = self.read_address.q.val();
        }
        self.dff_underflow.d.next =
            self.dff_underflow.q.val() | (self.is_empty.val() & self.sig.read.val());
        self.sig.underflow.next = self.dff_underflow.q.val();
        self.read_address_out.next = self.read_address.q.val();
    }
}

impl<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Default
    for FIFOReadLogic<D, N, NP1, BLOCK_SIZE>
{
    fn default() -> Self {
        Self {
            sig: Default::default(),
            clock: Default::default(),
            write_address_delayed: Default::default(),
            ram_read: Default::default(),
            read_address_out: Default::default(),
            read_address: Default::default(),
            is_empty: Default::default(),
            fill_level: Default::default(),
            dff_underflow: Default::default(),
            fifo_address_mask: Constant::new(((1_u32 << (N)) - 1).into()),
            fifo_size: Constant::new(Bits::<N>::count().into()),
            block_size: Constant::new(BLOCK_SIZE.into()),
        }
    }
}

#[test]
fn fifo_read_is_synthesizable() {
    rust_hdl_synth::top_wrap!(FIFOReadLogic<Bits<8>, 8, 9, 4>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.write_address_delayed.connect();
    dev.uut.ram_read.data.connect();
    dev.uut.clock.connect();
    dev.uut.sig.read.connect();
    dev.connect_all();
    rust_hdl_synth::yosys_validate("fifo_read", &generate_verilog(&dev)).unwrap();
}

#[derive(LogicBlock)]
pub struct FIFOWriteLogic<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    pub sig: FIFOWriteIF<D>,
    pub clock: Signal<In, Clock>,
    pub ram_write: RAMWrite<D, N>,
    pub read_address: Signal<In, Bits<NP1>>,
    pub write_address_delayed: Signal<Out, Bits<NP1>>,
    write_address: DFF<Bits<NP1>>,
    dff_write_address_delay: DFF<Bits<NP1>>,
    is_empty: Signal<Local, Bit>,
    is_full: Signal<Local, Bit>,
    fill_level: Signal<Local, Bits<NP1>>,
    dff_overflow: DFF<Bit>,
    fifo_address_mask: Constant<Bits<NP1>>,
    fifo_size: Constant<Bits<NP1>>,
    almost_full_level: Constant<Bits<NP1>>,
}

impl<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Default
    for FIFOWriteLogic<D, N, NP1, BLOCK_SIZE>
{
    fn default() -> Self {
        assert_eq!(N + 1, NP1);
        assert!(NP1 < 32);
        Self {
            sig: Default::default(),
            clock: Default::default(),
            ram_write: Default::default(),
            read_address: Default::default(),
            write_address: Default::default(),
            write_address_delayed: Default::default(),
            dff_write_address_delay: Default::default(),
            is_empty: Default::default(),
            is_full: Default::default(),
            fill_level: Default::default(),
            dff_overflow: Default::default(),
            fifo_address_mask: Constant::new(((1_u32 << (N)) - 1).into()),
            fifo_size: Constant::new(Bits::<N>::count().into()),
            almost_full_level: Constant::new((Bits::<N>::count() - BLOCK_SIZE as usize).into()),
        }
    }
}

impl<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Logic
    for FIFOWriteLogic<D, N, NP1, BLOCK_SIZE>
{
    #[hdl_gen]
    fn update(&mut self) {
        self.dff_overflow.clk.next = self.clock.val();
        self.write_address.clk.next = self.clock.val();
        self.dff_write_address_delay.clk.next = self.clock.val();
        self.ram_write.clock.next = self.clock.val();
        // We need a 1 cycle delay on the write address
        // This ensures we do not try to read a data element on the same
        // cycle it is written.
        self.dff_write_address_delay.d.next = self.write_address.q.val();
        self.write_address_delayed.next = self.dff_write_address_delay.q.val();
        // Default to not writing
        self.ram_write.enable.next = false.into();
        // Calculate the empty field - this is used to determine if we
        // can read
        self.is_empty.next =
            (self.read_address.val() == self.dff_write_address_delay.q.val()).into();
        // Calculate the is full field.  If the FIFO is not empty, and
        // the lower N bits of the addresses agree, the FIFO is full
        self.is_full.next = !self.is_empty.val()
            & ((self.read_address.val() & self.fifo_address_mask.val())
                == (self.write_address.q.val() & self.fifo_address_mask.val()));
        // Compute the fill level - we add N first, since we are subtracting.  And
        // we mask out the lower N bits, since we are ignoring the wrap levels.
        // Note that if the FIFO is empty, this calculation will give the wrong
        // answer, so we need to check the is_empty flag (which uses all N+1 bits).
        self.fill_level.next = ((self.dff_write_address_delay.q.val()
            & self.fifo_address_mask.val())
            + self.fifo_size.val()
            - (self.read_address.val() & self.fifo_address_mask.val()))
            & self.fifo_address_mask.val();
        if self.is_full.val() {
            self.fill_level.next = self.fifo_size.val().into();
        }
        self.sig.almost_full.next = (self.fill_level.val() >= self.almost_full_level.val()).into();
        self.sig.full.next = self.is_full.val();
        // Connect our write address register to the RAM write address
        self.ram_write.address.next =
            bit_cast::<N, NP1>(self.write_address.q.val() & self.fifo_address_mask.val());
        self.ram_write.data.next = self.sig.data_in.val();
        // Assign the enable for the write based on the outside
        // request and our availability to write
        if self.sig.write.val() & !self.is_full.val() {
            self.write_address.d.next = self.write_address.q.val() + 1_u32;
            self.ram_write.enable.next = true;
        } else {
            self.write_address.d.next = self.write_address.q.val();
            self.ram_write.enable.next = false;
        }
        // Compute the overflow signal - it is latched
        self.dff_overflow.d.next =
            self.dff_overflow.q.val() | (self.is_full.val() & self.sig.write.val());
        self.sig.overflow.next = self.dff_overflow.q.val();
    }
}

#[test]
fn fifo_write_is_synthesizable() {
    rust_hdl_synth::top_wrap!(FIFOWriteLogic<Bits<8>, 8, 9, 4>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.read_address.connect();
    dev.uut.clock.connect();
    dev.uut.sig.data_in.connect();
    dev.uut.sig.write.connect();
    dev.connect_all();
    rust_hdl_synth::yosys_validate("fifo_write", &generate_verilog(&dev)).unwrap();
}
