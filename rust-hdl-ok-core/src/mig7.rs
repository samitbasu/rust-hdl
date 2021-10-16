use crate::mcb_if::MCBInterface4GDDR3;
use rust_hdl_core::ast::Wrapper;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::dff::DFF;
use std::collections::BTreeMap;

#[derive(LogicBlock, Default)]
pub struct MemoryInterfaceGenerator7Series {
    // Raw clock from the system - differential and raw
    pub raw_pos_clock: Signal<In, Clock>,
    pub raw_neg_clock: Signal<In, Clock>,
    // MCB Interface
    pub mcb: MCBInterface4GDDR3,
    // Address (this is apparently a "word" address, not a byte address)
    pub address: Signal<In, Bits<29>>,
    pub command: Signal<In, Bits<3>>,
    pub enable: Signal<In, Bit>,
    pub write_data_in: Signal<In, Bits<128>>,
    pub write_data_end: Signal<In, Bit>,
    pub write_data_mask: Signal<In, Bits<16>>,
    pub write_enable: Signal<In, Bit>,
    pub read_data_out: Signal<Out, Bits<128>>,
    pub read_data_end: Signal<Out, Bit>,
    pub read_data_valid: Signal<Out, Bit>,
    pub ready: Signal<Out, Bit>,
    pub write_fifo_not_full: Signal<Out, Bit>,
    pub calib_done: Signal<Out, Bit>,
    pub reset: Signal<In, Bit>,
    pub clock: Signal<Out, Clock>,
    pub reset_out: Signal<Out, Bit>,
}

