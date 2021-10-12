use crate::dff::DFF;
use crate::fifo_expander_n::WordOrder;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::TopWrap;

#[derive(LogicBlock)]
pub struct FIFOReducerN<const DW: usize, const DN: usize> {
    // Data comes by reading from the source FIFO
    pub data_in: Signal<In, Bits<DW>>,
    pub read: Signal<Out, Bit>,
    pub empty: Signal<In, Bit>,
    // Data is written to the output FIFO
    pub data_out: Signal<Out, Bits<DN>>,
    pub write: Signal<Out, Bit>,
    pub full: Signal<In, Bit>,
    // This is a synchronous design.  The clock is assumed
    // to be shared with both the input and output fifos.
    pub clock: Signal<In, Clock>,
    load_count: DFF<Bits<8>>,
    data_available: Signal<Local, Bit>,
    will_write: Signal<Local, Bit>,
    will_consume: Signal<Local, Bit>,
    data_store: DFF<Bits<DW>>,
    msw_first: Constant<Bit>,
    ratio: Constant<Bits<8>>,
    offset: Constant<Bits<DW>>,
    select: Constant<Bits<DW>>,
}

impl<const DW: usize, const DN: usize> Logic for FIFOReducerN<DW, DN> {
    #[hdl_gen]
    fn update(&mut self) {
        self.load_count.clk.next = self.clock.val();
        self.load_count.d.next = self.load_count.q.val();
        self.data_store.clk.next = self.clock.val();
        self.data_store.d.next = self.data_store.q.val();
        // We have data if either the store has data or if data is ready
        // from the input fifo
        self.data_available.next = self.load_count.q.val().any() | !self.empty.val();
        // If we have data available, and output interface has space, we will write data.
        self.will_write.next = self.data_available.val() & !self.full.val();
        // If we have only one data element left, and we will write, then we need data
        // Or if we have no data, we need data
        self.will_consume.next =
            !self.load_count.q.val().any() & !self.empty.val() & self.will_write.val();
        if self.load_count.q.val().any() {
            // If the store contains data, then the output comes from the rightmost
            // bits of the data store
            self.data_out.next = self
                .data_store
                .q
                .val()
                .get_bits::<DN>(self.select.val().into())
        } else {
            // Otherwise, it comes directly from the read interface
            self.data_out.next = self.data_in.val().get_bits::<DN>(self.select.val().into());
        }
        // If we will write, then the data store should be right shifted.
        if self.will_write.val() {
            if !self.msw_first.val() {
                self.data_store.d.next = self.data_store.q.val() >> self.offset.val();
            } else {
                self.data_store.d.next = self.data_store.q.val() << self.offset.val();
            }
            if self.load_count.q.val().any() {
                self.load_count.d.next = self.load_count.q.val() - 1_u32;
            }
        }
        // if we will consume, then the store input comes from the data store
        if self.will_consume.val() {
            if !self.msw_first.val() {
                self.data_store.d.next = self.data_in.val() >> self.offset.val();
            } else {
                self.data_store.d.next = self.data_in.val() << self.offset.val();
            }
            self.load_count.d.next = self.ratio.val();
        }
        self.write.next = self.will_write.val();
        self.read.next = self.will_consume.val();
    }
}

impl<const DW: usize, const DN: usize> FIFOReducerN<DW, DN> {
    pub fn new(order: WordOrder) -> Self {
        assert_eq!(DW % DN, 0);
        let msw_first = match order {
            WordOrder::LeastSignificantFirst => false,
            WordOrder::MostSignificantFirst => true,
        };
        Self {
            data_in: Default::default(),
            read: Default::default(),
            empty: Default::default(),
            data_out: Default::default(),
            write: Default::default(),
            full: Default::default(),
            clock: Default::default(),
            load_count: Default::default(),
            data_available: Default::default(),
            will_write: Default::default(),
            will_consume: Default::default(),
            data_store: Default::default(),
            msw_first: Constant::new(msw_first),
            ratio: Constant::new((DW / DN - 1).into()),
            offset: Constant::new(DN.into()),
            select: if !msw_first {
                Constant::new(0_u32.into())
            } else {
                Constant::new((DW - DN).into())
            },
        }
    }
}

#[test]
fn fifo_reducern_is_synthesizable() {
    let mut dev = TopWrap::new(FIFOReducerN::<32, 4>::new(WordOrder::MostSignificantFirst));
    dev.uut.empty.connect();
    dev.uut.full.connect();
    dev.uut.data_in.connect();
    dev.uut.clock.connect();
    dev.connect_all();
    rust_hdl_synth::yosys_validate("fifo_reducern", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}
