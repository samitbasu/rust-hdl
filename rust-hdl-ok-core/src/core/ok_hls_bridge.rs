use crate::core::ok_pipe::{BTPipeIn, BTPipeOut};
use crate::core::ok_wire::{WireIn, WireOut};
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;
use rust_hdl_ok_frontpanel_sys::{
    make_u16_buffer, ok_ErrorCode_ok_DataAlignmentError, ok_ErrorCode_ok_InvalidParameter,
    ok_ErrorCode_ok_Timeout, OkError, OkHandle,
};
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub struct OKHLSBridgeAddressConfig {
    pub pipe_in: u8,
    pub pipe_out: u8,
    pub words_avail: u8,
    pub space_avail: u8,
    pub block_flow_control: u8,
}

impl Default for OKHLSBridgeAddressConfig {
    fn default() -> Self {
        Self {
            pipe_in: 0x9D,
            pipe_out: 0xBD,
            words_avail: 0x3D,
            space_avail: 0x3E,
            block_flow_control: 0x1D,
        }
    }
}

// Create a bridge object that can communicate to an HLS based SoC
// but connects to the opal kelly host interface to use the FrontPanel
// API for pc to FPGA communication.  To avoid a force of all firmware
// to HLS in one go, this bridge sits on the OK bus, and handles the
// comms between the FPGA and the pc.
#[derive(LogicBlock)]
pub struct OpalKellyHLSBridge<const A: usize> {
    /// Clock for the whole thing
    pub ti_clk: Signal<In, Clock>,
    /// OK1 bus (used for fan out from the OK Host)
    pub ok1: Signal<In, Bits<31>>,
    /// OK2 bus (used for logical or-in to the OK Host)
    pub ok2: Signal<Out, Bits<17>>,
    /// HLS Bus
    pub bus: SoCBusController<16, A>,
    pc_to_fpga_fifo: SyncFIFO<Bits<16>, 12, 13, 512>,
    fpga_to_pc_fifo: SyncFIFO<Bits<16>, 12, 13, 512>,
    controller: BaseController<A>,
    pipe_in: BTPipeIn,
    pipe_out: BTPipeOut,
    words_avail: WireOut,
    space_avail: WireOut,
    space_counter: DFF<Bits<16>>,
    word_counter: DFF<Bits<16>>,
    read_delay: DFF<bool>,
    block_flow_control: WireIn,
    sr: AutoReset,
    reset: Signal<Local, Reset>,
}

