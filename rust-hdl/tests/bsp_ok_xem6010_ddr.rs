mod test_common;

use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::bsp::ok_xem6010::mcb_if::MCBInterface1GDDR2;
use rust_hdl::bsp::ok_xem6010::ok_download_ddr::OpalKellyDDRBackedDownloadFIFO;
use rust_hdl::bsp::ok_xem6010::pins::xem_6010_base_clock;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct OpalKellyDownloadDDRFIFOStressTest {
    mcb: MCBInterface1GDDR2,
    hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    download: OpalKellyDDRBackedDownloadFIFO,
    count_in: DFF<Bits<32>>,
    raw_sys_clock: Signal<In, Clock>,
    strobe: Strobe<32>,
    will_write: Signal<Local, Bit>,
    reset: WireIn,
    enable: WireIn,
}

impl Default for OpalKellyDownloadDDRFIFOStressTest {
    fn default() -> Self {
        Self {
            mcb: MCBInterface1GDDR2::xem_6010(),
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::xem_6010(),
            download: OpalKellyDDRBackedDownloadFIFO::new(0xA0),
            count_in: Default::default(),
            raw_sys_clock: xem_6010_base_clock(),
            strobe: Strobe::new(48_000_000, 4_000_000.0),
            will_write: Default::default(),
            reset: WireIn::new(0x0),
            enable: WireIn::new(0x1),
        }
    }
}

impl Logic for OpalKellyDownloadDDRFIFOStressTest {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        MCBInterface1GDDR2::link(&mut self.mcb, &mut self.download.mcb);
        self.download.reset.next = self.reset.dataout.val().any();
        self.download.raw_sys_clock.next = self.raw_sys_clock.val();
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
fn test_opalkelly_xem_6010_synth_ddr_stress() {
    let mut uut = OpalKellyDownloadDDRFIFOStressTest::default();
    uut.hi.link_connect_dest();
    uut.mcb.link_connect_dest();
    uut.raw_sys_clock.connect();
    uut.connect_all();
    rust_hdl::bsp::ok_xem6010::synth::synth_obj(uut, target_path!("xem_6010/ddr_stress"));
    test_common::ddr::test_opalkelly_ddr_stress_runtime(target_path!("xem_6010/ddr_stress/top.bit"))
        .unwrap()
}