impl Logic for MemoryInterfaceGenerator7Series {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.read_data_out.connect();
        self.read_data_end.connect();
        self.read_data_valid.connect();
        self.ready.connect();
        self.write_fifo_not_full.connect();
        self.calib_done.connect();
        self.clock.connect();
        self.reset_out.connect();
        self.mcb.link_connect_source();
        self.mcb.link_connect_dest();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Blackbox(BlackBox {
            code: r##"
module MemoryInterfaceGenerator7Series(raw_pos_clock,raw_neg_clock,mcb$data_bus,mcb$address,mcb$bank_select,
mcb$row_address_strobe_not,mcb$column_address_strobe_not,mcb$write_enable_not,
mcb$on_die_termination,mcb$clock_enable,mcb$data_mask,mcb$data_strobe_signal,
mcb$data_strobe_signal_neg,mcb$dram_clock,mcb$dram_clock_neg,mcb$reset_not,
address,command,enable,write_data_in,write_data_end,write_data_mask,write_enable,
read_data_out,read_data_end,read_data_valid,ready,write_fifo_not_full,calib_done,
reset,clock,reset_out);

    // Module arguments
    input wire raw_pos_clock;
    input wire raw_neg_clock;
    inout wire [15:0] mcb$data_bus;
    output wire [14:0] mcb$address;
    output wire [2:0] mcb$bank_select;
    output wire mcb$row_address_strobe_not;
    output wire mcb$column_address_strobe_not;
    output wire mcb$write_enable_not;
    output wire mcb$on_die_termination;
    output wire mcb$clock_enable;
    output wire [1:0] mcb$data_mask;
    inout wire [1:0] mcb$data_strobe_signal;
    inout wire [1:0] mcb$data_strobe_signal_neg;
    output wire mcb$dram_clock;
    output wire mcb$dram_clock_neg;
    output wire mcb$reset_not;
    input wire [28:0] address;
    input wire [2:0] command;
    input wire enable;
    input wire [127:0] write_data_in;
    input wire write_data_end;
    input wire [15:0] write_data_mask;
    input wire write_enable;
    output wire [127:0] read_data_out;
    output wire read_data_end;
    output wire read_data_valid;
    output wire ready;
    output wire write_fifo_not_full;
    output wire calib_done;
    input wire reset;
    output wire clock;
    output wire reset_out;

mig7 mig_inst (
    // Memory interface ports
    .ddr3_addr                      (mcb$address),                          // output [14:0]        ddr3_addr
    .ddr3_ba                        (mcb$bank_select),                      // output [2:0]        ddr3_ba
    .ddr3_cas_n                     (mcb$column_address_strobe_not),        // output            ddr3_cas_n
    .ddr3_ck_n                      (mcb$dram_clock_neg),                   // output [0:0]        ddr3_ck_n
    .ddr3_ck_p                      (mcb$dram_clock),                       // output [0:0]        ddr3_ck_p
    .ddr3_cke                       (mcb$clock_enable),                     // output [0:0]        ddr3_cke
    .ddr3_ras_n                     (mcb$row_address_strobe_not),           // output            ddr3_ras_n
    .ddr3_reset_n                   (mcb$reset_not),                        // output            ddr3_reset_n
    .ddr3_we_n                      (mcb$write_enable_not),                 // output            ddr3_we_n
    .ddr3_dq                        (mcb$data_bus),                         // inout [15:0]        ddr3_dq
    .ddr3_dqs_n                     (mcb$data_strobe_signal_neg),           // inout [1:0]        ddr3_dqs_n
    .ddr3_dqs_p                     (mcb$data_strobe_signal),               // inout [1:0]        ddr3_dqs_p
    .init_calib_complete            (calib_done),                           // output            init_calib_complete
    .ddr3_dm                        (mcb$data_mask),                        // output [1:0]   ddr3_dm
    .ddr3_odt                       (mcb$on_die_termination),               // output [0:0]        ddr3_odt
    // Application interface ports
    .app_addr                       (address),                              // input [28:0]        app_addr
    .app_cmd                        (command),                              // input [2:0]        app_cmd
    .app_en                         (enable),                               // input                app_en
    .app_wdf_data                   (write_data_in),                        // input [127:0]        app_wdf_data
    .app_wdf_end                    (write_data_end),                       // input                app_wdf_end
    .app_wdf_wren                   (write_enable),                         // input                app_wdf_wren
    .app_rd_data                    (read_data_out),                        // output [127:0]        app_rd_data
    .app_rd_data_end                (read_data_end),                        // output            app_rd_data_end
    .app_rd_data_valid              (read_data_valid),                      // output            app_rd_data_valid
    .app_rdy                        (ready),                                // output            app_rdy
    .app_wdf_rdy                    (write_fifo_not_full),                  // output            app_wdf_rdy
    .app_sr_req                     (1'b0),                                 // input            app_sr_req
    .app_ref_req                    (1'b0),                                 // input            app_ref_req
    .app_zq_req                     (1'b0),                                 // input            app_zq_req
    .app_sr_active                  (),                                     // output            app_sr_active
    .app_ref_ack                    (),                                     // output            app_ref_ack
    .app_zq_ack                     (),                                     // output            app_zq_ack
    .ui_clk                         (clock),                                // output            ui_clk
    .ui_clk_sync_rst                (reset_out),                            // output            ui_clk_sync_rst
    .app_wdf_mask                   (write_data_mask),                      // input [15:0]        app_wdf_mask
    // System Clock Ports
    .sys_clk_p                      (raw_pos_clock),                        // input                sys_clk_p
    .sys_clk_n                      (raw_neg_clock),                        // input                sys_clk_n
    .sys_rst                        (reset)                               // input sys_rst
);
endmodule

(* blackbox *)
module mig7
(ddr3_dq, ddr3_dqs_n, ddr3_dqs_p, ddr3_addr,
  ddr3_ba, ddr3_ras_n, ddr3_cas_n, ddr3_we_n, ddr3_reset_n, ddr3_ck_p, ddr3_ck_n, ddr3_cke,
  ddr3_dm, ddr3_odt, sys_clk_p, sys_clk_n, app_addr, app_cmd, app_en, app_wdf_data, app_wdf_end,
  app_wdf_mask, app_wdf_wren, app_rd_data, app_rd_data_end, app_rd_data_valid, app_rdy,
  app_wdf_rdy, app_sr_req, app_ref_req, app_zq_req, app_sr_active, app_ref_ack, app_zq_ack,
  ui_clk, ui_clk_sync_rst, init_calib_complete, device_temp, sys_rst);
  inout [15:0]ddr3_dq;
  inout [1:0]ddr3_dqs_n;
  inout [1:0]ddr3_dqs_p;
  output [14:0]ddr3_addr;
  output [2:0]ddr3_ba;
  output ddr3_ras_n;
  output ddr3_cas_n;
  output ddr3_we_n;
  output ddr3_reset_n;
  output [0:0]ddr3_ck_p;
  output [0:0]ddr3_ck_n;
  output [0:0]ddr3_cke;
  output [1:0]ddr3_dm;
  output [0:0]ddr3_odt;
  input sys_clk_p;
  input sys_clk_n;
  input [28:0]app_addr;
  input [2:0]app_cmd;
  input app_en;
  input [127:0]app_wdf_data;
  input app_wdf_end;
  input [15:0]app_wdf_mask;
  input app_wdf_wren;
  output [127:0]app_rd_data;
  output app_rd_data_end;
  output app_rd_data_valid;
  output app_rdy;
  output app_wdf_rdy;
  input app_sr_req;
  input app_ref_req;
  input app_zq_req;
  output app_sr_active;
  output app_ref_ack;
  output app_zq_ack;
  output ui_clk;
  output ui_clk_sync_rst;
  output init_calib_complete;
  output [11:0]device_temp;
  input sys_rst;
endmodule
            "##.into(),
            name: "MemoryInterfaceGenerator7Series".into()
        })
    }
}

#[test]
fn test_mig7_gen() {
    let mig = MemoryInterfaceGenerator7Series::default();
    let vlog = generate_verilog_unchecked(&mig);
    println!("{}", vlog);
}

#[derive(LogicState, Copy, Clone, Debug, PartialEq)]
pub enum MIG7SimState {
    Reset,
    Calibrating,
    Idle,
}

/*
// Just used for simulation...
#[derive(LogicBlock, Default)]
pub struct MIG7Sim {
    pub raw_pos_clock: Signal<In, Clock>,
    pub raw_neg_clock: Signal<In, Clock>,
    pub mcb: MCBInterface4GDDR3,
    pub address: Signal<In, Bits<29>>,
    pub command: Signal<In, Bits<3>>,
    pub enable: Signal<In, Bit>,
    pub write_data_in: Signal<In, Bits<128>>,
    pub write_data_end: Signal<In, Bit>,
    pub write_data_mask: Signal<In, Bits<16>>,
    pub write_enable: Signal<In, Bit>,
    pub read_data_out: Signal<Out, Bits<128>>,
    pub read_data_end: Signal<Out, Bit>,
    pub read_data_valid: Signal<Out, Bit>,
    pub ready: Signal<Out, Bit>,
    pub write_fifo_not_full: Signal<Out, Bit>,
    pub calib_done: Signal<Out, Bit>,
    pub reset: Signal<In, Bit>,
    pub clock: Signal<Out, Clock>,
    pub reset_out: Signal<Out, Bit>,
    state: DFF<MIG7SimState>,
    data_in_register: DFF<Bits<128>>,
    data_out_register: DFF<Bits<128>>,
    calib_flag: DFF<Bit>,
    _sim: BTreeMap<Bits<29>, Bits<128>>,
}

impl<D: Synth, const N: usize> Logic for MIG7Sim {
    fn update(&mut self) {
        self.clock.next = self.raw_pos_clock.val();
        self.state.clk.next = self.raw_pos_clock.val();
        self.data_in_register.clk.next = self.raw_pos_clock.val();
        self.data_out_register.clk.next = self.raw_pos_clock.val();
        self.data_in_register.d.next = self.data_in_register.q.val();
        self.data_out_register.d.next = self.data_out_register.q.val();
        self.calib_flag.clk.next = self.raw_pos_clock.val();
        self.calib_flag.d.next = self.calib_flag.q.val();
        self.state.d.next = self.state.q.val();
        match self.state.q.val() {
            MIG7SimState::Reset => {
                self.calib_flag.d.next = false;
                if !self.reset.val() {
                    self.state.d.next = MIG7SimState::Calibrating;
                }
            }
            MIG7SimState::Calibrating => {
                self.calb_flag.d.next = true;
                self.state.d.next = MIG7SimState::Idle;
            }
            MIG7SimState::Idle => {
            }
        }
        if self.reset.val() {
            self.state.d.next = MIG7SimState::Reset;
        }
        self.calib_done.next = self.calib_flag.val();
        self.ready.next = self.state.q.val() == MIG7SimState::Idle;
    }
}

 */
