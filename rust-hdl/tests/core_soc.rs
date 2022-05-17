use rust_hdl::core::prelude::*;
use test_common::soc::SoCTestChip;

mod test_common;

#[test]
fn test_soc_test_chip_synthesizes() {
    let mut uut = SoCTestChip::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("soc_test", &vlog).unwrap();
}
