use crate::ok_pipe::BTPipeOut;
use crate::ok_wire::WireOut;
use rust_hdl_core::bits::bits;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::fifo_reducer::FIFOReducer;
use rust_hdl_widgets::prelude::*;

declare_sync_fifo!(OKDLFIFO, Bits<16>, 8192, 256);

#[derive(LogicBlock)]
pub struct OpalKellyDownloadFIFO {
    pub clock: Signal<In, Clock>,
    pub data_in: Signal<In, Bits<16>>,
    pub data_write: Signal<In, Bit>,
    pub data_full: Signal<Out, Bit>,
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    fifo: OKDLFIFO,
    o_pipe: BTPipeOut,
    delay_read: DFF<Bit>,
    status_wire: WireOut,
}

impl Logic for OpalKellyDownloadFIFO {
    #[hdl_gen]
    fn update(&mut self) {
        self.delay_read.clk.next = self.clock.val();
        self.fifo.clock.next = self.clock.val();
        self.o_pipe.ok1.next = self.ok1.val();
        self.status_wire.ok1.next = self.ok1.val();
        self.ok2.next = self.o_pipe.ok2.val() | self.status_wire.ok2.val();
        self.o_pipe.datain.next = self.fifo.data_out.val();
        self.o_pipe.ready.next = !self.fifo.almost_empty.val();
        self.delay_read.d.next = self.o_pipe.read.val();
        self.fifo.read.next = self.delay_read.q.val();
        self.fifo.data_in.next = self.data_in.val();
        self.fifo.write.next = self.data_write.val();
        self.data_full.next = self.fifo.full.val();
        self.status_wire.datain.next = bits::<16>(0xAD00_u128)
            | (bit_cast::<16, 1>(self.fifo.overflow.val().into()) << 4_u32)
            | (bit_cast::<16, 1>(self.fifo.full.val().into()) << 3_u32)
            | (bit_cast::<16, 1>(self.fifo.almost_empty.val().into()) << 2_u32)
            | (bit_cast::<16, 1>(self.fifo.empty.val().into()) << 1_u32)
            | (bit_cast::<16, 1>(self.fifo.underflow.val().into()) << 0_u32);
    }
}

impl OpalKellyDownloadFIFO {
    pub fn new(port: u8) -> Self {
        Self {
            clock: Default::default(),
            data_in: Default::default(),
            data_write: Default::default(),
            data_full: Default::default(),
            ok1: Default::default(),
            ok2: Default::default(),
            fifo: Default::default(),
            o_pipe: BTPipeOut::new(port),
            delay_read: Default::default(),
            status_wire: WireOut::new(0x38),
        }
    }
}

#[test]
fn test_okdf_synthesizes() {
    let uut = OpalKellyDownloadFIFO::new(0xA0);
    generate_verilog_unchecked(&uut);
}

declare_sync_fifo!(OKStageFIFO, Bits<32>, 32, 1);

#[derive(LogicBlock)]
pub struct OpalKellyDownload32FIFO {
    pub clock: Signal<In, Clock>,
    pub data_in: Signal<In, Bits<32>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    upstage: OKStageFIFO,
    redux: FIFOReducer<32, 16, false>,
    dfifo: OpalKellyDownloadFIFO,
}

impl Logic for OpalKellyDownload32FIFO {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clocks
        self.upstage.clock.next = self.clock.val();
        self.redux.clock.next = self.clock.val();
        self.dfifo.clock.next = self.clock.val();
        // Connect the OK bus
        self.dfifo.ok1.next = self.ok1.val();
        self.ok2.next = self.dfifo.ok2.val();
        // Connect the redux to the download fifo
        self.dfifo.data_in.next = self.redux.data_out.val();
        self.dfifo.data_write.next = self.redux.write.val();
        self.redux.full.next = self.dfifo.data_full.val();
        // Connect the upstage fifo to the redux
        self.redux.data_in.next = self.upstage.data_out.val();
        self.redux.empty.next = self.upstage.empty.val();
        self.upstage.read.next = self.redux.read.val();
        // Connect the upstage fifo to our inputs
        self.upstage.data_in.next = self.data_in.val();
        self.upstage.write.next = self.write.val();
        self.full.next = self.upstage.full.val();
    }
}

impl OpalKellyDownload32FIFO {
    pub fn new(port: u8) -> Self {
        Self {
            clock: Default::default(),
            data_in: Default::default(),
            write: Default::default(),
            full: Default::default(),
            ok1: Default::default(),
            ok2: Default::default(),
            upstage: Default::default(),
            redux: Default::default(),
            dfifo: OpalKellyDownloadFIFO::new(port),
        }
    }
}
