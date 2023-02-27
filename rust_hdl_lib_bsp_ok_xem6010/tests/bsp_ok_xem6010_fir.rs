use rust_hdl_lib_ok_core::core::prelude::*;

use rust_hdl::prelude::*;
use rust_hdl_lib_bsp_ok_xem6010::xem6010::XEM6010;
use rust_hdl_lib_ok_core::test_common::fir::{test_opalkelly_fir_runtime, OpalKellyFIRTest};

#[test]
fn test_opalkelly_xem_6010_fir() {
    let mut uut = OpalKellyFIRTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/fir"));
    test_opalkelly_fir_runtime(target_path!("xem_6010/fir/top.bit"), env!("XEM6010_SERIAL"))
        .unwrap();
}
