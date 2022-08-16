use rust_hdl::core::prelude::*;
use rust_hdl_bsp_alchitry_cu::synth;
use rust_hdl_fpga_support::lattice::ice40::ice_pll::ICE40PLLBlock;

#[test]
fn test_pll_synthesizable() {
    const MHZ100: u64 = 100_000_000;
    const MHZ25: u64 = 25_000_000;
    let mut uut: ICE40PLLBlock<MHZ100, MHZ25> = ICE40PLLBlock::default();
    uut.clock_in.add_location(0, "P7");
    uut.clock_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("vlog", &vlog).unwrap();
    synth::generate_bitstream(uut, target_path!("alchitry_cu/pll_cu"));
}
