use rust_hdl::prelude::*;
use rust_hdl_bsp_ok_xem6010::xem6010::{synth, XEM6010};
use rust_hdl_ok_core::test_common::wire::{test_opalkelly_xem_wire_runtime, OpalKellyWireTest};

#[test]
fn test_opalkelly_xem_6010_synth_wire() {
    let mut uut = OpalKellyWireTest::new::<XEM6010>();
    uut.connect_all();
    synth::synth_obj(uut, target_path!("xem_6010/wire"));
    test_opalkelly_xem_wire_runtime(
        target_path!("xem_6010/wire/top.bit"),
        env!("XEM6010_SERIAL"),
    )
    .unwrap()
}
