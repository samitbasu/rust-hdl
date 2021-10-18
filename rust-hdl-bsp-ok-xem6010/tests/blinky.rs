use rust_hdl_bsp_ok_xem6010::XEM6010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_test_ok_common::prelude::*;

#[cfg(feature = "fpga_hw_test")]
pub mod ddr;
mod download;
#[cfg(feature = "fpga_hw_test")]
pub mod mig;
mod mux_spi;
mod pipe;
mod spi;
mod wave;
mod wire;

#[test]
fn test_opalkelly_xem_6010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
    XEM6010::synth(uut, &target_path("xem_6010/blinky"));
}