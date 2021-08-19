use crate::ok_tools::ok_test_prelude;
use rust_hdl_core::bits::bit_cast;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::ok_pipe::{BTPipeOut, PipeIn, PipeOut};
use rust_hdl_ok::prelude::*;
use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};
use rust_hdl_widgets::prelude::*;
use rust_hdl_widgets::ram::RAM;
use std::num::Wrapping;

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010PipeTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub accum: DFF<Bits<16>>,
    pub o_wire: WireOut<0x20>,
    pub i_pipe: PipeIn<0x80>,
}

impl OpalKellyXEM6010PipeTest {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: Default::default(),
            accum: DFF::new(0_u16.into()),
            o_wire: Default::default(),
            i_pipe: Default::default(),
        }
    }
}

impl Logic for OpalKellyXEM6010PipeTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        self.hi.link(&mut self.ok_host.hi);

        // Clock connections
        self.accum.clk.next = self.ok_host.ti_clk.val();

        // Bus connections
        self.o_wire.ok1.next = self.ok_host.ok1.val();
        self.i_pipe.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.o_wire.ok2.val() | self.i_pipe.ok2.val();

        // Logic
        self.accum.d.next = self.accum.q.val();
        if self.i_pipe.write.val() {
            self.accum.d.next = self.accum.q.val() + self.i_pipe.dataout.val();
        }
        self.o_wire.datain.next = self.accum.q.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_pipe() {
    let mut uut = OpalKellyXEM6010PipeTest::new();
    uut.hi.link_connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_pipe");
}

fn sum_vec(t: &[u16]) -> u16 {
    let mut ret = Wrapping(0_u16);
    for x in t {
        ret += Wrapping(*x);
    }
    ret.0
}

#[test]
fn test_opalkelly_xem_6010_pipe_in_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_pipe/top.bit")?;
    let data = (0..1024 * 1024)
        .map(|_| rand::random::<u8>())
        .collect::<Vec<_>>();
    let data_16 = make_u16_buffer(&data);
    let cpu_sum = sum_vec(&data_16);
    hnd.write_to_pipe_in(0x80, &data)?;
    hnd.update_wire_outs();
    let fpga_sum = hnd.get_wire_out(0x20);
    println!("CPU sum {:x}, FPGA sum {:x}", cpu_sum, fpga_sum);
    assert_eq!(cpu_sum, fpga_sum);
    Ok(())
}

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010PipeRAMTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub ram: RAM<Bits<16>, 8>,
    pub i_pipe: PipeIn<0x80>,
    pub o_pipe: PipeOut<0xA0>,
    pub read_address: DFF<Bits<8>>,
    pub write_address: DFF<Bits<8>>,
}

impl OpalKellyXEM6010PipeRAMTest {
    fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: Default::default(),
            ram: RAM::new(Default::default()),
            i_pipe: Default::default(),
            o_pipe: Default::default(),
            read_address: Default::default(),
            write_address: Default::default(),
        }
    }
}

impl Logic for OpalKellyXEM6010PipeRAMTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        self.hi.link(&mut self.ok_host.hi);

        // Clock connections
        self.read_address.clk.next = self.ok_host.ti_clk.val();
        self.write_address.clk.next = self.ok_host.ti_clk.val();
        self.ram.read_clock.next = self.ok_host.ti_clk.val();
        self.ram.write_clock.next = self.ok_host.ti_clk.val();

        // Bus connections
        self.i_pipe.ok1.next = self.ok_host.ok1.val();
        self.o_pipe.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.i_pipe.ok2.val() | self.o_pipe.ok2.val();

        // Data connections
        self.ram.read_address.next = self.read_address.q.val();
        self.ram.write_address.next = self.write_address.q.val();
        self.o_pipe.datain.next = self.ram.read_data.val();
        self.ram.write_data.next = self.i_pipe.dataout.val();
        self.ram.write_enable.next = self.i_pipe.write.val();

        // Advance the address counters
        self.write_address.d.next = self.write_address.q.val() + self.i_pipe.write.val();
        self.read_address.d.next = self.read_address.q.val() + self.o_pipe.read.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_pipe_ram() {
    let mut uut = OpalKellyXEM6010PipeRAMTest::new();
    uut.hi.link_connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_pipe_ram");
}

#[test]
fn test_opalkelly_xem_6010_pipe_ram_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_pipe_ram/top.bit")?;
    let data = (0..512).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
    hnd.write_to_pipe_in(0x80, &data)?;
    let mut out = vec![0_u8; 512];
    hnd.read_from_pipe_out(0xa0, &mut out)?;
    assert_eq!(data, out);
    let mut out2 = vec![0_u8; 512];
    hnd.read_from_pipe_out(0xa0, &mut out2)?;
    assert_eq!(data, out2);
    Ok(())
}

