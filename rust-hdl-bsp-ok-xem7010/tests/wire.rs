use rust_hdl_ok_frontpanel_sys::OkError;

#[test]
fn test_opalkelly_xem_7010_synth_wire() {
    let mut uut = OpalKellyWireTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_7010(uut, "xem_7010_wire");
}

#[test]
fn test_opalkelly_xem_7010_wire_runtime() -> Result<(), OkError> {
    wire::test_opalkelly_xem_wire_runtime("xem_7010_wire/top.bit")
}
