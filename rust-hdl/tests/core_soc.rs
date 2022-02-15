use test_common::soc::SoCTestChip;
use rust_hdl::core::prelude::*;

mod test_common;

#[test]
fn test_soc_test_chip_synthesizes() {
    let mut uut = SoCTestChip::default();
    uut.sys_clock.connect();
    uut.clock.connect();
    uut.from_cpu.write.connect();
    uut.from_cpu.data.connect();
    uut.to_cpu.read.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("soc_test", &vlog).unwrap();
}
