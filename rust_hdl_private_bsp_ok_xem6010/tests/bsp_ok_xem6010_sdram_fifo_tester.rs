use rust_hdl::prelude::*;
use rust_hdl_private_bsp_ok_xem6010::xem6010;
use rust_hdl_private_bsp_ok_xem6010::xem6010::pins::xem_6010_base_clock;
use rust_hdl_private_bsp_ok_xem6010::xem6010::XEM6010;
use rust_hdl_private_hls::sdram_fifo::SDRAMFIFO;
use rust_hdl_private_ok_core::core::prelude::*;
use rust_hdl_private_ok_core::test_common::download::test_opalkelly_download_runtime;
use rust_hdl_private_sim::sdr_sdram::chip::SDRAMSimulator;

#[derive(LogicBlock)]
struct SDRAMSimulatedFIFOTester {
    pub hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    counter: DFF<Bits<16>>,
    chip: SDRAMSimulator<5, 5, 10, 16>,
    fifo: SDRAMFIFO<5, 5, 16, 16, 12>,
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
            chip: SDRAMSimulator::new(timing),
            fifo: SDRAMFIFO::new(3, timing, OutputBuffer::Wired),
            clock: xem_6010_base_clock(),
            cross: Default::default(),
            dl: OpalKellyDownloadFIFO::new(0xA0),
            will_write: Default::default(),
            will_read: Default::default(),
            will_cross: Default::default(),
        }
    }
}

impl Logic for SDRAMSimulatedFIFOTester {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        // Fast clock for these components
        self.counter.clock.next = self.clock.val();
        self.fifo.clock.next = self.clock.val();
        self.fifo.ram_clock.next = self.clock.val();
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
        SDRAMDriver::<16>::join(&mut self.fifo.sdram, &mut self.chip.sdram);
    }
}

#[test]
fn test_opalkelly_xem_6010_sdram_simulated_fifo_download() {
    let mut uut = SDRAMSimulatedFIFOTester::new::<XEM6010>();
    uut.connect_all();
    xem6010::synth::synth_obj(uut, target_path!("xem_6010/sdram_fifo_sim"));
    test_opalkelly_download_runtime(
        target_path!("xem_6010/sdram_fifo_sim/top.bit"),
        env!("XEM6010_SERIAL"),
    )
    .unwrap()
}
