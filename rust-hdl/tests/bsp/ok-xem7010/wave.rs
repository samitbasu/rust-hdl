use rust_hdl_bsp_ok_xem7010::XEM7010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_test_core::target_path;
use rust_hdl_test_ok_common::prelude::*;

#[test]
fn test_opalkelly_xem_7010_synth_wave() {
    let mut uut = OpalKellyWave::new::<XEM7010>();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_inout.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/wave"));
}
