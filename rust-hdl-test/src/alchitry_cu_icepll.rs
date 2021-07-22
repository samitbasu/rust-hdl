use rust_hdl_alchitry_cu::ice_pll::ICE40PLLBlock;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;


#[test]
fn test_pll_synthesizable() {
    let mut uut = ICE40PLLBlock::new(100.0, 25.0);
    uut.clock_in.add_location(0, "P7");
    uut.clock_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("vlog", &vlog).unwrap();
    println!("{}", vlog);
    rust_hdl_alchitry_cu::synth::generate_bitstream(uut, "pll_cu");
}