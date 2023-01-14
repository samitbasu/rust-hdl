pub mod bidi;
pub mod bridge;
pub mod bus;
pub mod controller;
pub mod cross_fifo;
pub mod expander;
pub mod fifo;
pub mod fifo_linker;
pub mod host;
pub mod miso_fifo_port;
pub mod miso_port;
pub mod miso_wide_port;
pub mod mosi_fifo_port;
pub mod mosi_port;
pub mod mosi_wide_port;
pub mod prelude;
pub mod reducer;
pub mod router;
pub mod router_rom;
pub mod sdram_controller;
pub mod sdram_controller_tester;
pub mod sdram_fifo;
pub mod sim;
pub mod spi;
pub mod test_helpers;
pub trait HLSNamedPorts {
    fn ports(&self) -> Vec<String>;
}
