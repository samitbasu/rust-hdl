use crate::ddr_fifo::DDRFIFO;
use crate::ddr_fifo7::DDR7FIFO;
use crate::mcb_if::MCBInterface4GDDR3;
use crate::ok_pipe::BTPipeOut;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;

#[derive(LogicBlock)]
pub struct OpalKellyDDRBackedDownloadFIFO7Series {
    pub mcb: MCBInterface4GDDR3,
    pub sys_clock_p: Signal<In, Clock>,
    pub sys_clock_n: Signal<In, Clock>,
    // You must assert reset!
    pub reset: Signal<In, Bit>,
    // FIFO In interface
    pub data_in: Signal<In, Bits<32>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub write_clock: Signal<In, Clock>,
    // The OK pipe out side requires the ti clock,
    // and connections to the ok1 and ok2 busses
    pub ti_clk: Signal<In, Clock>,
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    //  The DDR-backed FIFO
    ddr_fifo: DDR7FIFO<32>,
    reducer: FIFOReducer<32, 16, false>,
    fifo_out: SynchronousFIFO<Bits<16>, 13, 14, 512>,
    o_pipe: BTPipeOut,
    read_delay: DFF<Bit>,
}

impl OpalKellyDDRBackedDownloadFIFO7Series {
    pub fn new(n: u8) -> Self {
        Self {
            mcb: Default::default(),
            sys_clock_p: Default::default(),
            sys_clock_n: Default::default(),
            reset: Default::default(),
            data_in: Default::default(),
            write: Default::default(),
            full: Default::default(),
            write_clock: Default::default(),
            ti_clk: Default::default(),
            ok1: Default::default(),
            ok2: Default::default(),
            ddr_fifo: Default::default(),
            reducer: Default::default(),
            fifo_out: Default::default(),
            o_pipe: BTPipeOut::new(n),
            read_delay: Default::default(),
        }
    }
}

impl Logic for OpalKellyDDRBackedDownloadFIFO7Series {
    #[hdl_gen]
    fn update(&mut self) {
        self.mcb.link(&mut self.ddr_fifo.mcb);
        self.ddr_fifo.reset.next = self.reset.val();
        self.ddr_fifo.sys_clock_p.next = self.sys_clock_p.val();
        self.ddr_fifo.sys_clock_n.next = self.sys_clock_n.val();
        self.fifo_out.clock.next = self.ti_clk.val();
        self.reducer.clock.next = self.ti_clk.val();
        self.read_delay.clk.next = self.ti_clk.val();
        self.ddr_fifo.write_clock.next = self.write_clock.val();
        self.ddr_fifo.read_clock.next = self.ti_clk.val();
        // Data source - counts on each strobe pulse and writes it to the input FIFO.
        self.ddr_fifo.data_in.next = self.data_in.val();
        self.ddr_fifo.write.next = self.write.val();
        self.full.next = self.ddr_fifo.full.val();
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
fn test_ddr_download_fifo7_gen() {
    let ddr = OpalKellyDDRBackedDownloadFIFO7Series::new(0xA0);
    let vlog = generate_verilog_unchecked(&ddr);
    println!("{}", vlog);
}
