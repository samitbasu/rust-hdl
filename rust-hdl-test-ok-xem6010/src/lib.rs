mod blinky;
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