declare_sync_fifo!(OKTestFIFO, Bits<16>, 256, 1);

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010PipeFIFOTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub fifo: OKTestFIFO,
    pub i_pipe: PipeIn<0x80>,
    pub o_pipe: PipeOut<0xA0>,
    pub delay_read: DFF<Bit>,
}

impl OpalKellyXEM6010PipeFIFOTest {
    fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: Default::default(),
            fifo: Default::default(),
            i_pipe: Default::default(),
            o_pipe: Default::default(),
            delay_read: Default::default(),
        }
    }
}

impl Logic for OpalKellyXEM6010PipeFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        self.hi.link(&mut self.ok_host.hi);

        // Clock connections
        self.fifo.clock.next = self.ok_host.ti_clk.val();
        self.delay_read.clk.next = self.ok_host.ti_clk.val();

        // Bus connections
        self.i_pipe.ok1.next = self.ok_host.ok1.val();
        self.o_pipe.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.i_pipe.ok2.val() | self.o_pipe.ok2.val();

        // Data connections
        self.delay_read.d.next = self.o_pipe.read.val();
        self.fifo.read.next = self.delay_read.q.val();
        self.o_pipe.datain.next = self.fifo.data_out.val();
        self.fifo.write.next = self.i_pipe.write.val();
        self.fifo.data_in.next = self.i_pipe.dataout.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_pipe_fifo() {
    let mut uut = OpalKellyXEM6010PipeFIFOTest::new();
    uut.hi.sig_inout.connect();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_fifo");
}

#[test]
fn test_opalkelly_xem_6010_pipe_fifo_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_fifo/top.bit")?;
    let data = (0..512).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
    let orig_data_16 = make_u16_buffer(&data);
    hnd.write_to_pipe_in(0x80, &data)?;
    let mut out = vec![0_u8; 512];
    hnd.read_from_pipe_out(0xa0, &mut out)?;
    let copy_data_16 = make_u16_buffer(&out);
    assert_eq!(orig_data_16, copy_data_16);
    Ok(())
}

declare_async_fifo!(OKTestAFIFO, Bits<16>, 256, 1);

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010PipeAFIFOTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub fifo_in: OKTestAFIFO,
    pub fifo_out: OKTestAFIFO,
    pub i_pipe: PipeIn<0x80>,
    pub o_pipe: PipeOut<0xA0>,
    pub delay_read: DFF<Bit>,
    pub fast_clock: Signal<In, Clock>,
}

impl OpalKellyXEM6010PipeAFIFOTest {
    fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: Default::default(),
            fifo_in: Default::default(),
            fifo_out: Default::default(),
            i_pipe: Default::default(),
            o_pipe: Default::default(),
            delay_read: Default::default(),
            fast_clock: xem_6010_base_clock(),
        }
    }
}

impl Logic for OpalKellyXEM6010PipeAFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        self.hi.link(&mut self.ok_host.hi);

        // Clock connections
        self.fifo_in.read_clock.next = self.fast_clock.val();
        self.fifo_in.write_clock.next = self.ok_host.ti_clk.val();
        self.fifo_out.read_clock.next = self.ok_host.ti_clk.val();
        self.fifo_out.write_clock.next = self.fast_clock.val();
        self.delay_read.clk.next = self.ok_host.ti_clk.val();

        // Bus connections
        self.i_pipe.ok1.next = self.ok_host.ok1.val();
        self.o_pipe.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.i_pipe.ok2.val() | self.o_pipe.ok2.val();

        // Data connections
        // Input pipe connections
        self.fifo_in.write.next = self.i_pipe.write.val();
        self.fifo_in.data_in.next = self.i_pipe.dataout.val();
        // Output pipe connections
        self.fifo_out.read.next = self.delay_read.q.val();
        self.o_pipe.datain.next = self.fifo_out.data_out.val();
        self.delay_read.d.next = self.o_pipe.read.val();

        // Connect the two fifos...
        self.fifo_in.read.next = !self.fifo_in.empty.val() & !self.fifo_out.full.val();
        self.fifo_out.data_in.next = self.fifo_in.data_out.val() << 1_u32;
        self.fifo_out.write.next = !self.fifo_in.empty.val() && !self.fifo_out.full.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_pipe_afifo() {
    let mut uut = OpalKellyXEM6010PipeAFIFOTest::new();
    uut.hi.link_connect();
    uut.fast_clock.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_afifo");
}

