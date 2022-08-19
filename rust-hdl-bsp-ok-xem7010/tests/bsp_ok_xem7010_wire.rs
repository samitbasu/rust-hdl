use rust_hdl_ok_core::core::prelude::*;
use rust_hdl::core::prelude::*;
use rust_hdl_bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl_ok_core::test_common::wire::{OpalKellyWireTest, test_opalkelly_xem_wire_runtime};

#[test]
fn test_opalkelly_xem_7010_synth_wire() {
    let mut uut = OpalKellyWireTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/wire"));
    test_opalkelly_xem_wire_runtime(target_path!("xem_7010/wire/top.bit")).unwrap();
}
