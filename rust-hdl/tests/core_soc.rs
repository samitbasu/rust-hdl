use rust_hdl::core::prelude::*;
use rust_hdl::widgets::test_helpers::SoCTestChip;

#[test]
fn test_soc_test_chip_synthesizes() {
    let mut uut = SoCTestChip::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("soc_test", &vlog).unwrap();
}
