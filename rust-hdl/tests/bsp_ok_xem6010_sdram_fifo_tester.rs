use rust_hdl::bsp::ok_xem6010::XEM6010;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

mod test_common;
#[cfg(feature = "frontpanel")]
use test_common::download::*;
use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::sim::sdr_sdram::chip::SDRAMSimulator;
use rust_hdl::hls::sdram_fifo::SDRAMFIFO;
use rust_hdl::bsp::ok_xem6010::pins::xem_6010_base_clock;

#[derive(LogicBlock)]
struct SDRAMSimulatedFIFOTester {
    pub hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    counter: DFF<Bits<16>>,
    sdram: SDRAMSimulator<16>,
    fifo: SDRAMFIFO<5, 5, 12, 16>,
    clock: Signal<In, Clock>,
    cross: AsynchronousFIFO<Bits<16>, 4, 5, 1>,
    dl: OpalKellyDownloadFIFO,
    will_write: Signal<Local, Bit>,
    will_read: Signal<Local, Bit>,
    will_cross: Signal<Local, Bit>,
}

impl SDRAMSimulatedFIFOTester {
    pub fn new<B: OpalKellyBSP>() -> Self {
        let timing = MemoryTimings::fast_boot_sim(100e6);
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            counter: Default::default(),
            sdram: SDRAMSimulator::new(timing),
            fifo: SDRAMFIFO::new(3, timing),
            clock: xem_6010_base_clock(),
            cross: Default::default(),
            dl: OpalKellyDownloadFIFO::new(0xA0),
            will_write: Default::default(),
            will_read: Default::default(),
            will_cross: Default::default()
        }
    }
}

impl Logic for SDRAMSimulatedFIFOTester {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        // Fast clock for these components
        self.counter.clk.next = self.clock.val();
        self.sdram.clock.next = self.clock.val();
        self.fifo.clock.next = self.clock.val();
        self.cross.write_clock.next = self.clock.val();
        // Slow clock here
        self.cross.read_clock.next = self.ok_host.ti_clk.val();
        self.dl.clock.next = self.ok_host.ti_clk.val();
        // Connect the counter to the SDRAM-FIFO input bus
        self.will_write.next = !self.fifo.bus_write.full.val();
        self.counter.d.next = self.counter.q.val() + self.will_write.val();
        self.fifo.bus_write.data.next = self.counter.q.val();
        self.fifo.bus_write.write.next = self.will_write.val();
        // Connect the cross fifo to the SDRAM-FIFO output bus
        self.will_read.next = !self.fifo.bus_read.empty.val() & !self.cross.full.val();
        self.fifo.bus_read.read.next = self.will_read.val();
        self.cross.data_in.next = self.fifo.bus_read.data.val();
        self.cross.write.next = self.will_read.val();
        // Connect the cross fifo output to the DL widget
        self.will_cross.next = !self.cross.empty.val() & !self.dl.data_full.val();
        self.dl.data_in.next = self.cross.data_out.val();
        self.dl.data_write.next = self.will_cross.val();
        self.cross.read.next = self.will_cross.val();
        // Connect the DL widget to the OK busses
        self.dl.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.dl.ok2.val();
        // Link the SDRAM and the controller
        self.sdram.data.join(&mut self.fifo.data);
        self.sdram.address.next = self.fifo.address.val();
        self.sdram.cmd.next = self.fifo.cmd.val();
        self.sdram.bank.next = self.fifo.bank.val();
    }
}

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_6010_sdram_simulated_fifo_download() {
    let mut uut = SDRAMSimulatedFIFOTester::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.clock.connect();
    uut.connect_all();
    rust_hdl::bsp::ok_xem6010::synth::synth_obj(uut, target_path!("xem_6010/sdram_fifo_sim"));
    test_opalkelly_download_runtime(target_path!("xem_6010/sdram_fifo_sim/top.bit")).unwrap()
}

