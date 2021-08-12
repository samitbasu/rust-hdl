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
    // These should be N+1 bits, but Rust does not
    // currently allow manipulation of const generics.
    // So we assume the largest FIFO you will instantiate
    // is < 4GB.  I assume that the synthesis software
    // will remove the unused extra bits (hence the masks).
    write_address: DFF<Bits<32>, T>,
    write_address_delayed: DFF<Bits<32>, T>,
    read_address: DFF<Bits<32>, T>,
    // Constant sizes
    // This is the total number of elements the FIFO can store
    fifo_size: Constant<Bits<32>>,
    // This is the size of the "almost" block.  I.e., if
    // there are fewer than block_size spaces, it is "almost full"
    block_size: Constant<Bits<32>>,
    // This is the mask for the bottom N bits of the address
    fifo_address_mask: Constant<Bits<32>>,
    // This is the mask for the bottom N+1 bits of the address
    fifo_ptr_mask: Constant<Bits<32>>,
    // Local signals (temps)
    fill_level: Signal<Local, Bits<32>, T>,
    free_space: Signal<Local, Bits<32>, T>,
    is_full: Signal<Local, Bit, T>,
    is_empty: Signal<Local, Bit, T>,
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
            fifo_address_mask: Constant::new(((1_u32 << (N)) - 1).into()),
            fifo_ptr_mask: Constant::new(((1_u32 << (N+1)) - 1).into()),
            fill_level: Default::default(),
            free_space: Default::default(),
            is_full: Default::default(),
            is_empty: Default::default(),
            dff_overflow: Default::default(),
            dff_underflow: Default::default(),
        }
    }
}

// Ported from fifo.luc in AlchitryLabs under MIT license
// Modified to use an extra bit for the read/write address
// pointers per:
// http://www.sunburst-design.com/papers/CummingsSNUG2002SJ_FIFO1.pdf
// That modification allows you to fill the FIFO all the way
// since you have 2 different conditions for full and empty.
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
        // Calculate the empty field - this is used to determine if we
        // can read
        self.is_empty.next = (self.read_address.q.val() == self.write_address.q.val()).into();
        // Calculate the is full field.  If the FIFO is not empty, and
        // the lower N bits of the addresses agree, the FIFO is full
        self.is_full.next = !self.is_empty.val() &
            ((self.read_address.q.val() & self.fifo_address_mask.val()) ==
                (self.write_address.q.val() & self.fifo_address_mask.val()));
        // Compute the fill level - we add N first, since we are subtracting.  And
        // we mask out the lower N bits, since we are ignoring the wrap levels.
        // Note that if the FIFO is empty, this calculation will give the wrong
        // answer, so we need to check the is_empty flag (which uses all N+1 bits).
        self.fill_level.next = ((self.write_address.q.val() & self.fifo_address_mask.val())
            + self.fifo_size.val() - (self.read_address.q.val() & self.fifo_address_mask.val()));
        if self.is_empty.val().any() {
            self.fill_level.next = 0_u32.into();
        }
        // The free space is simply N-fill level
        self.free_space.next = self.fifo_size.val() - self.fill_level.val();
        // Compute the almost full signal
        self.almost_full.next = (self.free_space.val() < self.block_size.val()).into();
        // Compute the almost empty signal
        self.almost_empty.next = (self.fill_level.val() < self.block_size.val()).into();
        // Propagate these signals to the outside interface
        self.full.next = self.is_full.val();
        self.empty.next = self.is_empty.val();

        // Connect our write address register to the RAM write address
        self.ram.write_address.next = tagged_bit_cast::<_, N, 32>(self.write_address.q.val() &
            self.fifo_address_mask.val());
        self.ram.read_address.next = tagged_bit_cast::<_, N, 32>(self.read_address.q.val() &
            self.fifo_address_mask.val());
        self.ram.write_data.next = self.data_in.val();
        self.data_out.next = self.ram.read_data.val();

        // Assign the enable for the write based on the outside
        // request and our availability to write
        if (self.write.val() & !self.is_full.val()).any() {
            self.write_address.d.next = (self.write_address.q.val() + 1_u32) &
                self.fifo_ptr_mask.val();
            self.ram.write_enable.next = true.into();
        } else {
            self.write_address.d.next = self.write_address.q.val();
            self.ram.write_enable.next = false.into();
        }

        // Assign the read advance based on the outside request
        // and our availability to read.
        if (self.read.val() & !self.is_empty.val()).any() {
            self.read_address.d.next = (self.read_address.q.val() + 1_u32) &
                self.fifo_ptr_mask.val();
        } else {
            self.read_address.d.next = self.read_address.q.val();
        }

        // Compute the overflow signal - it is latched
        self.dff_overflow.d.next =
            self.dff_overflow.q.val() | (self.is_full.val() & self.write.val());
        self.dff_underflow.d.next =
            self.dff_underflow.q.val() | (self.is_empty.val() & self.read.val());
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
