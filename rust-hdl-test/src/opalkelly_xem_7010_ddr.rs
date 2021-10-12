use crate::ok_tools::ok_test_prelude;
use crate::opalkelly_xem_6010_ddr::test_opalkelly_ddr_stress_runtime;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::ddr_fifo7::DDR7FIFO;
use rust_hdl_ok::mcb_if::MCBInterface4GDDR3;
use rust_hdl_ok::ok_download_ddr7::OpalKellyDDRBackedDownloadFIFO7Series;
use rust_hdl_ok::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_widgets::dff::DFF;
use rust_hdl_widgets::prelude::*;
use std::time::Instant;

#[derive(LogicBlock)]
struct OpalKellyDDR7Test {
    mcb: MCBInterface4GDDR3,
    hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    ddr_fifo: DDR7FIFO<16>,
    sys_clock_p: Signal<In, Clock>,
    sys_clock_n: Signal<In, Clock>,
    reset: WireIn,
    pipe_in: PipeIn,
    pipe_out: PipeOut,
    delay_read: DFF<Bit>,
    leds: Signal<Out, Bits<8>>,
}

impl Default for OpalKellyDDR7Test {
    fn default() -> Self {
        Self {
            mcb: MCBInterface4GDDR3::xem_7010(),
            hi: OpalKellyHostInterface::xem_7010(),
            ok_host: OpalKellyHost::xem_7010(),
            ddr_fifo: DDR7FIFO::default(),
            sys_clock_p: Default::default(),
            sys_clock_n: Default::default(),
            reset: WireIn::new(0x00),
            pipe_in: PipeIn::new(0x80),
            pipe_out: PipeOut::new(0xA0),
            delay_read: Default::default(),
            leds: xem_7010_leds(),
        }
    }
}

impl Logic for OpalKellyDDR7Test {
    #[hdl_gen]
    fn update(&mut self) {
        // DDR and clocks
        self.mcb.link(&mut self.ddr_fifo.mcb);
        self.hi.link(&mut self.ok_host.hi);
        self.ddr_fifo.sys_clock_p.next = self.sys_clock_p.val();
        self.ddr_fifo.sys_clock_n.next = self.sys_clock_n.val();
        // Pipe in connects to DDR FIFO input
        self.ddr_fifo.data_in.next = self.pipe_in.dataout.val();
        self.ddr_fifo.write.next = self.pipe_in.write.val();
        self.ddr_fifo.write_clock.next = self.ok_host.ti_clk.val();
        // Pipe out connects to DDR FIFO output
        self.pipe_out.datain.next = self.ddr_fifo.data_out.val();
        self.delay_read.d.next = self.pipe_out.read.val();
        self.ddr_fifo.read.next = self.delay_read.q.val();
        self.ddr_fifo.read_clock.next = self.ok_host.ti_clk.val();
        // Connect the reset
        self.ddr_fifo.reset.next = self.reset.dataout.val().any();
        // Wire the OK busses
        self.reset.ok1.next = self.ok_host.ok1.val();
        self.pipe_in.ok1.next = self.ok_host.ok1.val();
        self.pipe_out.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.pipe_in.ok2.val() | self.pipe_out.ok2.val();
        self.delay_read.clk.next = self.ok_host.ti_clk.val();
        self.leds.next = !self.ddr_fifo.status.val();
    }
}

#[test]
fn test_synthesis_of_xem7010_ddr() {
    let mut uut = OpalKellyDDR7Test::default();
    uut.hi.link_connect_dest();
    uut.mcb.link_connect_dest();
    uut.sys_clock_p.connect();
    uut.sys_clock_n.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj_7010(uut, "xem_7010_ddr");
}

