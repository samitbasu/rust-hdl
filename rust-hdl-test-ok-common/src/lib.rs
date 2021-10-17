#[cfg(feature = "fpga_hw_test")]
pub mod tools;
mod blinky;
mod download;
mod opalkelly_mux_spi;
mod pipe;
mod spi;
mod wave;
mod wire;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