impl<const A: usize> Logic for OpalKellyHLSBridge<A> {
    #[hdl_gen]
    fn update(&mut self) {
        // Wire up the reset
        self.sr.clock.next = self.ti_clk.val();
        self.reset.next = self.sr.reset.val();
        // Clock the internal components
        clock_reset!(
            self,
            ti_clk,
            reset,
            controller,
            pc_to_fpga_fifo,
            fpga_to_pc_fifo
        );
        dff_setup!(self, ti_clk, reset, space_counter, word_counter, read_delay);
        // Link the FIFOs to the HLS controller
        FIFOReadController::<Bits<16>>::join(
            &mut self.controller.from_cpu,
            &mut self.pc_to_fpga_fifo.bus_read,
        );
        FIFOWriteController::<Bits<16>>::join(
            &mut self.controller.to_cpu,
            &mut self.fpga_to_pc_fifo.bus_write,
        );
        // Link the bus to the controller
        SoCBusController::<16, A>::link(&mut self.bus, &mut self.controller.bus);
        // Connect up the read delay flop
        self.read_delay.d.next = self.pipe_out.read.val();
        // Connect the pipes to the OK1 and OK2 busses
        self.pipe_in.ok1.next = self.ok1.val();
        self.pipe_out.ok1.next = self.ok1.val();
        self.words_avail.ok1.next = self.ok1.val();
        self.space_avail.ok1.next = self.ok1.val();
        self.block_flow_control.ok1.next = self.ok1.val();
        self.ok2.next = self.pipe_in.ok2.val()
            | self.pipe_out.ok2.val()
            | self.words_avail.ok2.val()
            | self.space_avail.ok2.val();
        // Update the total number of words available in the FIFO
        if self.fpga_to_pc_fifo.bus_write.write.val() & !self.fpga_to_pc_fifo.bus_read.read.val() {
            self.word_counter.d.next = self.word_counter.q.val() + 1_usize;
        } else if self.fpga_to_pc_fifo.bus_read.read.val()
            & !self.fpga_to_pc_fifo.bus_write.write.val()
        {
            self.word_counter.d.next = self.word_counter.q.val() - 1_usize;
        }
        if self.pc_to_fpga_fifo.bus_write.write.val() & !self.pc_to_fpga_fifo.bus_read.read.val() {
            self.space_counter.d.next = self.space_counter.q.val() - 1_usize;
        } else if self.pc_to_fpga_fifo.bus_read.read.val()
            & !self.pc_to_fpga_fifo.bus_write.write.val()
        {
            self.space_counter.d.next = self.space_counter.q.val() + 1_usize;
        }
        // Reflect the word counter out to the wire
        self.words_avail.datain.next = self.word_counter.q.val();
        self.space_avail.datain.next = self.space_counter.q.val();
        // Connect the pipes to the fifos
        self.fpga_to_pc_fifo.bus_read.read.next = self.read_delay.q.val();
        self.pipe_out.datain.next = self.fpga_to_pc_fifo.bus_read.data.val();
        self.pc_to_fpga_fifo.bus_write.write.next = self.pipe_in.write.val();
        self.pc_to_fpga_fifo.bus_write.data.next = self.pipe_in.dataout.val();
        // The stream control ties the pipe ready signals to either true (no stream control)
        // or false (stream control based on FIFO level).
        if self.block_flow_control.dataout.val().get_bit(0_usize) {
            self.pipe_out.ready.next = !self.fpga_to_pc_fifo.bus_read.almost_empty.val();
        } else {
            self.pipe_out.ready.next = true;
        }
        if self.block_flow_control.dataout.val().get_bit(1_usize) {
            self.pipe_in.ready.next = !self.pc_to_fpga_fifo.bus_write.almost_full.val();
        } else {
            self.pipe_in.ready.next = true;
        }
    }
}

impl<const A: usize> OpalKellyHLSBridge<A> {
    pub fn new(config: OKHLSBridgeAddressConfig) -> Self {
        Self {
            ti_clk: Default::default(),
            ok1: Default::default(),
            ok2: Default::default(),
            bus: Default::default(),
            pc_to_fpga_fifo: Default::default(),
            fpga_to_pc_fifo: Default::default(),
            controller: Default::default(),
            pipe_in: BTPipeIn::new(config.pipe_in),
            pipe_out: BTPipeOut::new(config.pipe_out),
            words_avail: WireOut::new(config.words_avail),
            space_avail: WireOut::new(config.space_avail),
            space_counter: DFF::new_with_reset_val((1_usize << 12).into()),
            word_counter: Default::default(),
            read_delay: Default::default(),
            block_flow_control: WireIn::new(config.block_flow_control),
            sr: Default::default(),
            reset: Default::default(),
        }
    }
}

impl<const A: usize> Default for OpalKellyHLSBridge<A> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[test]
fn test_ok_hls_bridge_synthesizes() {
    let mut uut = OpalKellyHLSBridge::<16>::default();
    uut.connect_all();
    yosys_validate("ok_hls_bridge", &generate_verilog(&uut)).unwrap()
}

pub fn mk_u8(dat: &[u16]) -> Vec<u8> {
    let mut ret = vec![0_u8; dat.len() * 2];
    for (ndx, el) in dat.iter().enumerate() {
        ret[2 * ndx] = (el & 0xFF) as u8;
        ret[2 * ndx + 1] = ((el & 0xFF00) >> 8) as u8;
    }
    ret
}

