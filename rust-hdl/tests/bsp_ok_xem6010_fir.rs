mod test_common;

use rust_hdl::bsp::ok_xem6010::XEM6010;
#[cfg(feature = "frontpanel")]
use test_common::fir::*;
use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::core::prelude::*;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_6010_fir() {
    let mut uut = OpalKellyFIRTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
//    XEM6010::synth(uut, target_path!("xem_6010/fir"));
    test_opalkelly_fir_runtime(target_path!("xem_6010/fir/top.bit")).unwrap();
}