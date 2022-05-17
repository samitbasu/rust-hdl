
use rust_hdl_ok_core::xem6010::XEM6010;
use rust_hdl::core::prelude::*;

mod test_common;

use test_common::download::*;


#[test]
fn test_opalkelly_xem_6010_synth_download32() {
    let mut uut = OpalKellyDownload32FIFOTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_ok_core::xem6010::synth::synth_obj(uut, target_path!("xem_6010/download32"));
    test_opalkelly_download32_runtime(target_path!("xem_6010/download32/top.bit")).unwrap();
}
