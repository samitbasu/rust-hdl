use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;

use crate::dff::DFF;
use crate::fifo_if::{FIFOReadIF, FIFOWriteIF};
use crate::ram::{RAMRead, RAMWrite, RAM};
use crate::sync_fifo::MHz1;

// The read side of the circuitry for the FIFO.  Manages the read
// address
#[derive(LogicBlock)]
pub struct FIFOReadLogic<
    D: Synth,
    T: Domain,
    const N: usize,
    const NP1: usize,
    const BlockSize: u32,
> {
    pub clock: Signal<In, Clock, T>,
    pub sig: FIFOReadIF<D, T>,
    pub write_address_delayed: Signal<In, Bits<NP1>, T>,
    pub ram_read: RAMRead<Bits<N>, D, T>,
    pub read_address_out: Signal<Out, Bits<NP1>, T>,
    read_address: DFF<Bits<NP1>, T>,
    is_empty: Signal<Local, Bit, T>,
    fill_level: Signal<Local, Bits<NP1>, T>,
    dff_underflow: DFF<Bit, T>,
    fifo_address_mask: Constant<Bits<NP1>>,
    fifo_size: Constant<Bits<NP1>>,
    block_size: Constant<Bits<NP1>>,
}

impl<D: Synth, T: Domain, const N: usize, const NP1: usize, const Blocksize: u32> Logic
    for FIFOReadLogic<D, T, N, NP1, Blocksize>
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
            tagged_bit_cast::<_, N, NP1>(self.read_address.q.val() & self.fifo_address_mask.val());
        // Forward the output of the RAM read to the FIFO interface.
        self.sig.data_out.next = self.ram_read.data.val();
        // Assign the read advance based on the outside request
        // and our availability to read.
        if (self.sig.read.val() & !self.is_empty.val()).any() {
            self.read_address.d.next = self.read_address.q.val() + 1_u32;
            // We "forward" by a cycle so that we don't loose a cycle waiting for the
            // update to propagate through the flip flop.
            self.ram_read.address.next = tagged_bit_cast::<_, N, NP1>(
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

impl<D: Synth, T: Domain, const N: usize, const NP1: usize, const Blocksize: u32> Default
    for FIFOReadLogic<D, T, N, NP1, Blocksize>
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
            block_size: Constant::new(Blocksize.into()),
        }
    }
}

#[test]
fn fifo_read_is_synthesizable() {
    rust_hdl_synth::top_wrap!(FIFOReadLogic<Bits<8>, MHz1, 8, 9, 4>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.write_address_delayed.connect();
    dev.uut.ram_read.data.connect();
    dev.uut.clock.connect();
    dev.uut.sig.read.connect();
    dev.connect_all();
    yosys_validate("fifo_read", &generate_verilog(&dev)).unwrap();
}

#[derive(LogicBlock)]
pub struct FIFOWriteLogic<
    D: Synth,
    T: Domain,
    const N: usize,
    const NP1: usize,
    const BlockSize: u32,
> {
    pub sig: FIFOWriteIF<D, T>,
    pub clock: Signal<In, Clock, T>,
    pub ram_write: RAMWrite<Bits<N>, D, T>,
    pub read_address: Signal<In, Bits<NP1>, T>,
    pub write_address_delayed: Signal<Out, Bits<NP1>, T>,
    write_address: DFF<Bits<NP1>, T>,
    dff_write_address_delay: DFF<Bits<NP1>, T>,
    is_empty: Signal<Local, Bit, T>,
    is_full: Signal<Local, Bit, T>,
    fill_level: Signal<Local, Bits<NP1>, T>,
    dff_overflow: DFF<Bit, T>,
    fifo_address_mask: Constant<Bits<NP1>>,
    fifo_size: Constant<Bits<NP1>>,
    almost_full_level: Constant<Bits<NP1>>,
}

impl<D: Synth, T: Domain, const N: usize, const NP1: usize, const BlockSize: u32> Default
    for FIFOWriteLogic<D, T, N, NP1, BlockSize>
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
            almost_full_level: Constant::new((Bits::<N>::count() - BlockSize as usize).into()),
        }
    }
}

