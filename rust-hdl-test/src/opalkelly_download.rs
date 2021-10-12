use crate::ok_tools::ok_test_prelude;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::ok_download::{OpalKellyDownload32FIFO, OpalKellyDownloadFIFO};
use rust_hdl_ok::prelude::*;
use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, make_u32_buffer, OkError};
use rust_hdl_widgets::prelude::*;

#[derive(LogicBlock)]
pub struct OpalKellyDownload32FIFOTest {
    pub hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    dl: OpalKellyDownload32FIFO,
    counter: DFF<Bits<32>>,
    will_write: Signal<Local, Bit>,
}

impl Logic for OpalKellyDownload32FIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        self.dl.clock.next = self.ok_host.ti_clk.val();
        self.counter.clk.next = self.ok_host.ti_clk.val();
        self.dl.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.dl.ok2.val();
        self.dl.data_in.next = self.counter.q.val();
        self.will_write.next = !self.dl.full.val();
        self.counter.d.next =
            self.counter.q.val() + bit_cast::<32, 1>(self.will_write.val().into());
        self.dl.write.next = self.will_write.val();
    }
}

impl OpalKellyDownload32FIFOTest {
    fn new<B: OpalKellyBSP>() -> Self {
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            dl: OpalKellyDownload32FIFO::new(0xA0),
            counter: Default::default(),
            will_write: Default::default(),
        }
    }
}

#[test]
fn test_opalkelly_xem_6010_synth_download32() {
    let mut uut = OpalKellyDownload32FIFOTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    crate::ok_tools::synth_obj_6010(uut, "xem_6010_download32");
}

#[test]
fn test_opalkelly_xem_7010_synth_download32() {
    let mut uut = OpalKellyDownload32FIFOTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    crate::ok_tools::synth_obj_7010(uut, "xem_7010_download32");
}

#[cfg(test)]
fn test_opalkelly_download32_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
    // Read the data in 256*2 = 512 byte blocks
    let mut data = vec![0_u8; 1024 * 128];
    let mut last_val = 0;
    for iter in 0..50 {
        println!("Iteration {}", iter);
        hnd.read_from_block_pipe_out(0xA0, 256, &mut data).unwrap();
        let data_shorts = make_u16_buffer(&data);
        let data_words = make_u32_buffer(&data_shorts);
        for val in data_words {
            assert_eq!(((last_val as u128) & 0xFFFFFFFF_u128) as u32, val);
            last_val += 1;
        }
    }
    Ok(())
}

#[test]
fn test_opalkelly_xem_6010_download32_runtime() -> Result<(), OkError> {
    test_opalkelly_download32_runtime("xem_6010_download32/top.bit")
}

#[test]
fn test_opalkelly_xem_7010_download32_runtime() -> Result<(), OkError> {
    test_opalkelly_download32_runtime("xem_7010_download32/top.bit")
}

#[derive(LogicBlock)]
pub struct OpalKellyDownloadFIFOTest {
    pub hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    dl: OpalKellyDownloadFIFO,
    counter: DFF<Bits<16>>,
    will_write: Signal<Local, Bit>,
}

impl Logic for OpalKellyDownloadFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        self.dl.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.dl.ok2.val();
        self.dl.clock.next = self.ok_host.ti_clk.val();
        self.counter.clk.next = self.ok_host.ti_clk.val();
        self.will_write.next = !self.dl.data_full.val();
        self.counter.d.next =
            self.counter.q.val() + bit_cast::<16, 1>(self.will_write.val().into());
        self.dl.data_in.next = self.counter.q.val();
        self.dl.data_write.next = self.will_write.val();
    }
}

impl OpalKellyDownloadFIFOTest {
    fn new<B: OpalKellyBSP>() -> Self {
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            dl: OpalKellyDownloadFIFO::new(0xA0),
            counter: Default::default(),
            will_write: Default::default(),
        }
    }
}

#[test]
fn test_opalkelly_xem_6010_synth_download() {
    let mut uut = OpalKellyDownloadFIFOTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    crate::ok_tools::synth_obj_6010(uut, "xem_6010_download");
}

#[test]
fn test_opalkelly_xem_7010_synth_download() {
    let mut uut = OpalKellyDownloadFIFOTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    crate::ok_tools::synth_obj_7010(uut, "xem_7010_download");
}

#[cfg(test)]
fn test_opalkelly_download_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
    // Read the data in 256*2 = 512 byte blocks
    let mut data = vec![0_u8; 1024 * 128];
    hnd.read_from_block_pipe_out(0xA0, 256, &mut data).unwrap();
    let data_shorts = make_u16_buffer(&data);
    for (ndx, val) in data_shorts.iter().enumerate() {
        assert_eq!(((ndx as u128) & 0xFFFF_u128) as u16, *val);
    }
    Ok(())
}

#[test]
fn test_opalkelly_xem_6010_download_runtime() -> Result<(), OkError> {
    test_opalkelly_download_runtime("xem_6010_download/top.bit")
}

#[test]
fn test_opalkelly_xem_7010_download_runtime() -> Result<(), OkError> {
    test_opalkelly_download_runtime("xem_7010_download/top.bit")
}
