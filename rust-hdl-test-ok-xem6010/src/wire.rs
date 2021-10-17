use rust_hdl_ok_frontpanel_sys::OkError;

#[test]
fn test_opalkelly_xem_6010_synth_wire() {
    let mut uut = OpalKellyWireTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "xem_6010_wire");
}

#[test]
fn test_opalkelly_xem_6010_wire_runtime() -> Result<(), OkError> {
    wire::test_opalkelly_xem_wire_runtime("xem_6010_wire/top.bit")
}