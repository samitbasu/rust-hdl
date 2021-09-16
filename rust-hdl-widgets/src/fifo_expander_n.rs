use crate::dff::DFF;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::TopWrap;

pub enum WordOrder {
    LeastSignificantFirst,
    MostSignificantFirst,
}

#[derive(LogicBlock)]
pub struct FIFOExpanderN<const DN: usize, const DW: usize> {
    // Data comes by reading from the source FIFO
    pub data_in: Signal<In, Bits<DN>>,
    pub read: Signal<Out, Bit>,
    pub empty: Signal<In, Bit>,
    // Data is written to the output FIFO
    pub data_out: Signal<Out, Bits<DW>>,
    pub write: Signal<Out, Bit>,
    pub full: Signal<In, Bit>,
    // Synchronous design.  Assumes the same clock drives the
    // corresponding interfaces of the input and output fifos.
    pub clock: Signal<In, Clock>,
    load_count: DFF<Bits<8>>,
    loaded: Signal<Local, Bit>,
    complete_data_available: Signal<Local, Bit>,
    will_write: Signal<Local, Bit>,
    will_consume: Signal<Local, Bit>,
    data_store: DFF<Bits<DW>>,
    offset: Constant<Bits<DW>>,
    ratio: Constant<Bits<8>>,
    placement: Constant<Bits<DW>>,
    msw_first: Constant<bool>,
}

impl<const DN: usize, const DW: usize> Logic for FIFOExpanderN<DN, DW> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clocks and latch prevention for the DFFs
        self.load_count.clk.next = self.clock.val();
        self.data_store.clk.next = self.clock.val();
        self.load_count.d.next = self.load_count.q.val();
        self.data_store.d.next = self.data_store.q.val();
        // Loaded if we have shifted M-1 data elements into the data store
        self.loaded.next = self.load_count.q.val() == self.ratio.val();
        // Complete data is available if we have shifted M-1 data elements into
        // the data store, and there is data available at the input
        self.complete_data_available.next = self.loaded.val() & !self.empty.val();
        // We will write if the write interface is not full and we have compelte data
        self.will_write.next = self.complete_data_available.val() & !self.full.val();
        // We will consume if there is data available and we will write or if we are not loaded
        self.will_consume.next = !self.empty.val() & (self.will_write.val() | !self.loaded.val());
        // If we will consume data and we are not loaded, then the data goes to the store
        if self.will_consume.val() & !self.loaded.val() {
            if self.msw_first.val() {
                self.data_store.d.next = (self.data_store.q.val() << self.offset.val())
                    | bit_cast::<DW, DN>(self.data_in.val());
            } else {
                self.data_store.d.next = (self.data_store.q.val() >> self.offset.val())
                    | bit_cast::<DW, DN>(self.data_in.val()) << self.placement.val();
            }
            self.load_count.d.next = self.load_count.q.val() + 1_u32;
        }
        // The output FIFO always sees the data store shifted with the input or-ed in
        if self.msw_first.val() {
            self.data_out.next = bit_cast::<DW, DN>(self.data_in.val())
                | (self.data_store.q.val() << self.offset.val());
        } else {
            self.data_out.next = bit_cast::<DW, DN>(self.data_in.val()) << self.placement.val()
                | (self.data_store.q.val() >> self.offset.val());
        }
        self.write.next = self.will_write.val();
        self.read.next = self.will_consume.val();
        if self.will_write.val() {
            self.load_count.d.next = 0_u32.into();
        }
    }
}

impl<const DN: usize, const DW: usize> FIFOExpanderN<DN, DW> {
    pub fn new(order: WordOrder) -> Self {
        assert!(DW > DN);
        assert_eq!(DW % DN, 0);
        Self {
            data_in: Default::default(),
            read: Default::default(),
            empty: Default::default(),
            data_out: Default::default(),
            write: Default::default(),
            full: Default::default(),
            clock: Default::default(),
            load_count: Default::default(),
            loaded: Default::default(),
            complete_data_available: Default::default(),
            will_write: Default::default(),
            will_consume: Default::default(),
            data_store: Default::default(),
            offset: Constant::new(DN.into()),
            ratio: Constant::new((DW / DN - 1).into()),
            placement: Constant::new((DN * (DW / DN - 1)).into()),
            msw_first: Constant::new(match order {
                WordOrder::LeastSignificantFirst => false,
                WordOrder::MostSignificantFirst => true,
            }),
        }
    }
}

#[test]
fn fifo_expandern_is_synthesizable() {
    let mut dev = TopWrap::new(FIFOExpanderN::<4, 32>::new(WordOrder::MostSignificantFirst));
    dev.uut.empty.connect();
    dev.uut.full.connect();
    dev.uut.data_in.connect();
    dev.uut.clock.connect();
    dev.connect_all();
    rust_hdl_synth::yosys_validate("fifo_expandern", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}
