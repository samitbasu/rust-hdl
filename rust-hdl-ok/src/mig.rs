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

#[derive(LogicBlock)]
pub struct MemoryInterfaceGenerator {
    // Raw clock from the system - cannot be intercepted
    pub raw_sys_clk: Signal<In, Clock>,
    // Reset - must be handled externally
    pub reset: Signal<In, Bit>,
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

// TODO - currently assumes the MIG is at the top level of the
// Verilog structure, and that the object is named "mig".  Generalizing
// this would be a good idea.
fn add_mig_timing_constraint(raw_sys_clk: &mut Signal<In, Clock>) {
    raw_sys_clk.add_constraint(PinConstraint {
        index: 0,
        constraint: Constraint::Timing(Timing::Custom(r#"NET "*selfrefresh_mcb_mode" TIG"#.into())),
    });
    raw_sys_clk.add_constraint(PinConstraint {
        index: 0,
        constraint: Constraint::Timing(Timing::Custom(r#"NET "*pll_lock" TIG"#.into())),
    });
    raw_sys_clk.add_constraint(PinConstraint {
        index: 0,
        constraint: Constraint::Timing(Timing::Custom(r#"NET "*CKE_Train" TIG"#.into())),
    });
}

impl Default for MemoryInterfaceGenerator {
    fn default() -> Self {
        let mut raw_sys_clk = Signal::default();
        add_mig_timing_constraint(&mut raw_sys_clk);
        Self {
            raw_sys_clk,
            reset: Default::default(),
            calib_done: Default::default(),
            clk_out: Default::default(),
            reset_out: Default::default(),
            p0_cmd: Default::default(),
            p0_wr: Default::default(),
            p0_rd: Default::default(),
            mcb: Default::default(),
        }
    }
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
        self.mcb.link_connect_source();
        self.mcb.link_connect_dest();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(BlackBox {
            code: r##"
module MemoryInterfaceGenerator(raw_sys_clk,reset,calib_done,clk_out,reset_out,
p0_cmd$clock,p0_cmd$enable,p0_cmd$instruction,p0_cmd$burst_length,p0_cmd$byte_address,
p0_cmd$empty,p0_cmd$full,p0_wr$clock,p0_wr$enable,p0_wr$mask,p0_wr$data,p0_wr$full,
p0_wr$empty,p0_wr$count,p0_wr$underrun,p0_wr$error,p0_rd$clock,p0_rd$enable,p0_rd$data,
p0_rd$full,p0_rd$empty,p0_rd$count,p0_rd$overflow,p0_rd$error,mcb$data_bus,mcb$address,
mcb$bank_select,mcb$row_address_strobe_not,mcb$column_address_strobe_not,mcb$write_enable_not,
mcb$on_die_termination,mcb$clock_enable,mcb$data_mask,mcb$upper_byte_data_strobe,
mcb$upper_byte_data_strobe_neg,mcb$rzq,mcb$zio,mcb$upper_data_mask,mcb$data_strobe_signal,
mcb$data_strobe_signal_neg,mcb$dram_clock,mcb$dram_clock_neg,mcb$chip_select_neg);

    // Module arguments
    input raw_sys_clk;
    input reset;
    output calib_done;
    output clk_out;
    output reset_out;
    input p0_cmd$clock;
    input p0_cmd$enable;
    input [2:0] p0_cmd$instruction;
    input [5:0] p0_cmd$burst_length;
    input [29:0] p0_cmd$byte_address;
    output p0_cmd$empty;
    output p0_cmd$full;
    input p0_wr$clock;
    input p0_wr$enable;
    input [3:0] p0_wr$mask;
    input [31:0] p0_wr$data;
    output p0_wr$full;
    output p0_wr$empty;
    output [6:0] p0_wr$count;
    output p0_wr$underrun;
    output p0_wr$error;
    input p0_rd$clock;
    input p0_rd$enable;
    output [31:0] p0_rd$data;
    output p0_rd$full;
    output p0_rd$empty;
    output [6:0] p0_rd$count;
    output p0_rd$overflow;
    output p0_rd$error;
    inout [15:0] mcb$data_bus;
    output [12:0] mcb$address;
    output [2:0] mcb$bank_select;
    output mcb$row_address_strobe_not;
    output mcb$column_address_strobe_not;
    output mcb$write_enable_not;
    output mcb$on_die_termination;
    output mcb$clock_enable;
    output mcb$data_mask;
    output mcb$upper_byte_data_strobe;
    output mcb$upper_byte_data_strobe_neg;
    inout mcb$rzq;
    inout mcb$zio;
    output mcb$upper_data_mask;
    inout mcb$data_strobe_signal;
    inout mcb$data_strobe_signal_neg;
    output mcb$dram_clock;
    output mcb$dram_clock_neg;
    output mcb$chip_select_neg;

    // Memory Interface Block
    /* These are constants for our generated memory interface */
       localparam C3_P0_MASK_SIZE           = 4;
       localparam C3_P0_DATA_PORT_SIZE      = 32;
       localparam C3_P1_MASK_SIZE           = 4;
       localparam C3_P1_DATA_PORT_SIZE      = 32;
       localparam DEBUG_EN                  = 0;
       localparam C3_MEMCLK_PERIOD          = 3200;
       localparam C3_CALIB_SOFT_IP          = "TRUE";
       localparam C3_SIMULATION             = "FALSE";
       localparam C3_HW_TESTING             = "FALSE";
       localparam C3_RST_ACT_LOW            = 0;
       localparam C3_INPUT_CLK_TYPE         = "DIFFERENTIAL";
       localparam C3_MEM_ADDR_ORDER         = "ROW_BANK_COLUMN";
       localparam C3_NUM_DQ_PINS            = 16;
       localparam C3_MEM_ADDR_WIDTH         = 13;
       localparam C3_MEM_BANKADDR_WIDTH     = 3;
       localparam C3_CLKOUT0_DIVIDE       = 1;
       localparam C3_CLKOUT1_DIVIDE       = 1;
       localparam C3_CLKOUT2_DIVIDE       = 16;
       localparam C3_CLKOUT3_DIVIDE       = 8;
       localparam C3_CLKFBOUT_MULT        = 2;
       localparam C3_DIVCLK_DIVIDE        = 1;
       localparam C3_ARB_NUM_TIME_SLOTS   = 12;
       localparam C3_ARB_TIME_SLOT_0      = 3'o0;
       localparam C3_ARB_TIME_SLOT_1      = 3'o0;
       localparam C3_ARB_TIME_SLOT_2      = 3'o0;
       localparam C3_ARB_TIME_SLOT_3      = 3'o0;
       localparam C3_ARB_TIME_SLOT_4      = 3'o0;
       localparam C3_ARB_TIME_SLOT_5      = 3'o0;
       localparam C3_ARB_TIME_SLOT_6      = 3'o0;
       localparam C3_ARB_TIME_SLOT_7      = 3'o0;
       localparam C3_ARB_TIME_SLOT_8      = 3'o0;
       localparam C3_ARB_TIME_SLOT_9      = 3'o0;
       localparam C3_ARB_TIME_SLOT_10     = 3'o0;
       localparam C3_ARB_TIME_SLOT_11     = 3'o0;
       localparam C3_MEM_TRAS             = 40000;
       localparam C3_MEM_TRCD             = 15000;
       localparam C3_MEM_TREFI            = 7800000;
       localparam C3_MEM_TRFC             = 127500;
       localparam C3_MEM_TRP              = 15000;
       localparam C3_MEM_TWR              = 15000;
       localparam C3_MEM_TRTP             = 7500;
       localparam C3_MEM_TWTR             = 7500;
       localparam C3_MEM_TYPE             = "DDR2";
       localparam C3_MEM_DENSITY          = "1Gb";
       localparam C3_MEM_BURST_LEN        = 4;
       localparam C3_MEM_CAS_LATENCY      = 5;
       localparam C3_MEM_NUM_COL_BITS     = 10;
       localparam C3_MEM_DDR1_2_ODS       = "FULL";
       localparam C3_MEM_DDR2_RTT         = "50OHMS";
       localparam C3_MEM_DDR2_DIFF_DQS_EN  = "YES";
       localparam C3_MEM_DDR2_3_PA_SR     = "FULL";
       localparam C3_MEM_DDR2_3_HIGH_TEMP_SR  = "NORMAL";
       localparam C3_MEM_DDR3_CAS_LATENCY  = 6;
       localparam C3_MEM_DDR3_ODS         = "DIV6";
       localparam C3_MEM_DDR3_RTT         = "DIV2";
       localparam C3_MEM_DDR3_CAS_WR_LATENCY  = 5;
       localparam C3_MEM_DDR3_AUTO_SR     = "ENABLED";
       localparam C3_MEM_DDR3_DYN_WRT_ODT  = "OFF";
       localparam C3_MEM_MOBILE_PA_SR     = "FULL";
       localparam C3_MEM_MDDR_ODS         = "FULL";
       localparam C3_MC_CALIB_BYPASS      = "NO";
       localparam C3_MC_CALIBRATION_MODE  = "CALIBRATION";
       localparam C3_MC_CALIBRATION_DELAY  = "HALF";
       localparam C3_SKIP_IN_TERM_CAL     = 0;
       localparam C3_SKIP_DYNAMIC_CAL     = 0;
       localparam C3_LDQSP_TAP_DELAY_VAL  = 0;
       localparam C3_LDQSN_TAP_DELAY_VAL  = 0;
       localparam C3_UDQSP_TAP_DELAY_VAL  = 0;
       localparam C3_UDQSN_TAP_DELAY_VAL  = 0;
       localparam C3_DQ0_TAP_DELAY_VAL    = 0;
       localparam C3_DQ1_TAP_DELAY_VAL    = 0;
       localparam C3_DQ2_TAP_DELAY_VAL    = 0;
       localparam C3_DQ3_TAP_DELAY_VAL    = 0;
       localparam C3_DQ4_TAP_DELAY_VAL    = 0;
       localparam C3_DQ5_TAP_DELAY_VAL    = 0;
       localparam C3_DQ6_TAP_DELAY_VAL    = 0;
       localparam C3_DQ7_TAP_DELAY_VAL    = 0;
       localparam C3_DQ8_TAP_DELAY_VAL    = 0;
       localparam C3_DQ9_TAP_DELAY_VAL    = 0;
       localparam C3_DQ10_TAP_DELAY_VAL   = 0;
       localparam C3_DQ11_TAP_DELAY_VAL   = 0;
       localparam C3_DQ12_TAP_DELAY_VAL   = 0;
       localparam C3_DQ13_TAP_DELAY_VAL   = 0;
       localparam C3_DQ14_TAP_DELAY_VAL   = 0;
       localparam C3_DQ15_TAP_DELAY_VAL   = 0;
       localparam C3_p0_BEGIN_ADDRESS                   = (C3_HW_TESTING == "TRUE") ? 32'h01000000:32'h00000100;
       localparam C3_p0_DATA_MODE                       = 4'b0010;
       localparam C3_p0_END_ADDRESS                     = (C3_HW_TESTING == "TRUE") ? 32'h02ffffff:32'h000002ff;
       localparam C3_p0_PRBS_EADDR_MASK_POS             = (C3_HW_TESTING == "TRUE") ? 32'hfc000000:32'hfffffc00;
       localparam C3_p0_PRBS_SADDR_MASK_POS             = (C3_HW_TESTING == "TRUE") ? 32'h01000000:32'h00000100;

       wire                                                   c3_sys_clk;
       wire                                                   c3_error;
       wire                                                   c3_rst0;
       wire                                                   c3_async_rst;
       wire                                                   c3_sysclk_2x;
       wire                                                   c3_sysclk_2x_180;
       wire                                                   c3_pll_ce_0;
       wire                                                   c3_pll_ce_90;
       wire                                                   c3_pll_lock;
       wire                                                   c3_mcb_drp_clk;
       wire                                                   c3_clk0;

       assign c3_sys_clk     = 1'b0;
       assign reset_out      = 1'b0;

       memc3_infrastructure #
         (
          .C_MEMCLK_PERIOD                  (C3_MEMCLK_PERIOD),
          .C_RST_ACT_LOW                    (C3_RST_ACT_LOW)
          )
       memc3_infrastructure_inst
         (
          .sys_clk_p                      (raw_sys_clk),
          .sys_clk                        (c3_sys_clk),
          .sys_rst_n                      (reset),
          .clk0                           (clk_out),
          .rst0                           (c3_rst0),
          .async_rst                      (c3_async_rst),
          .sysclk_2x                      (c3_sysclk_2x),
          .sysclk_2x_180                  (c3_sysclk_2x_180),
          .pll_ce_0                       (c3_pll_ce_0),
          .pll_ce_90                      (c3_pll_ce_90),
          .pll_lock                       (c3_pll_lock),
          .mcb_drp_clk                    (c3_mcb_drp_clk)
          );

       // wrapper instantiation
       memc3_wrapper #
         (
          .C_MEMCLK_PERIOD                  (C3_MEMCLK_PERIOD),
          .C_CALIB_SOFT_IP                  (C3_CALIB_SOFT_IP),
          .C_SIMULATION                     (C3_SIMULATION),
          .C_ARB_NUM_TIME_SLOTS             (C3_ARB_NUM_TIME_SLOTS),
          .C_ARB_TIME_SLOT_0                (C3_ARB_TIME_SLOT_0),
          .C_ARB_TIME_SLOT_1                (C3_ARB_TIME_SLOT_1),
          .C_ARB_TIME_SLOT_2                (C3_ARB_TIME_SLOT_2),
          .C_ARB_TIME_SLOT_3                (C3_ARB_TIME_SLOT_3),
          .C_ARB_TIME_SLOT_4                (C3_ARB_TIME_SLOT_4),
          .C_ARB_TIME_SLOT_5                (C3_ARB_TIME_SLOT_5),
          .C_ARB_TIME_SLOT_6                (C3_ARB_TIME_SLOT_6),
          .C_ARB_TIME_SLOT_7                (C3_ARB_TIME_SLOT_7),
          .C_ARB_TIME_SLOT_8                (C3_ARB_TIME_SLOT_8),
          .C_ARB_TIME_SLOT_9                (C3_ARB_TIME_SLOT_9),
          .C_ARB_TIME_SLOT_10               (C3_ARB_TIME_SLOT_10),
          .C_ARB_TIME_SLOT_11               (C3_ARB_TIME_SLOT_11),
          .C_MEM_TRAS                       (C3_MEM_TRAS),
          .C_MEM_TRCD                       (C3_MEM_TRCD),
          .C_MEM_TREFI                      (C3_MEM_TREFI),
          .C_MEM_TRFC                       (C3_MEM_TRFC),
          .C_MEM_TRP                        (C3_MEM_TRP),
          .C_MEM_TWR                        (C3_MEM_TWR),
          .C_MEM_TRTP                       (C3_MEM_TRTP),
          .C_MEM_TWTR                       (C3_MEM_TWTR),
          .C_MEM_ADDR_ORDER                 (C3_MEM_ADDR_ORDER),
          .C_NUM_DQ_PINS                    (C3_NUM_DQ_PINS),
          .C_MEM_TYPE                       (C3_MEM_TYPE),
          .C_MEM_DENSITY                    (C3_MEM_DENSITY),
          .C_MEM_BURST_LEN                  (C3_MEM_BURST_LEN),
          .C_MEM_CAS_LATENCY                (C3_MEM_CAS_LATENCY),
          .C_MEM_ADDR_WIDTH                 (C3_MEM_ADDR_WIDTH),
          .C_MEM_BANKADDR_WIDTH             (C3_MEM_BANKADDR_WIDTH),
          .C_MEM_NUM_COL_BITS               (C3_MEM_NUM_COL_BITS),
          .C_MEM_DDR1_2_ODS                 (C3_MEM_DDR1_2_ODS),
          .C_MEM_DDR2_RTT                   (C3_MEM_DDR2_RTT),
          .C_MEM_DDR2_DIFF_DQS_EN           (C3_MEM_DDR2_DIFF_DQS_EN),
          .C_MEM_DDR2_3_PA_SR               (C3_MEM_DDR2_3_PA_SR),
          .C_MEM_DDR2_3_HIGH_TEMP_SR        (C3_MEM_DDR2_3_HIGH_TEMP_SR),
          .C_MEM_DDR3_CAS_LATENCY           (C3_MEM_DDR3_CAS_LATENCY),
          .C_MEM_DDR3_ODS                   (C3_MEM_DDR3_ODS),
          .C_MEM_DDR3_RTT                   (C3_MEM_DDR3_RTT),
          .C_MEM_DDR3_CAS_WR_LATENCY        (C3_MEM_DDR3_CAS_WR_LATENCY),
          .C_MEM_DDR3_AUTO_SR               (C3_MEM_DDR3_AUTO_SR),
          .C_MEM_DDR3_DYN_WRT_ODT           (C3_MEM_DDR3_DYN_WRT_ODT),
          .C_MEM_MOBILE_PA_SR               (C3_MEM_MOBILE_PA_SR),
          .C_MEM_MDDR_ODS                   (C3_MEM_MDDR_ODS),
          .C_MC_CALIB_BYPASS                (C3_MC_CALIB_BYPASS),
          .C_MC_CALIBRATION_MODE            (C3_MC_CALIBRATION_MODE),
          .C_MC_CALIBRATION_DELAY           (C3_MC_CALIBRATION_DELAY),
          .C_SKIP_IN_TERM_CAL               (C3_SKIP_IN_TERM_CAL),
          .C_SKIP_DYNAMIC_CAL               (C3_SKIP_DYNAMIC_CAL),
          .C_LDQSP_TAP_DELAY_VAL            (C3_LDQSP_TAP_DELAY_VAL),
          .C_LDQSN_TAP_DELAY_VAL            (C3_LDQSN_TAP_DELAY_VAL),
          .C_UDQSP_TAP_DELAY_VAL            (C3_UDQSP_TAP_DELAY_VAL),
          .C_UDQSN_TAP_DELAY_VAL            (C3_UDQSN_TAP_DELAY_VAL),
          .C_DQ0_TAP_DELAY_VAL              (C3_DQ0_TAP_DELAY_VAL),
          .C_DQ1_TAP_DELAY_VAL              (C3_DQ1_TAP_DELAY_VAL),
          .C_DQ2_TAP_DELAY_VAL              (C3_DQ2_TAP_DELAY_VAL),
          .C_DQ3_TAP_DELAY_VAL              (C3_DQ3_TAP_DELAY_VAL),
          .C_DQ4_TAP_DELAY_VAL              (C3_DQ4_TAP_DELAY_VAL),
          .C_DQ5_TAP_DELAY_VAL              (C3_DQ5_TAP_DELAY_VAL),
          .C_DQ6_TAP_DELAY_VAL              (C3_DQ6_TAP_DELAY_VAL),
          .C_DQ7_TAP_DELAY_VAL              (C3_DQ7_TAP_DELAY_VAL),
          .C_DQ8_TAP_DELAY_VAL              (C3_DQ8_TAP_DELAY_VAL),
          .C_DQ9_TAP_DELAY_VAL              (C3_DQ9_TAP_DELAY_VAL),
          .C_DQ10_TAP_DELAY_VAL             (C3_DQ10_TAP_DELAY_VAL),
          .C_DQ11_TAP_DELAY_VAL             (C3_DQ11_TAP_DELAY_VAL),
          .C_DQ12_TAP_DELAY_VAL             (C3_DQ12_TAP_DELAY_VAL),
          .C_DQ13_TAP_DELAY_VAL             (C3_DQ13_TAP_DELAY_VAL),
          .C_DQ14_TAP_DELAY_VAL             (C3_DQ14_TAP_DELAY_VAL),
          .C_DQ15_TAP_DELAY_VAL             (C3_DQ15_TAP_DELAY_VAL)
          )
       memc3_wrapper_inst
         (
          .mcb3_dram_dq                        (mcb$data_bus),
          .mcb3_dram_a                         (mcb$address),
          .mcb3_dram_ba                        (mcb$bank_select),
          .mcb3_dram_ras_n                     (mcb$row_address_strobe_not),
          .mcb3_dram_cas_n                     (mcb$column_address_strobe_not),
          .mcb3_dram_we_n                      (mcb$write_enable_not),
          .mcb3_dram_odt                       (mcb$on_die_termination),
          .mcb3_dram_cke                       (mcb$clock_enable),
          .mcb3_dram_dm                        (mcb$data_mask),
          .mcb3_dram_udqs                      (mcb$upper_byte_data_strobe),
          .mcb3_dram_udqs_n                    (mcb$upper_byte_data_strobe_neg),
          .mcb3_rzq                            (mcb$rzq),
          .mcb3_zio                            (mcb$zio),
          .mcb3_dram_udm                       (mcb$upper_data_mask),
          .calib_done                          (calib_done),
          .async_rst                           (c3_async_rst),
          .sysclk_2x                           (c3_sysclk_2x),
          .sysclk_2x_180                       (c3_sysclk_2x_180),
          .pll_ce_0                            (c3_pll_ce_0),
          .pll_ce_90                           (c3_pll_ce_90),
          .pll_lock                            (c3_pll_lock),
          .mcb_drp_clk                         (c3_mcb_drp_clk),
          .mcb3_dram_dqs                       (mcb$data_strobe_signal),
          .mcb3_dram_dqs_n                     (mcb$data_strobe_signal_neg),
          .mcb3_dram_ck                        (mcb$dram_clock),
          .mcb3_dram_ck_n                      (mcb$dram_clock_neg),
          .p0_cmd_clk                          (p0_cmd$clock),
          .p0_cmd_en                           (p0_cmd$enable),
          .p0_cmd_instr                        (p0_cmd$instruction),
          .p0_cmd_bl                           (p0_cmd$burst_length),
          .p0_cmd_byte_addr                    (p0_cmd$byte_address),
          .p0_cmd_empty                        (p0_cmd$empty),
          .p0_cmd_full                         (p0_cmd$full),
          .p0_wr_clk                           (p0_wr$clock),
          .p0_wr_en                            (p0_wr$enable),
          .p0_wr_mask                          (p0_wr$mask),
          .p0_wr_data                          (p0_wr$data),
          .p0_wr_full                          (p0_wr$full),
          .p0_wr_empty                         (p0_wr$empty),
          .p0_wr_count                         (p0_wr$count),
          .p0_wr_underrun                      (p0_wr$underrun),
          .p0_wr_error                         (p0_wr$error),
          .p0_rd_clk                           (p0_rd$clock),
          .p0_rd_en                            (p0_rd$enable),
          .p0_rd_data                          (p0_rd$data),
          .p0_rd_full                          (p0_rd$full),
          .p0_rd_empty                         (p0_rd$empty),
          .p0_rd_count                         (p0_rd$count),
          .p0_rd_overflow                      (p0_rd$overflow),
          .p0_rd_error                         (p0_rd$error),
          .selfrefresh_enter                   (0)
          );
endmodule

(* blackbox *)
module memc3_infrastructure #
  (
   parameter C_MEMCLK_PERIOD    = 2500,
   parameter C_RST_ACT_LOW      = 1,
   parameter C_INPUT_CLK_TYPE   = "DIFFERENTIAL"
   )
  (
   input  wire sys_clk_p,
   //input  wire sys_clk_n,
   input  wire sys_clk,
   input  wire sys_rst_n,
   output wire clk0,
   output wire rst0,
   output wire async_rst,
   output wire sysclk_2x,
   output wire sysclk_2x_180,
   output wire mcb_drp_clk,
   output wire pll_ce_0,
   output wire pll_ce_90,
   output wire pll_lock
   );
endmodule

(* blackbox *)
module memc3_wrapper #
(
parameter C_MEMCLK_PERIOD              = 3200,
parameter C_P0_MASK_SIZE               = 4,
parameter C_P0_DATA_PORT_SIZE          = 32,
parameter C_P1_MASK_SIZE               = 4,
parameter C_P1_DATA_PORT_SIZE          = 32,

parameter  C_ARB_NUM_TIME_SLOTS        = 12,
parameter  C_ARB_TIME_SLOT_0           = 3'o0,
parameter  C_ARB_TIME_SLOT_1           = 3'o0,
parameter  C_ARB_TIME_SLOT_2           = 3'o0,
parameter  C_ARB_TIME_SLOT_3           = 3'o0,
parameter  C_ARB_TIME_SLOT_4           = 3'o0,
parameter  C_ARB_TIME_SLOT_5           = 3'o0,
parameter  C_ARB_TIME_SLOT_6           = 3'o0,
parameter  C_ARB_TIME_SLOT_7           = 3'o0,
parameter  C_ARB_TIME_SLOT_8           = 3'o0,
parameter  C_ARB_TIME_SLOT_9           = 3'o0,
parameter  C_ARB_TIME_SLOT_10          = 3'o0,
parameter  C_ARB_TIME_SLOT_11          = 3'o0,

parameter  C_MEM_TRAS                  = 40000,
parameter  C_MEM_TRCD                  = 15000,
parameter  C_MEM_TREFI                 = 7800000,
parameter  C_MEM_TRFC                  = 127500,
parameter  C_MEM_TRP                   = 15000,
parameter  C_MEM_TWR                   = 15000,
parameter  C_MEM_TRTP                  = 7500,
parameter  C_MEM_TWTR                  = 7500,

parameter  C_MEM_ADDR_ORDER            = "ROW_BANK_COLUMN",
parameter  C_NUM_DQ_PINS               = 16,
parameter  C_MEM_TYPE                  = "DDR2",
parameter  C_MEM_DENSITY               = "1Gb",
parameter  C_MEM_BURST_LEN             = 4,
parameter  C_MEM_CAS_LATENCY           = 5,
parameter  C_MEM_ADDR_WIDTH            = 13,
parameter  C_MEM_BANKADDR_WIDTH        = 3,
parameter  C_MEM_NUM_COL_BITS          = 10,

parameter  C_MEM_DDR1_2_ODS            = "REDUCED",
parameter  C_MEM_DDR2_RTT              = "50OHMS",
parameter  C_MEM_DDR2_DIFF_DQS_EN      = "YES",
parameter  C_MEM_DDR2_3_PA_SR          = "FULL",
parameter  C_MEM_DDR2_3_HIGH_TEMP_SR   = "NORMAL",

parameter  C_MEM_DDR3_CAS_LATENCY      = 7,
parameter  C_MEM_DDR3_ODS              = "DIV6",
parameter  C_MEM_DDR3_RTT              = "DIV2",
parameter  C_MEM_DDR3_CAS_WR_LATENCY   = 5,
parameter  C_MEM_DDR3_AUTO_SR          = "ENABLED",
parameter  C_MEM_DDR3_DYN_WRT_ODT      = "OFF",
parameter  C_MEM_MOBILE_PA_SR          = "FULL",
parameter  C_MEM_MDDR_ODS              = "FULL",

parameter  C_MC_CALIB_BYPASS           = "NO",
parameter  C_SIMULATION                = "FALSE",

parameter C_LDQSP_TAP_DELAY_VAL  = 16,  // 0 to 255 inclusive
parameter C_UDQSP_TAP_DELAY_VAL  = 16,  // 0 to 255 inclusive
parameter C_LDQSN_TAP_DELAY_VAL  = 16,  // 0 to 255 inclusive
parameter C_UDQSN_TAP_DELAY_VAL  = 16,  // 0 to 255 inclusive
parameter C_DQ0_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ1_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ2_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ3_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ4_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ5_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ6_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ7_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ8_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ9_TAP_DELAY_VAL  = 0,  // 0 to 255 inclusive
parameter C_DQ10_TAP_DELAY_VAL = 0,  // 0 to 255 inclusive
parameter C_DQ11_TAP_DELAY_VAL = 0,  // 0 to 255 inclusive
parameter C_DQ12_TAP_DELAY_VAL = 0,  // 0 to 255 inclusive
parameter C_DQ13_TAP_DELAY_VAL = 0,  // 0 to 255 inclusive
parameter C_DQ14_TAP_DELAY_VAL = 0,  // 0 to 255 inclusive
parameter C_DQ15_TAP_DELAY_VAL = 0,  // 0 to 255 inclusive

parameter  C_MC_CALIBRATION_MODE       = "CALIBRATION",
parameter  C_MC_CALIBRATION_DELAY      = "HALF",
parameter  C_CALIB_SOFT_IP             = "TRUE",
parameter  C_SKIP_IN_TERM_CAL            = 0,
parameter  C_SKIP_DYNAMIC_CAL            = 0

    )
  (

      // high-speed PLL clock interface
      input sysclk_2x,
      input sysclk_2x_180,
      input pll_ce_0,
      input pll_ce_90,
      input pll_lock,
      input async_rst,

      //User Port0 Interface Signals

      input                            p0_cmd_clk,
      input                             p0_cmd_en,
      input [2:0]       p0_cmd_instr,
      input [5:0]       p0_cmd_bl,
      input [29:0]        p0_cmd_byte_addr,
      output                          p0_cmd_empty,
      output                           p0_cmd_full,

      // Data Wr Port signals
      input                             p0_wr_clk,
      input                              p0_wr_en,
      input [C_P0_MASK_SIZE - 1:0]                        p0_wr_mask,
      input [C_P0_DATA_PORT_SIZE - 1:0]                             p0_wr_data,
      output                            p0_wr_full,
      output                           p0_wr_empty,
      output [6:0]       p0_wr_count,
      output                        p0_wr_underrun,
      output                           p0_wr_error,

      //Data Rd Port signals
      input                             p0_rd_clk,
      input                              p0_rd_en,
      output [C_P0_DATA_PORT_SIZE - 1:0]                             p0_rd_data,
      output                            p0_rd_full,
      output                           p0_rd_empty,
      output [6:0]       p0_rd_count,
      output                        p0_rd_overflow,
      output                           p0_rd_error,



      // memory interface signals
   inout  [C_NUM_DQ_PINS-1:0]                      mcb3_dram_dq,
   output [C_MEM_ADDR_WIDTH-1:0]       mcb3_dram_a,
   output [C_MEM_BANKADDR_WIDTH-1:0]   mcb3_dram_ba,
   output                              mcb3_dram_ras_n,
   output                              mcb3_dram_cas_n,
   output                              mcb3_dram_we_n,
      output                              mcb3_dram_odt,
   output                              mcb3_dram_cke,
   inout                               mcb3_dram_dqs,
   inout                               mcb3_dram_dqs_n,
   output                              mcb3_dram_ck,
   output                              mcb3_dram_ck_n,

   inout                                            mcb3_dram_udqs,
   output                                           mcb3_dram_udm,


   inout                                            mcb3_dram_udqs_n,



   output                                           mcb3_dram_dm,

      inout                             mcb3_rzq,
      inout                             mcb3_zio,


      // Calibration signals
      input                            mcb_drp_clk,
      output                           calib_done,
      input			       selfrefresh_enter,
      output                           selfrefresh_mode

    );
endmodule

    "##.into(),
            name: "MemoryInterfaceGenerator".into()
        })
    }
}

#[test]
fn test_mig_gen() {
    let mig = MemoryInterfaceGenerator::default();
    let vlog = generate_verilog_unchecked(&mig);
    println!("{}", vlog);
}
