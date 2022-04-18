use super::mcb_if::MCBInterface1GDDR2;
use crate::core::prelude::*;
use crate::widgets::prelude::*;
use std::collections::BTreeMap;

#[derive(LogicState, Copy, Clone, Debug, PartialEq)]
pub enum MIGInstruction {
    Write,
    Read,
    WritePrecharge,
    ReadPrecharge,
    Refresh,
}

#[derive(LogicStruct, Copy, Clone, Debug, Default, PartialEq)]
pub struct MIGCommand {
    pub instruction: MIGInstruction,
    pub burst_len: Bits<6>,
    pub byte_address: Bits<30>,
}

#[derive(LogicStruct, Copy, Clone, Debug, Default, PartialEq)]
pub struct MaskedWrite {
    pub mask: Bits<4>,
    pub data: Bits<32>,
}

#[derive(LogicInterface, Default)]
pub struct CommandPort {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub cmd: Signal<In, MIGCommand>,
    pub empty: Signal<Out, Bit>,
    pub full: Signal<Out, Bit>,
}

#[derive(LogicInterface, Default)]
pub struct WritePort {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub data: Signal<In, MaskedWrite>,
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

#[derive(LogicState, Copy, Clone, Debug, PartialEq)]
enum State {
    Init,
    Calibrating,
    Idle,
    Reading,
    Writing,
    Error,
    Refresh,
}

#[derive(LogicBlock)]
pub struct MemoryInterfaceGenerator {
    // Raw clock from the system - cannot be intercepted
    pub raw_sys_clk: Signal<In, Clock>,
    // Reset - must be handled externally
    pub reset: Signal<In, ResetN>,
    // Calibration complete
    pub calib_done: Signal<Out, Bit>,
    // Buffered 100 MHz clock
    pub clk_out: Signal<Out, Clock>,
    // Delayed reset
    pub reset_out: Signal<Out, ResetN>,
    // P0 command port
    pub p0_cmd: CommandPort,
    // P0 write port
    pub p0_wr: WritePort,
    // P0 read port
    pub p0_rd: ReadPort,
    // MCB interface
    pub mcb: MCBInterface1GDDR2,
    // FIFO for the commands
    cmd_fifo: AsynchronousFIFO<MIGCommand, 2, 3, 1>,
    write_fifo: AsynchronousFIFO<MaskedWrite, 6, 7, 1>,
    read_fifo: AsynchronousFIFO<Bits<32>, 6, 7, 1>,
    timer: DFF<Bits<16>>,
    address: DFF<Bits<32>>,
    state: DFF<State>,
    calib: DFF<bool>,
    _dram: BTreeMap<Bits<32>, Bits<32>>,
    cmd: Signal<Local, MIGCommand>,
    wr_reset: AutoReset,
    rd_reset: AutoReset,
    cmd_reset: AutoReset,
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
            cmd_fifo: AsynchronousFIFO::default(),
            write_fifo: AsynchronousFIFO::default(),
            read_fifo: Default::default(),
            timer: Default::default(),
            address: Default::default(),
            state: Default::default(),
            calib: Default::default(),
            _dram: Default::default(),
            cmd: Default::default(),
            wr_reset: Default::default(),
            rd_reset: Default::default(),
            cmd_reset: Default::default(),
        }
    }
}

