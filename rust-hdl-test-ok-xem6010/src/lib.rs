mod blinky;
mod download;
#[cfg(feature = "fpga_hw_test")]
pub mod opalkelly_xem_6010_ddr;
#[cfg(feature = "fpga_hw_test")]
pub mod opalkelly_xem_6010_mig;
mod mux_spi;
mod pipe;
mod spi;
mod wave;
mod wire;
