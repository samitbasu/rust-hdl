#[cfg(feature = "frontpanel")]
use rust_hdl::bsp::ok_xem6010::*;
use rust_hdl::core::prelude::*;

mod test_common;
#[cfg(feature = "frontpanel")]
use test_common::wave::*;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_6010_synth_wave() {
    let mut uut = OpalKellyWave::new::<XEM6010>();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_inout.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    synth::synth_obj(uut, target_path!("xem_6010/wave"));
}
