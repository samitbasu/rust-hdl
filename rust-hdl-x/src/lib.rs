use ruint::{aliases::U32, uint};

mod bit_iter;
mod bit_slice;
mod counter;
mod derive_vcd;
mod pulser;
mod shot;
mod spi_controller;
mod strobe;
mod synchronous;
mod tracer;
mod vcd;

#[test]
fn bits_benchmark() {
    let tic = std::time::Instant::now();
    let x = rust_hdl::core::bits::Bits::<65>::from(0x12345678);
    let y = rust_hdl::core::bits::Bits::<65>::from(0x1);
    let mut z = rust_hdl::core::bits::Bits::<65>::from(0x0);
    for i in 0..1000000 {
        let _ = x.get_bit(i % 32);
        let _ = y.get_bit(i % 32);
        z = z + y;
    }
    let toc = std::time::Instant::now();
    println!("Time to run bit benchmark: {:?}", toc - tic);
}

#[test]
fn uint_benchmark() {
    let tic = std::time::Instant::now();
    let x = uint!(0x12345678_U65);
    let y = uint!(0x1_U65);
    let mut z = uint!(0x0_U65);
    for i in 0..1000000 {
        let _ = x.bit(i % 32);
        let _ = y.bit(i % 32);
        z += y;
    }
    let toc = std::time::Instant::now();
    println!("Time to run uint benchmark: {:?}", toc - tic);
}
