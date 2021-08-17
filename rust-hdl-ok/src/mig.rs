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

// Wrapper around the MIG infrastructure parts
#[derive(LogicBlock, Default)]
pub struct DDR2Core {}

impl Logic for DDR2Core {
    fn update(&mut self) {}

    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(BlackBox {
            code: r#"
(* blackbox *)
module ddr2  (
   inout  [16-1:0]                      mcb3_dram_dq,
   output [13-1:0]                   mcb3_dram_a,
   output [3-1:0]               mcb3_dram_ba,
   output                                           mcb3_dram_ras_n,
   output                                           mcb3_dram_cas_n,
   output                                           mcb3_dram_we_n,
   output                                           mcb3_dram_odt,
   output                                           mcb3_dram_cke,
   output                                           mcb3_dram_dm,
   inout                                            mcb3_dram_udqs,
   inout                                            mcb3_dram_udqs_n,
   inout                                            mcb3_rzq,
   inout                                            mcb3_zio,
   output                                           mcb3_dram_udm,
   input                                            c3_sys_clk_p,
   input                                            c3_sys_rst_n,
   output                                           c3_calib_done,
   output                                           c3_clk0,
   output                                           c3_rst0,
   inout                                            mcb3_dram_dqs,
   inout                                            mcb3_dram_dqs_n,
   output                                           mcb3_dram_ck,
   output                                           mcb3_dram_ck_n,
      input		c3_p0_cmd_clk,
      input		c3_p0_cmd_en,
      input [2:0]	c3_p0_cmd_instr,
      input [5:0]	c3_p0_cmd_bl,
      input [29:0]	c3_p0_cmd_byte_addr,
      output		c3_p0_cmd_empty,
      output		c3_p0_cmd_full,
      input		c3_p0_wr_clk,
      input		c3_p0_wr_en,
      input [4 - 1:0]	c3_p0_wr_mask,
      input [32 - 1:0]	c3_p0_wr_data,
      output		c3_p0_wr_full,
      output		c3_p0_wr_empty,
      output [6:0]	c3_p0_wr_count,
      output		c3_p0_wr_underrun,
      output		c3_p0_wr_error,
      input		c3_p0_rd_clk,
      input		c3_p0_rd_en,
      output [32 - 1:0]	c3_p0_rd_data,
      output		c3_p0_rd_full,
      output		c3_p0_rd_empty,
      output [6:0]	c3_p0_rd_count,
      output		c3_p0_rd_overflow,
      output		c3_p0_rd_error
   );
endmodule
            "#
            .into(),
            name: "ddr2".into(),
        })
    }
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
    // DDR2 Core
    pub ddr2: DDR2Core,
}

impl Logic for MemoryInterfaceGenerator {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.calib_done.connect();
        self.clk_out.connect();
        self.reset_out.connect();
        self.p0_cmd.empty.connect();
        self.p0_cmd.full.connect();
        self.p0_wr.empty.connect();
        self.p0_wr.full.connect();
        self.p0_wr.count.connect();
        self.p0_wr.error.connect();
        self.p0_wr.underrun.connect();
        self.p0_rd.error.connect();
        self.p0_rd.count.connect();
        self.p0_rd.full.connect();
        self.p0_rd.empty.connect();
        self.p0_rd.data.connect();
        self.p0_rd.overflow.connect();
        self.mcb.link_connect();
    }
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
   .mcb3_dram_dq(mcb_data_bus),
   .mcb3_dram_a(mcb_address),
   .mcb3_dram_ba(mcb_bank_select),
   .mcb3_dram_ras_n(mcb_row_address_strobe_not),
   .mcb3_dram_cas_n(mcb_column_address_strobe_not),
   .mcb3_dram_we_n(mcb_write_enable_not),
   .mcb3_dram_odt(mcb_on_die_termination),
   .mcb3_dram_cke(mcb_clock_enable),
   .mcb3_dram_dm(mcb_data_mask),
   .mcb3_dram_udqs(mcb_upper_byte_data_strobe),
   .mcb3_dram_udqs_n(mcb_upper_byte_data_strobe_neg),
   .mcb3_rzq(mcb_rzq),
   .mcb3_zio(mcb_zio),
   .mcb3_dram_udm(mcb_upper_data_mask),
   .mcb3_dram_dqs(mcb_data_strobe_signal),
   .mcb3_dram_dqs_n(mcb_data_strobe_signal_neg),
   .mcb3_dram_ck(mcb_dram_clock),
   .mcb3_dram_ck_n(mcb_dram_clock_neg),
   .c3_sys_clk_p(raw_sys_clk),
   .c3_sys_rst_n(reset_n),
   .c3_calib_done(calib_done),
   .c3_clk0(clk_out),
   .c3_rst0(reset_out),
   .c3_p0_cmd_clk(p0_cmd_clock),
   .c3_p0_cmd_en(p0_cmd_enable),
   .c3_p0_cmd_instr(p0_cmd_instruction),
   .c3_p0_cmd_bl(p0_cmd_burst_length),
   .c3_p0_cmd_byte_addr(p0_cmd_byte_address),
   .c3_p0_cmd_empty(p0_cmd_empty),
   .c3_p0_cmd_full(p0_cmd_full),
   .c3_p0_wr_clk(p0_wr_clock),
   .c3_p0_wr_en(p0_wr_enable),
   .c3_p0_wr_mask(p0_wr_mask),
   .c3_p0_wr_data(p0_wr_data),
   .c3_p0_wr_full(p0_wr_full),
   .c3_p0_wr_empty(p0_wr_empty),
   .c3_p0_wr_count(p0_wr_count),
   .c3_p0_wr_underrun(p0_wr_underrun),
   .c3_p0_wr_error(p0_wr_error),
   .c3_p0_rd_clk(p0_rd_clock),
   .c3_p0_rd_en(p0_rd_enable),
   .c3_p0_rd_data(p0_rd_data),
   .c3_p0_rd_full(p0_rd_full),
   .c3_p0_rd_empty(p0_rd_empty),
   .c3_p0_rd_count(p0_rd_count),
   .c3_p0_rd_overflow(p0_rd_overflow),
   .c3_p0_rd_error(p0_rd_error));
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