impl<D: Synth, T: Domain, const N: usize, const NP1: usize, const BlockSize: u32> Logic
    for FIFOWriteLogic<D, T, N, NP1, BlockSize>
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
        if self.is_full.val().any() {
            self.fill_level.next = self.fifo_size.val().into();
        }
        self.sig.almost_full.next = (self.fill_level.val() >= self.almost_full_level.val()).into();
        self.sig.full.next = self.is_full.val();
        // Connect our write address register to the RAM write address
        self.ram_write.address.next =
            tagged_bit_cast::<_, N, NP1>(self.write_address.q.val() & self.fifo_address_mask.val());
        self.ram_write.data.next = self.sig.data_in.val();
        // Assign the enable for the write based on the outside
        // request and our availability to write
        if (self.sig.write.val() & !self.is_full.val()).any() {
            self.write_address.d.next = self.write_address.q.val() + 1_u32;
            self.ram_write.enable.next = true.into();
        } else {
            self.write_address.d.next = self.write_address.q.val();
            self.ram_write.enable.next = false.into();
        }
        // Compute the overflow signal - it is latched
        self.dff_overflow.d.next =
            self.dff_overflow.q.val() | (self.is_full.val() & self.sig.write.val());
        self.sig.overflow.next = self.dff_overflow.q.val();
    }
}

#[test]
fn fifo_write_is_synthesizable() {
    rust_hdl_synth::top_wrap!(FIFOWriteLogic<Bits<8>, MHz1, 8, 9, 4>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.read_address.connect();
    dev.uut.clock.connect();
    dev.uut.sig.data_in.connect();
    dev.uut.sig.write.connect();
    dev.connect_all();
    yosys_validate("fifo_write", &generate_verilog(&dev)).unwrap();
}

#[derive(LogicBlock)]
pub struct SynchronousFIFO<
    D: Synth,
    T: Domain,
    const N: usize,
    const NP1: usize,
    const BlockSize: u32,
> {
    pub clock: Signal<In, Clock, T>,
    pub read_if: FIFOReadIF<D, T>,
    pub write_if: FIFOWriteIF<D, T>,
    // Internal RAM
    ram: RAM<Bits<N>, D, T, T>,
    // Read logic
    read: FIFOReadLogic<D, T, N, NP1, BlockSize>,
    // write logic
    write: FIFOWriteLogic<D, T, N, NP1, BlockSize>,
}

impl<D: Synth, T: Domain, const N: usize, const NP1: usize, const BlockSize: u32> Default
    for SynchronousFIFO<D, T, N, NP1, BlockSize>
{
    fn default() -> Self {
        Self {
            clock: Default::default(),
            read_if: Default::default(),
            write_if: Default::default(),
            ram: RAM::new(Default::default()),
            read: Default::default(),
            write: Default::default(),
        }
    }
}

impl<D: Synth, T: Domain, const N: usize, const NP1: usize, const BlockSize: u32> Logic
    for SynchronousFIFO<D, T, N, NP1, BlockSize>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect up the read interface
        self.read.clock.next = self.clock.val();
        self.read.sig.read.next = self.read_if.read.val();
        self.read_if.empty.next = self.read.sig.empty.val();
        self.read_if.almost_empty.next = self.read.sig.almost_empty.val();
        self.read_if.data_out.next = self.read.sig.data_out.val();
        self.read_if.underflow.next = self.read.sig.underflow.val();
        // Connect up the write interface
        self.write.clock.next = self.clock.val();
        self.write_if.overflow.next = self.write.sig.overflow.val();
        self.write_if.almost_full.next = self.write.sig.almost_full.val();
        self.write_if.full.next = self.write.sig.full.val();
        self.write.sig.write.next = self.write_if.write.val();
        self.write.sig.data_in.next = self.write_if.data_in.val();
        // Connect the RAM to the two blocks
        self.ram.write.clock.next = self.clock.val();
        self.ram.write.enable.next = self.write.ram_write.enable.val();
        self.ram.write.address.next = self.write.ram_write.address.val();
        self.ram.write.data.next = self.write.ram_write.data.val();
        self.ram.read.clock.next = self.clock.val();
        self.ram.read.address.next = self.read.ram_read.address.val();
        self.read.ram_read.data.next = self.ram.read.data.val();
        // Connect the two blocks
        self.read.write_address_delayed.next = self.write.write_address_delayed.val();
        self.write.read_address.next = self.read.read_address_out.val();
    }
}

#[test]
fn component_fifo_is_synthesizable() {
    rust_hdl_synth::top_wrap!(SynchronousFIFO<Bits<8>, MHz1, 4, 5, 1>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.read_if.read.connect();
    dev.uut.write_if.write.connect();
    dev.uut.write_if.data_in.connect();
    dev.connect_all();
    yosys_validate("fifo", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}
