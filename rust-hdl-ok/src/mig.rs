use crate::mcb_if::MCBInterface;
use rust_hdl_core::prelude::*;

#[derive(LogicInterface, Default)]
pub struct CommandPort {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub instruction: Signal<In, Bits<3>>,
    pub burst_length: Signal<In, Bits<6>>,
    pub byte_address: Signal<In, Bits<30>>,
    pub empty: Signal<Out, Bit>,
    pub full: Signal<Out, Bit>,
}

#[derive(LogicInterface, Default)]
pub struct WritePort {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub mask: Signal<In, Bits<4>>,
    pub data: Signal<In, Bits<32>>,
    pub full: Signal<Out, Bit>,
    pub empty: Signal<Out, Bit>,
    pub count: Signal<Out, Bits<7>>,
    pub underrun: Signal<Out, Bit>,
    pub error: Signal<Out, Bit>,
}

#[derive(LogicInterface, Default)]
pub struct ReadPort {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub data: Signal<Out, Bits<32>>,
    pub full: Signal<Out, Bit>,
    pub empty: Signal<Out, Bit>,
    pub count: Signal<Out, Bits<7>>,
    pub overflow: Signal<Out, Bit>,
    pub error: Signal<Out, Bit>,
}

#[derive(LogicBlock, Default)]
pub struct MemoryInterfaceGenerator {
    // Raw clock from the system - cannot be intercepted
    pub raw_sys_clk: Signal<In, Clock>,
    // Reset - must be handled externally
    pub reset_n: Signal<In, Bit>,
    // Calibration complete
    pub calib_done: Signal<Out, Bit>,
    // Buffered 100 MHz clock
    pub clk_out: Signal<Out, Clock>,
    // Delayed reset
    pub reset_out: Signal<Out, Bit>,
    // P0 command port
    pub p0_cmd: CommandPort,
    // P0 write port
    pub p0_wr: WritePort,
    // P0 read port
    pub p0_rd: ReadPort,
    // MCB interface
    pub mcb: MCBInterface,
}

impl Logic for MemoryInterfaceGenerator {
    fn update(&mut self) {}
    fn hdl(&self) -> Verilog {
        Verilog::Custom(
            r#"
// Instantiate the DDR2 module, which is a slightly modified
// version of the code-gen ddr2 module from MIG.  This is a
// top level wrapper that instantiates all the lower level
// pieces.  The only difference to the core-gen version is
// that the clock is not differential, but single ended.
// At least on the XEM6010. YMMV.
ddr2 ddr2_tb(
   .mcb3_dram_dq(mcb_dq),
   .mcb3_dram_a(mcb_a),
   .mcb3_dram_ba(mcb_ba),
   .mcb3_dram_ras_n(mcb_n),
   .mcb3_dram_cas_n(mcb_n),
   .mcb3_dram_we_n(mcb_n),
   .mcb3_dram_odt(mcb_odt),
   .mcb3_dram_cke(mcb_cke),
   .mcb3_dram_dm(mcb_dm),
   .mcb3_dram_udqs(mcb_udqs),
   .mcb3_dram_udqs_n(mcb_n),
   .mcb3_rzq(mcb_rzq),
   .mcb3_zio(mcb_zio),
   .mcb3_dram_udm(mcb_udm),
   .mcb3_dram_dqs(mcb_dqs),
   .mcb3_dram_dqs_n(mcb_n),
   .mcb3_dram_ck(mcb_ck),
   .mcb3_dram_ck_n(mcb_n),
   .c3_sys_clk_p(raw_sys_clk),
   .c3_sys_rst_n(reset_n),
   .c3_calib_done(calib_done),
   .c3_clk0(clk_out),
   .c3_rst0(reset_out),
   .c3_p0_cmd_clk(p0_cmd_clk),
   .c3_p0_cmd_en(p0_cmd_en),
   .c3_p0_cmd_instr(p0_cmd_instr),
   .c3_p0_cmd_bl(p0_cmd_bl),
   .c3_p0_cmd_byte_addr(p0_cmd_byte_addr),
   .c3_p0_cmd_empty(p0_cmd_empty),
   .c3_p0_cmd_full(p0_cmd_full),
   .c3_p0_wr_clk(p0_wr_clk),
   .c3_p0_wr_en(p0_wr_en),
   .c3_p0_wr_mask(p0_wr_mask),
   .c3_p0_wr_data(p0_wr_data),
   .c3_p0_wr_full(p0_wr_full),
   .c3_p0_wr_empty(p0_wr_empty),
   .c3_p0_wr_count(p0_wr_count),
   .c3_p0_wr_underrun(p0_wr_underrun),
   .c3_p0_wr_error(p0_wr_error),
   .c3_p0_rd_clk(p0_rd_clk),
   .c3_p0_rd_en(p0_rd_en),
   .c3_p0_rd_data(p0_rd_data),
   .c3_p0_rd_full(p0_rd_full),
   .c3_p0_rd_empty(p0_rd_empty),
   .c3_p0_rd_count(p0_rd_count),
   .c3_p0_rd_overflow(p0_rd_overflow),
   .c3_p0_rd_error(p0_rd_error));
endmodule
        "#
            .into(),
        )
    }
}

#[test]
fn test_mig_gen() {
    let mig = MemoryInterfaceGenerator::default();
    let vlog = generate_verilog_unchecked(&mig);
    println!("{}", vlog);
}
