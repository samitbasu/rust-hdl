mod test_common;

use rust_hdl_ok_core::core::prelude::*;

use rust_hdl::core::prelude::*;
use rust_hdl_ok_core::xem6010::XEM6010;

use test_common::fir::*;

#[test]
fn test_opalkelly_xem_6010_fir() {
    let mut uut = OpalKellyFIRTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/fir"));
    test_opalkelly_fir_runtime(target_path!("xem_6010/fir/top.bit")).unwrap();
}
