use rust_hdl::core::prelude::*;
use rust_hdl_ok_core::xem6010::*;

mod test_common;

use test_common::wave::*;

#[test]
fn test_opalkelly_xem_6010_synth_wave() {
    let mut uut = OpalKellyWave::new::<XEM6010>();
    uut.connect_all();
    synth::synth_obj(uut, target_path!("xem_6010/wave"));
}