pub fn write_bridge_bytes(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
    data: &[u8],
) -> Result<(), OkError> {
    if data.len() % 2 != 0 {
        return Err(OkError {
            code: ok_ErrorCode_ok_InvalidParameter,
        });
    }
    let mut send_ok = false;
    for _retry in 0..100 {
        hnd.update_wire_outs();
        if hnd.get_wire_out(config.space_avail as i32) >= ((data.len() * 2) as u16) {
            send_ok = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if !send_ok {
        return Err(OkError {
            code: ok_ErrorCode_ok_Timeout,
        });
    }
    // Send the message
    hnd.write_to_pipe_in(config.pipe_in as i32, data)
}

fn read_bridge_bytes(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
    len: usize,
) -> Result<Vec<u8>, OkError> {
    if len % 2 != 0 {
        return Err(OkError {
            code: ok_ErrorCode_ok_InvalidParameter,
        });
    }
    let mut data_ok = false;
    for _retry in 0..100 {
        hnd.update_wire_outs();
        if hnd.get_wire_out(config.words_avail as i32) * 2 >= (len as u16) {
            data_ok = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if !data_ok {
        return Err(OkError {
            code: ok_ErrorCode_ok_Timeout,
        });
    }
    let mut data = vec![0_u8; len];
    hnd.read_from_pipe_out(config.pipe_out as i32, &mut data)?;
    Ok(data)
}

pub fn write_data_to_address(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
    address: u8,
    data: &[u16],
) -> Result<(), OkError> {
    let mut msg = vec![0_u16; data.len() + 2];
    msg[0] = 0x0300 | (address as u16);
    msg[1] = data.len() as u16;
    for (ndx, el) in data.iter().enumerate() {
        msg[ndx + 2] = *el;
    }
    write_bridge_bytes(hnd, config, &mk_u8(&msg))
}

pub fn read_data_from_address(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
    address: u8,
    len: usize,
) -> Result<Vec<u16>, OkError> {
    assert!(len <= (1 << 16));
    let msg = [0x0200_u16 | (address as u16), len as u16];
    write_bridge_bytes(hnd, config, &mk_u8(&msg))?;
    let data = read_bridge_bytes(hnd, config, len * 2)?;
    Ok(make_u16_buffer(&data))
}

pub fn send_bridge_ping(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
    id: u8,
) -> Result<(), OkError> {
    write_bridge_bytes(hnd, config, &mk_u8(&[0x0100 | (id as u16)]))
}

pub fn receive_bridge_ping(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
) -> Result<u16, OkError> {
    let data = read_bridge_bytes(hnd, config, 2)?;
    Ok(make_u16_buffer(&data)[0])
}

pub fn ping_bridge(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
    id: u8,
) -> Result<(), OkError> {
    send_bridge_ping(hnd, config, id)?;
    let ret = receive_bridge_ping(hnd, config)?;
    if ret != 0x100 | (id as u16) {
        Err(OkError {
            code: ok_ErrorCode_ok_DataAlignmentError,
        })
    } else {
        Ok(())
    }
}

pub fn enable_streaming(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
    address: u8,
) -> Result<(), OkError> {
    if hnd.get_wire_in(config.block_flow_control as i32)? & 1 != 1 {
        hnd.set_wire_in(config.block_flow_control as i32, 1);
        hnd.update_wire_ins();
    }
    write_bridge_bytes(hnd, config, &mk_u8(&[0x500 | (address as u16)]))
}

pub fn disable_streaming(hnd: &OkHandle, config: &OKHLSBridgeAddressConfig) -> Result<(), OkError> {
    write_bridge_bytes(hnd, config, &mk_u8(&[0xFFFF]))?;
    if hnd.get_wire_in(config.block_flow_control as i32)? & 1 != 0 {
        hnd.set_wire_in(config.block_flow_control as i32, 0);
        hnd.update_wire_ins();
    }
    Ok(())
}

pub fn stream_read(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
    num_words: usize,
) -> Result<Vec<u16>, OkError> {
    let mut data = vec![0_u8; num_words * 2];
    hnd.read_from_block_pipe_out(config.pipe_out as i32, 1024, &mut data)?;
    Ok(make_u16_buffer(&data))
}

pub fn drain_stream(
    hnd: &OkHandle,
    config: &OKHLSBridgeAddressConfig,
) -> Result<Vec<u16>, OkError> {
    let mut drain_complete = false;
    let mut ret = vec![];
    for _retry in 0..100 {
        hnd.update_wire_outs();
        let read_buffer_count = hnd.get_wire_out(config.words_avail as i32) as usize;
        if read_buffer_count == 0 {
            drain_complete = true;
            break;
        }
        let mut data = vec![0_u8; read_buffer_count * 2];
        hnd.read_from_pipe_out(config.pipe_out as i32, &mut data)?;
        ret.append(&mut data);
    }
    if !drain_complete {
        return Err(OkError {
            code: ok_ErrorCode_ok_Timeout,
        });
    }
    Ok(make_u16_buffer(&ret))
}
