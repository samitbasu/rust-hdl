use crate::dff::DFF;
use crate::ram::RAM;
use rust_hdl_core::bits::bit_cast;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;

#[derive(LogicBlock)]
pub struct SyncFIFO<D: Synth, T: Domain, const N: usize, const BlockSize: u32> {
    // Clock interface
    pub clock: Signal<In, Clock, T>,
    // FIFO Read interface
    pub read: Signal<In, Bit, T>,
    pub data_out: Signal<Out, D, T>,
    pub empty: Signal<Out, Bit, T>,
    pub almost_empty: Signal<Out, Bit, T>,
    pub underflow: Signal<Out, Bit, T>,
    // FIFO Write interface
    pub write: Signal<In, Bit, T>,
    pub data_in: Signal<In, D, T>,
    pub full: Signal<Out, Bit, T>,
    pub almost_full: Signal<Out, Bit, T>,
    pub overflow: Signal<Out, Bit, T>,
    // Internal RAM and address registers
    ram: RAM<Bits<N>, D, T, T>,
    write_address: DFF<Bits<N>, T>,
    write_address_delayed: DFF<Bits<N>, T>,
    read_address: DFF<Bits<N>, T>,
    // Constant sizes
    fifo_size: Constant<Bits<32>>,
    block_size: Constant<Bits<32>>,
    // Local signals (temps)
    write_ptr: Signal<Local, Bits<32>, T>,
    read_ptr: Signal<Local, Bits<32>, T>,
    fill_level: Signal<Local, Bits<32>, T>,
    free_space: Signal<Local, Bits<32>, T>,
    write_ready: Signal<Local, Bit, T>,
    read_ready: Signal<Local, Bit, T>,
    // Error flags are latching...
    dff_overflow: DFF<Bit, T>,
    dff_underflow: DFF<Bit, T>,
}

impl<D: Synth, T: Domain, const N: usize, const Blocksize: u32> Default
    for SyncFIFO<D, T, N, Blocksize>
{
    fn default() -> Self {
        Self {
            clock: Default::default(),
            read: Default::default(),
            data_out: Default::default(),
            empty: Default::default(),
            almost_empty: Default::default(),
            underflow: Default::default(),
            write: Default::default(),
            data_in: Default::default(),
            full: Default::default(),
            almost_full: Default::default(),
            overflow: Default::default(),
            ram: RAM::new(Default::default()),
            write_address: Default::default(),
            write_address_delayed: Default::default(),
            read_address: Default::default(),
            fifo_size: Constant::new(Bits::<N>::count().into()),
            block_size: Constant::new(Blocksize.into()),
            write_ptr: Default::default(),
            read_ptr: Default::default(),
            fill_level: Default::default(),
            free_space: Default::default(),
            write_ready: Default::default(),
            read_ready: Default::default(),
            dff_overflow: Default::default(),
            dff_underflow: Default::default(),
        }
    }
}

// Ported from fifo.luc in AlchitryLabs under MIT license
impl<D: Synth, T: Domain, const N: usize, const Blocksize: u32> Logic
    for SyncFIFO<D, T, N, Blocksize>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clocks.
        self.ram.read_clock.next = self.clock.val();
        self.ram.write_clock.next = self.clock.val();
        self.write_address.clk.next = self.clock.val();
        self.write_address_delayed.clk.next = self.clock.val();
        self.read_address.clk.next = self.clock.val();
        self.dff_overflow.clk.next = self.clock.val();
        self.dff_underflow.clk.next = self.clock.val();

        // We need a 1 cycle delay on the write address
        // This ensures we do not try to read a data element on the same
        // cycle it is written.
        self.write_address_delayed.d.next = self.write_address.q.val();
        // Default to not writing
        self.ram.write_enable.next = false.into();
        // We can write if there is at least 1 space open
        self.write_ready.next =
            ((self.write_address.q.val() + 1_u32) != self.read_address.q.val()).into();
        // We can read if there is at least 1 element in the FIFO (use delayed write address here)
        self.read_ready.next =
            (self.read_address.q.val() != self.write_address_delayed.q.val()).into();
        // Cast the write address to a 32 bit pointer (assumes the FIFO is smaller than 4G elements in size)
        self.write_ptr.next = tagged_bit_cast::<T, 32, { N }>(self.write_address_delayed.q.val());
        // Cast the read address to a 32 bit pointer as well
        self.read_ptr.next = tagged_bit_cast::<T, 32, { N }>(self.read_address.q.val());
        // Compute the fill level - we add N first, since
        self.fill_level.next = (self.write_ptr.val() + self.fifo_size.val() - self.read_ptr.val())
            & bit_cast::<32, { N }>(Bits::<N>::mask());
        // The free space is simply N-1-fill level
        self.free_space.next = self.fifo_size.val() - self.fill_level.val() - 1_u32;
        // Compute the almost full signal
        self.almost_full.next = (self.free_space.val() < self.block_size.val()).into();
        // Compute the almost empty signal
        self.almost_empty.next = (self.fill_level.val() < self.block_size.val()).into();
        // Propogate these signals to the outside interface
        self.full.next = !self.write_ready.val();
        self.empty.next = !self.read_ready.val();

        // Connect our write address register to the RAM write address
        self.ram.write_address.next = self.write_address.q.val();
        self.ram.read_address.next = self.read_address.q.val();
        self.ram.write_data.next = self.data_in.val();
        self.data_out.next = self.ram.read_data.val();

        // Assign the enable for the write based on the outside
        // request and our availability to write
        if (self.write.val() & self.write_ready.val()).any() {
            self.write_address.d.next = self.write_address.q.val() + 1_u32;
            self.ram.write_enable.next = true.into();
        } else {
            self.write_address.d.next = self.write_address.q.val();
            self.ram.write_enable.next = false.into();
        }

        // Assign the read advance based on the outside request
        // and our availability to read.
        if (self.read.val() & self.read_ready.val()).any() {
            self.read_address.d.next = self.read_address.q.val() + 1_u32;
            self.ram.read_address.next = self.read_address.q.val() + 1_u32;
        } else {
            self.read_address.d.next = self.read_address.q.val();
            self.ram.read_address.next = self.read_address.q.val();
        }

        // Compute the overflow signal - it is latched
        self.dff_overflow.d.next =
            self.dff_overflow.q.val() | (!self.write_ready.val() & self.write.val());
        self.dff_underflow.d.next =
            self.dff_underflow.q.val() | (!self.read_ready.val() & self.read.val());
        self.overflow.next = self.dff_overflow.q.val();
        self.underflow.next = self.dff_underflow.q.val();
    }
}

make_domain!(MHz1, 1_000_000);

#[test]
fn fifo_is_synthesizable() {
    rust_hdl_synth::top_wrap!(SyncFIFO<Bits<8>, MHz1, 4, 1>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.read.connect();
    dev.uut.write.connect();
    dev.uut.data_in.connect();
    dev.connect_all();
    yosys_validate("fifo", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}
