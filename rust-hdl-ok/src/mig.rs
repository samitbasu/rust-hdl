use crate::mcb_if::MCBInterface1GDDR2;
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
    pub mcb: MCBInterface1GDDR2,
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
mcb$data_strobe_signal_neg,mcb$dram_clock,mcb$dram_clock_neg,mcb$chip_select_neg,mcb$dram_reset_not);

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
    output mcb$dram_reset_not;

    assign mcb$dram_reset_not=1'b1; // Unused for DDR2

// Unfortunately, the generated MIG for Spartan6 is not particularly
// useful for us, since it assumes the raw clock is running at memory
// speeds.  You are advised to edit the MIG directly, and change
// the PLL properties so that you can get the desired behavior.
// To keep our implementation as clean as possible, we handle this by
// patching the generated file before including them in the project.
    mig mig_inst(
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
          .c3_sys_clk                          (raw_sys_clk),
          .c3_calib_done                       (calib_done),
          .c3_sys_rst_i                        (reset),
          .c3_clk0                             (clk_out),
          .mcb3_dram_dqs                       (mcb$data_strobe_signal),
          .mcb3_dram_dqs_n                     (mcb$data_strobe_signal_neg),
          .mcb3_dram_ck                        (mcb$dram_clock),
          .mcb3_dram_ck_n                      (mcb$dram_clock_neg),
          .c3_p0_cmd_clk                       (p0_cmd$clock),
          .c3_p0_cmd_en                        (p0_cmd$enable),
          .c3_p0_cmd_instr                     (p0_cmd$instruction),
          .c3_p0_cmd_bl                        (p0_cmd$burst_length),
          .c3_p0_cmd_byte_addr                 (p0_cmd$byte_address),
          .c3_p0_cmd_empty                     (p0_cmd$empty),
          .c3_p0_cmd_full                      (p0_cmd$full),
          .c3_p0_wr_clk                        (p0_wr$clock),
          .c3_p0_wr_en                         (p0_wr$enable),
          .c3_p0_wr_mask                       (p0_wr$mask),
          .c3_p0_wr_data                       (p0_wr$data),
          .c3_p0_wr_full                       (p0_wr$full),
          .c3_p0_wr_empty                      (p0_wr$empty),
          .c3_p0_wr_count                      (p0_wr$count),
          .c3_p0_wr_underrun                   (p0_wr$underrun),
          .c3_p0_wr_error                      (p0_wr$error),
          .c3_p0_rd_clk                        (p0_rd$clock),
          .c3_p0_rd_en                         (p0_rd$enable),
          .c3_p0_rd_data                       (p0_rd$data),
          .c3_p0_rd_full                       (p0_rd$full),
          .c3_p0_rd_empty                      (p0_rd$empty),
          .c3_p0_rd_count                      (p0_rd$count),
          .c3_p0_rd_overflow                   (p0_rd$overflow),
          .c3_p0_rd_error                      (p0_rd$error)
    );
endmodule

(* blackbox *)
module mig #
(
   parameter C3_P0_MASK_SIZE           = 4,
   parameter C3_P0_DATA_PORT_SIZE      = 32,
   parameter C3_P1_MASK_SIZE           = 4,
   parameter C3_P1_DATA_PORT_SIZE      = 32,
   parameter DEBUG_EN                = 0,       
                                       // # = 1, Enable debug signals/controls,
                                       //   = 0, Disable debug signals/controls.
   parameter C3_MEMCLK_PERIOD        = 3200,       
                                       // Memory data transfer clock period
   parameter C3_CALIB_SOFT_IP        = "TRUE",       
                                       // # = TRUE, Enables the soft calibration logic,
                                       // # = FALSE, Disables the soft calibration logic.
   parameter C3_SIMULATION           = "FALSE",       
                                       // # = TRUE, Simulating the design. Useful to reduce the simulation time,
                                       // # = FALSE, Implementing the design.
   parameter C3_RST_ACT_LOW          = 0,       
                                       // # = 1 for active low reset,
                                       // # = 0 for active high reset.
   parameter C3_INPUT_CLK_TYPE       = "SINGLE_ENDED",       
                                       // input clock type DIFFERENTIAL or SINGLE_ENDED
   parameter C3_MEM_ADDR_ORDER       = "ROW_BANK_COLUMN",       
                                       // The order in which user address is provided to the memory controller,
                                       // ROW_BANK_COLUMN or BANK_ROW_COLUMN
   parameter C3_NUM_DQ_PINS          = 16,       
                                       // External memory data width
   parameter C3_MEM_ADDR_WIDTH       = 13,       
                                       // External memory address width
   parameter C3_MEM_BANKADDR_WIDTH   = 3        
                                       // External memory bank address width
)	
(
   inout  [C3_NUM_DQ_PINS-1:0]                      mcb3_dram_dq,
   output [C3_MEM_ADDR_WIDTH-1:0]                   mcb3_dram_a,
   output [C3_MEM_BANKADDR_WIDTH-1:0]               mcb3_dram_ba,
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
   input                                            c3_sys_clk,
   input                                            c3_sys_rst_i,
   output                                           c3_calib_done,
   output                                           c3_clk0,
   output                                           c3_rst0,
   inout                                            mcb3_dram_dqs,
   inout                                            mcb3_dram_dqs_n,
   output                                           mcb3_dram_ck,
   output                                           mcb3_dram_ck_n,
   input		                            c3_p0_cmd_clk,
   input		                            c3_p0_cmd_en,
   input [2:0]	                                    c3_p0_cmd_instr,
   input [5:0]	                                    c3_p0_cmd_bl,
   input [29:0]	                                    c3_p0_cmd_byte_addr,
   output		                            c3_p0_cmd_empty,
   output		                            c3_p0_cmd_full,
   input		                            c3_p0_wr_clk,
   input		                            c3_p0_wr_en,
   input [C3_P0_MASK_SIZE - 1:0]	            c3_p0_wr_mask,
   input [C3_P0_DATA_PORT_SIZE - 1:0]	            c3_p0_wr_data,
   output		                            c3_p0_wr_full,
   output		                            c3_p0_wr_empty,
   output [6:0]	                                    c3_p0_wr_count,
   output		                            c3_p0_wr_underrun,
   output		                            c3_p0_wr_error,
   input		                            c3_p0_rd_clk,
   input		                            c3_p0_rd_en,
   output [C3_P0_DATA_PORT_SIZE - 1:0]	            c3_p0_rd_data,
   output		                            c3_p0_rd_full,
   output		                            c3_p0_rd_empty,
   output [6:0]	                                    c3_p0_rd_count,
   output		                            c3_p0_rd_overflow,
   output		                            c3_p0_rd_error
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
