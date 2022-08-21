use rust_hdl::core::prelude::*;
use rust_hdl_bsp_ok_xem6010::xem6010::*;
use rust_hdl_ok_core::test_common::wave::OpalKellyWave;

#[test]
fn test_opalkelly_xem_6010_synth_wave() {
    let mut uut = OpalKellyWave::new::<XEM6010>();
    uut.connect_all();
    synth::synth_obj(uut, target_path!("xem_6010/wave"));
}
