use rust_hdl_core::check_connected::check_connected;

#[test]
fn test_opalkelly_xem_6010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "xem_6010_blinky");
}