#[test]
fn test_opalkelly_xem_6010_pipe_afifo_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_afifo/top.bit")?;
    let data = (0..512).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
    let orig_data_16 = make_u16_buffer(&data);
    hnd.write_to_pipe_in(0x80, &data)?;
    let mut out = vec![0_u8; 512];
    hnd.read_from_pipe_out(0xa0, &mut out)?;
    let copy_data_16 = make_u16_buffer(&out);
    let mod_data_16 = orig_data_16
        .iter()
        .map(|x| Wrapping(*x) << 1)
        .map(|x| x.0)
        .collect::<Vec<_>>();
    assert_eq!(mod_data_16, copy_data_16);
    Ok(())
}

declare_async_fifo!(OKTestAFIFO2, Bits<16>, 1024, 256);

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010BTPipeOutTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub fifo_out: OKTestAFIFO2,
    pub o_pipe: BTPipeOut<0xA0>,
    pub delay_read: DFF<Bit>,
    pub fast_clock: Signal<In, Clock>,
    pub counter: DFF<Bits<16>>,
    pub strobe: Strobe<100_000_000, 32>,
    pub can_run: Signal<Local, Bit>,
    pub led: Signal<Out, Bits<8>>,
}

impl Logic for OpalKellyXEM6010BTPipeOutTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Link the interfaces
        self.hi.link(&mut self.ok_host.hi);

        // Connect the clocks
        // Read side objects
        self.fifo_out.read_clock.next = self.ok_host.ti_clk.val();
        self.delay_read.clk.next = self.ok_host.ti_clk.val();
        // Write side objects
        self.fifo_out.write_clock.next = self.fast_clock.val();
        self.counter.clk.next = self.fast_clock.val();
        self.strobe.clock.next = self.fast_clock.val();

        // Connect the ok1 and ok2 busses
        self.o_pipe.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.o_pipe.ok2.val();

        self.can_run.next = !self.fifo_out.full.val();

        // Set up the counter
        self.counter.d.next =
            self.counter.q.val() + (self.strobe.strobe.val() & self.can_run.val());

        // Enable the strobe
        self.strobe.enable.next = self.can_run.val();

        // Connect the counter to the fifo
        self.fifo_out.data_in.next = self.counter.q.val();
        self.fifo_out.write.next = self.strobe.strobe.val() & self.can_run.val();

        // Connect the delay counter for the fifo
        self.delay_read.d.next = self.o_pipe.read.val();
        self.fifo_out.read.next = self.delay_read.q.val();

        // Connect the pipe to the output of the fifo
        self.o_pipe.datain.next = self.fifo_out.data_out.val();
        // Connect the enable for the pipe to the not-almost-empty for the fifo
        self.o_pipe.ready.next = !self.fifo_out.almost_empty.val();

        // Signal the LEDs
        self.led.next = !(bit_cast::<8, 1>(self.fifo_out.empty.val().into())
            | (bit_cast::<8, 1>(self.fifo_out.full.val().into()) << 1_usize)
            | (bit_cast::<8, 1>(self.fifo_out.almost_empty.val().into()) << 2_usize)
            | (bit_cast::<8, 1>(self.fifo_out.almost_full.val().into()) << 3_usize)
            | (bit_cast::<8, 1>(self.fifo_out.overflow.val().into()) << 4_usize)
            | (bit_cast::<8, 1>(self.fifo_out.underflow.val().into()) << 5_usize));
    }
}

impl OpalKellyXEM6010BTPipeOutTest {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: Default::default(),
            fifo_out: Default::default(),
            o_pipe: Default::default(),
            delay_read: Default::default(),
            fast_clock: xem_6010_base_clock(),
            counter: Default::default(),
            strobe: Strobe::new(1_000_000.0),
            can_run: Default::default(),
            led: xem_6010_leds(),
        }
    }
}

#[test]
fn test_opalkelly_xem_6010_btpipe() {
    let mut uut = OpalKellyXEM6010BTPipeOutTest::new();
    uut.hi.link_connect();
    uut.fast_clock.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_btpipe");
}

#[test]
fn test_opalkelly_xem_6010_btpipe_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_btpipe/top.bit")?;
    // Read the data in 256*2 = 512 byte blocks
    let mut data = vec![0_u8; 1024 * 128];
    hnd.read_from_block_pipe_out(0xA0, 256, &mut data).unwrap();
    let data_shorts = make_u16_buffer(&data);
    for (ndx, val) in data_shorts.iter().enumerate() {
        assert_eq!(((ndx as u128) & 0xFFFF_u128) as u16, *val);
    }
    Ok(())
}