#[test]
fn test_opalkelly_xem_7010_ddr_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("xem_7010_ddr/top.bit")?;
    hnd.reset_firmware(0);
    let test_size = 1024 * 1024 * 10;
    let mut data = (0..test_size)
        .map(|_| rand::random::<u8>())
        .collect::<Vec<_>>();
    let now = Instant::now();
    hnd.write_to_pipe_in(0x80, &data).unwrap();
    let elapsed = Instant::now() - now;
    println!("Elapsed write time: {}", elapsed.as_micros());
    let mut out_data = vec![0; test_size];
    let now = Instant::now();
    hnd.read_from_pipe_out(0xA0, &mut out_data).unwrap();
    let elapsed = Instant::now() - now;
    println!(
        "Elapsed read time: {} -> {}",
        elapsed.as_micros(),
        (test_size as f64 * 8.0) / (elapsed.as_micros() as f64 / 1.0e6)
    );
    for i in 0..data.len() {
        assert_eq!(data[i], out_data[i]);
    }
    Ok(())
}

#[derive(LogicBlock)]
struct OpalKellyDownloadDDRFIFO7SeriesStressTest {
    mcb: MCBInterface4GDDR3,
    hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    download: OpalKellyDDRBackedDownloadFIFO7Series,
    count_in: DFF<Bits<32>>,
    sys_clock_p: Signal<In, Clock>,
    sys_clock_n: Signal<In, Clock>,
    strobe: Strobe<32>,
    will_write: Signal<Local, Bit>,
    reset: WireIn,
    enable: WireIn,
}

impl Default for OpalKellyDownloadDDRFIFO7SeriesStressTest {
    fn default() -> Self {
        Self {
            mcb: Default::default(),
            hi: OpalKellyHostInterface::xem_7010(),
            ok_host: OpalKellyHost::xem_7010(),
            download: OpalKellyDDRBackedDownloadFIFO7Series::new(0xA0),
            count_in: Default::default(),
            sys_clock_p: Default::default(),
            sys_clock_n: Default::default(),
            strobe: Strobe::new(48_000_000, 4_000_000.0),
            will_write: Default::default(),
            reset: WireIn::new(0x0),
            enable: WireIn::new(0x1),
        }
    }
}

impl Logic for OpalKellyDownloadDDRFIFO7SeriesStressTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        self.mcb.link(&mut self.download.mcb);
        self.download.reset.next = self.reset.dataout.val().any();
        self.download.sys_clock_p.next = self.sys_clock_p.val();
        self.download.sys_clock_n.next = self.sys_clock_n.val();
        self.download.ti_clk.next = self.ok_host.ti_clk.val();
        self.count_in.clk.next = self.ok_host.ti_clk.val();
        self.strobe.clock.next = self.ok_host.ti_clk.val();
        self.download.write_clock.next = self.ok_host.ti_clk.val();
        // Data source - counts on each strobe pulse and writes it to the input FIFO.
        self.will_write.next =
            self.strobe.strobe.val() & !self.download.full.val() & self.enable.dataout.val().any();
        self.count_in.d.next = self.count_in.q.val() + self.will_write.val();
        self.download.data_in.next = self.count_in.q.val();
        self.download.write.next = self.will_write.val();
        self.download.ok1.next = self.ok_host.ok1.val();
        self.reset.ok1.next = self.ok_host.ok1.val();
        self.enable.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.download.ok2.val();
        self.strobe.enable.next = self.enable.dataout.val().any();
    }
}

#[test]
fn test_opalkelly_xem_7010_ddr_stress_synth() {
    let mut uut = OpalKellyDownloadDDRFIFO7SeriesStressTest::default();
    uut.hi.link_connect_dest();
    uut.mcb.link_connect_dest();
    uut.sys_clock_p.connect();
    uut.sys_clock_n.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj_7010(uut, "xem_7010_ddr_stress");
}

#[test]
fn test_opalkelly_xem_7010_ddr_stress() -> Result<(), OkError> {
    test_opalkelly_ddr_stress_runtime("xem_7010_ddr_stress/top.bit")
}
