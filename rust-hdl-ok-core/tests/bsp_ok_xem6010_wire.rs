use rust_hdl::core::prelude::*;
use rust_hdl_ok_core::xem6010::*;

mod test_common;

use test_common::wire::*;

#[test]
fn test_opalkelly_xem_6010_synth_wire() {
    let mut uut = OpalKellyWireTest::new::<XEM6010>();
    uut.connect_all();
    synth::synth_obj(uut, target_path!("xem_6010/wire"));
    test_opalkelly_xem_wire_runtime(target_path!("xem_6010/wire/top.bit")).unwrap()
}
