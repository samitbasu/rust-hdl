use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;
use crate::mcb_if::MCBInterface;
use crate::ddr_fifo::DDRFIFO;
use rust_hdl_widgets::fifo_reducer::FIFOReducer;
use rust_hdl_widgets::prelude::SynchronousFIFO;
use crate::ok_pipe::BTPipeOut;

#[derive(LogicBlock, Default)]
pub struct OpalKellyDDRBackedDownloadFIFO<const PORT: u8> {
    pub mcb: MCBInterface,
    pub raw_sys_clock: Signal<In, Clock>,
    // You must assert reset!
    pub reset: Signal<In, Bit>,
    // FIFO In interface
    pub data_in: Signal<In, Bits<32>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub almost_full: Signal<Out, Bit>,
    pub write_clock: Signal<In, Clock>,
    // The OK pipe out side requires the ti clock,
    // and connections to the ok1 and ok2 busses
    pub ti_clk: Signal<In, Clock>,
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    //  The DDR-backed FIFO
    pub ddr_fifo: DDRFIFO, // TODO - don't need this public
    reducer: FIFOReducer<32, 16, false>,
    fifo_out: SynchronousFIFO<Bits<16>, 13, 14, 512>,
    o_pipe: BTPipeOut<PORT>,
    read_delay: DFF<Bit>,
}

impl<const PORT: u8> Logic for OpalKellyDDRBackedDownloadFIFO<PORT> {
    #[hdl_gen]
    fn update(&mut self) {
        self.mcb.link(&mut self.ddr_fifo.mcb);
        self.ddr_fifo.reset.next = self.reset.val();
        self.ddr_fifo.raw_sys_clock.next = self.raw_sys_clock.val();
        self.fifo_out.clock.next = self.ti_clk.val();
        self.reducer.clock.next = self.ti_clk.val();
        self.read_delay.clk.next = self.ti_clk.val();
        self.ddr_fifo.write_clock.next = self.write_clock.val();
        self.ddr_fifo.read_clock.next = self.ti_clk.val();
        // Data source - counts on each strobe pulse and writes it to the input FIFO.
        self.ddr_fifo.data_in.next = self.data_in.val();
        self.ddr_fifo.write.next = self.write.val();
        self.full.next = self.ddr_fifo.full.val();
        self.almost_full.next = self.ddr_fifo.almost_full.val();
        // Link the DDR fifo to the output fifo via the reducer
        self.reducer.empty.next = self.ddr_fifo.empty.val();
        self.reducer.data_in.next = self.ddr_fifo.data_out.val();
        self.ddr_fifo.read.next = self.reducer.read.val();
        self.fifo_out.data_in.next = self.reducer.data_out.val();
        self.fifo_out.write.next = self.reducer.write.val();
        self.reducer.full.next = self.fifo_out.full.val();
        // Add a 1 cycle delay between the output pipe and the read delay
        self.fifo_out.read.next = self.read_delay.q.val();
        self.read_delay.d.next = self.o_pipe.read.val();
        self.o_pipe.ready.next = !self.fifo_out.almost_empty.val();
        self.o_pipe.datain.next = self.fifo_out.data_out.val();
        self.o_pipe.ok1.next = self.ok1.val();
        self.ok2.next = self.o_pipe.ok2.val();
    }
}

#[test]
fn test_ddr_download_fifo_gen() {
    let ddr : OpalKellyDDRBackedDownloadFIFO<0xA0> = Default::default();
    let vlog = generate_verilog_unchecked(&ddr);
    println!("{}", vlog);
}