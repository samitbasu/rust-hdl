use crate::ok_pipe::{BTPipeOut, PipeOut};
use rust_hdl_core::prelude::*;
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
}

impl Logic for OpalKellyDownloadFIFO {
    #[hdl_gen]
    fn update(&mut self) {
        self.delay_read.clk.next = self.clock.val();
        self.fifo.clock.next = self.clock.val();
        self.o_pipe.ok1.next = self.ok1.val();
        self.ok2.next = self.o_pipe.ok2.val();
        self.o_pipe.datain.next = self.fifo.data_out.val();
        self.o_pipe.ready.next = !self.fifo.almost_empty.val();
        self.delay_read.d.next = self.o_pipe.read.val();
        self.fifo.read.next = self.delay_read.q.val();
        self.fifo.data_in.next = self.data_in.val();
        self.fifo.write.next = self.data_write.val();
        self.data_full.next = self.fifo.full.val();
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
        }
    }
}

#[test]
fn test_okdf_synthesizes() {
    let uut = OpalKellyDownloadFIFO::new(0xA0);
    generate_verilog_unchecked(&uut);
}