impl Logic for MemoryInterfaceGenerator {
    fn update(&mut self) {
        // Connect the hardware side of the fifos to the raw clock
        self.cmd_fifo.read_clock.next = self.raw_sys_clk.val();
        self.cmd_fifo.read_reset.next = self.reset.val();
        self.write_fifo.read_clock.next = self.raw_sys_clk.val();
        self.write_fifo.read_reset.next = self.reset.val();
        self.read_fifo.write_clock.next = self.raw_sys_clk.val();
        self.read_fifo.write_reset.next = self.reset.val();
        dff_setup!(self, raw_sys_clk, reset, state, calib, timer, address);
        // Connect the command fifo to the command port
        self.cmd_fifo.data_in.next = self.p0_cmd.cmd.val();
        self.cmd_fifo.write.next = self.p0_cmd.enable.val();
        self.p0_cmd.empty.next = !self.cmd_fifo.write_fill.val().any();
        self.p0_cmd.full.next = self.cmd_fifo.full.val();
        self.cmd_fifo.write_clock.next = self.p0_cmd.clock.val();
        self.cmd_reset.clock.next = self.p0_cmd.clock.val();
        self.cmd_fifo.write_reset.next = self.cmd_reset.reset.val();
        // Connect the write fifo to the write port
        self.write_fifo.data_in.next = self.p0_wr.data.val();
        self.write_fifo.write.next = self.p0_wr.enable.val();
        self.p0_wr.empty.next = !self.write_fifo.write_fill.val().any();
        self.p0_wr.full.next = self.write_fifo.full.val();
        self.p0_wr.error.next = self.write_fifo.underflow.val() | self.write_fifo.overflow.val();
        self.p0_wr.underrun.next = self.write_fifo.underflow.val();
        self.write_fifo.write_clock.next = self.p0_wr.clock.val();
        self.wr_reset.clock.next = self.p0_wr.clock.val();
        self.write_fifo.write_reset.next = self.wr_reset.reset.val();
        self.p0_wr.count.next = self.write_fifo.write_fill.val();
        // Connect the read fifo to the read port
        self.p0_rd.data.next = self.read_fifo.data_out.val();
        self.read_fifo.read.next = self.p0_rd.enable.val();
        self.p0_rd.error.next = self.read_fifo.underflow.val() | self.read_fifo.overflow.val();
        self.p0_rd.overflow.next = self.read_fifo.overflow.val();
        self.p0_rd.empty.next = self.read_fifo.empty.val();
        self.p0_rd.full.next = self.read_fifo.full.val();
        self.read_fifo.read_clock.next = self.p0_rd.clock.val();
        self.rd_reset.clock.next = self.p0_rd.clock.val();
        self.read_fifo.read_reset.next = self.rd_reset.reset.val();
        self.p0_rd.count.next = self.read_fifo.read_fill.val();
        self.calib_done.next = self.calib.q.val();
        self.cmd.next = self.cmd_fifo.data_out.val();
        if self.timer.q.val().any() {
            self.timer.d.next = self.timer.q.val() - 1_usize;
        } else {
            self.timer.d.next = self.timer.q.val();
        }
        self.clk_out.next = self.raw_sys_clk.val();
        self.cmd_fifo.read.next = false;
        self.write_fifo.read.next = false;
        self.read_fifo.write.next = false;
        self.reset_out.next = true.into();
        match self.state.q.val() {
            State::Init => {
                self.state.d.next = State::Calibrating;
                self.timer.d.next = 100_usize.into();
            }
            State::Calibrating => {
                if self.timer.q.val() == 0_usize {
                    self.calib.d.next = true;
                    self.state.d.next = State::Idle;
                    self.reset_out.next = false.into();
                }
            }
            State::Idle => {
                if !self.cmd_fifo.empty.val() {
                    // Byte address lower 2 bits must be zero
                    if self.cmd.val().byte_address.get_bits::<2>(0).any() {
                        self.state.d.next = State::Error;
                    } else {
                        match self.cmd.val().instruction {
                            MIGInstruction::Write | MIGInstruction::WritePrecharge => {
                                self.state.d.next = State::Writing;
                                self.timer.d.next =
                                    bit_cast::<16, 6>(self.cmd.val().burst_len) + 1_usize;
                                self.address.d.next =
                                    bit_cast::<32, 30>(self.cmd.val().byte_address) >> 2_usize;
                                self.cmd_fifo.read.next = true;
                            }
                            MIGInstruction::Read | MIGInstruction::ReadPrecharge => {
                                self.timer.d.next =
                                    bit_cast::<16, 6>(self.cmd.val().burst_len) + 1_usize;
                                self.address.d.next =
                                    bit_cast::<32, 30>(self.cmd.val().byte_address) >> 2_usize;
                                self.state.d.next = State::Reading;
                                self.cmd_fifo.read.next = true;
                            }
                            MIGInstruction::Refresh => {
                                self.state.d.next = State::Refresh;
                            }
                        }
                    }
                }
            }
            State::Reading => {
                if self.timer.q.val().any() {
                    self.read_fifo.data_in.next = *self
                        ._dram
                        .get(&self.address.q.val())
                        .unwrap_or(&Default::default());
                    self.read_fifo.write.next = true;
                    self.address.d.next = self.address.q.val() + 1_usize;
                } else {
                    self.state.d.next = State::Idle;
                }
            }
            State::Writing => {
                if self.timer.q.val().any() {
                    self._dram
                        .insert(self.address.q.val(), self.write_fifo.data_out.val().data);
                    self.write_fifo.read.next = true;
                    self.address.d.next = self.address.q.val() + 1_usize;
                } else {
                    self.state.d.next = State::Idle;
                }
            }
            State::Refresh => {
                self.cmd_fifo.read.next = true;
                self.state.d.next = State::Idle;
            }
            State::Error => {}
        }
    }
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
        self.write_fifo.write_clock.connect();
        self.cmd_fifo.write_clock.connect();
        self.state.clock.connect();
        self.cmd_fifo.read.connect();
        self.cmd_fifo.data_in.connect();
        self.read_fifo.write.connect();
        self.read_fifo.write_clock.connect();
        self.timer.clock.connect();
        self.write_fifo.read_clock.connect();
        self.read_fifo.read_clock.connect();
        self.state.d.connect();
        self.write_fifo.write.connect();
        self.write_fifo.write_clock.connect();
        self.calib.clock.connect();
        self.read_fifo.data_in.connect();
        self.calib.d.connect();
        self.cmd_fifo.write.connect();
        self.timer.d.connect();
        self.cmd_fifo.read_clock.connect();
        self.read_fifo.read.connect();
        self.write_fifo.read.connect();
        self.write_fifo.data_in.connect();
        self.address.d.connect();
        self.cmd.connect();
        self.address.clock.connect();
        self.wr_reset.clock.connect();
        self.rd_reset.clock.connect();
        self.read_fifo.read_reset.connect();
        self.read_fifo.write_reset.connect();
        self.cmd_fifo.write_reset.connect();
        self.cmd_fifo.read_reset.connect();
        self.write_fifo.read_reset.connect();
        self.write_fifo.write_reset.connect();
        self.address.reset.connect();
        self.calib.reset.connect();
        self.timer.reset.connect();
        self.state.reset.connect();
        self.cmd_reset.clock.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: r##"
// assign mcb$dram_reset_not=1'b1;
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
          .c3_p0_cmd_instr                     (p0_cmd$cmd[2:0]), // Lowest 3 bits are the cmd
          .c3_p0_cmd_bl                        (p0_cmd$cmd[8:3]), // 6 bits for the burst length
          .c3_p0_cmd_byte_addr                 (p0_cmd$cmd[38:9]), // 30 bits for the byte address
          .c3_p0_cmd_empty                     (p0_cmd$empty),
          .c3_p0_cmd_full                      (p0_cmd$full),
          .c3_p0_wr_clk                        (p0_wr$clock),
          .c3_p0_wr_en                         (p0_wr$enable),
          .c3_p0_wr_mask                       (p0_wr$data[3:0]), // Lowest 4 bits are the byte mask
          .c3_p0_wr_data                       (p0_wr$data[35:4]), // Upper 32 bits are the data
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
"##.into(),
cores: r##"
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
        })
    }
}

#[test]
fn test_mig_gen() {
    let mig = MemoryInterfaceGenerator::default();
    let _vlog = generate_verilog_unchecked(&mig);
}
