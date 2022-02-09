use std::thread::sleep;
use std::time::{Duration, Instant};

mod test_common;

#[cfg(feature = "frontpanel")]
use test_common::ddr::*;

use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::bsp::ok_xem7010::download::OpalKellyDDRBackedDownloadFIFO7Series;
use rust_hdl::bsp::ok_xem7010::mcb_if::MCBInterface4GDDR3;
use rust_hdl::bsp::ok_xem7010::XEM7010;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

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
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        MCBInterface4GDDR3::link(&mut self.mcb, &mut self.download.mcb);
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

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_7010_ddr_stress_synth() {
    let mut uut = OpalKellyDownloadDDRFIFO7SeriesStressTest::default();
    uut.hi.link_connect_dest();
    uut.mcb.link_connect_dest();
    uut.sys_clock_p.connect();
    uut.sys_clock_n.connect();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/ddr_stress"));
    test_opalkelly_ddr_stress_runtime(target_path!("xem_7010/ddr_stress/top.bit")).unwrap()
}
