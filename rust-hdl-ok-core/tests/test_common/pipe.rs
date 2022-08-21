use std::num::Wrapping;

use crate::test_common::tools::ok_test_prelude;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
use rust_hdl_ok_core::core::prelude::*;
use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};

declare_sync_fifo!(OKTestFIFO, Bits<16>, 256, 1);

declare_async_fifo!(OKTestAFIFO2, Bits<16>, 1024, 256);

#[derive(LogicBlock)]
pub struct OpalKellyPipeTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub accum: DFF<Bits<16>>,
    pub o_wire: WireOut,
    pub i_pipe: PipeIn,
}

impl OpalKellyPipeTest {
    #[cfg(test)]
    pub fn new<B: OpalKellyBSP>() -> Self {
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            accum: Default::default(),
            o_wire: WireOut::new(0x20),
            i_pipe: PipeIn::new(0x80),
        }
    }
}

impl Logic for OpalKellyPipeTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);

        // Clock connections
        self.accum.clock.next = self.ok_host.ti_clk.val();

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

#[cfg(test)]
fn sum_vec(t: &[u16]) -> u16 {
    let mut ret = Wrapping(0_u16);
    for x in t {
        ret += Wrapping(*x);
    }
    ret.0
}

#[cfg(test)]
pub fn test_opalkelly_pipe_in_runtime(bit_name: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_name)?;
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
pub struct OpalKellyPipeRAMTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub ram: RAM<Bits<16>, 8>,
    pub i_pipe: PipeIn,
    pub o_pipe: PipeOut,
    pub read_address: DFF<Bits<8>>,
    pub write_address: DFF<Bits<8>>,
}

impl OpalKellyPipeRAMTest {
    #[cfg(test)]
    pub fn new<B: OpalKellyBSP>() -> Self {
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            ram: RAM::new(Default::default()),
            i_pipe: PipeIn::new(0x80),
            o_pipe: PipeOut::new(0xA0),
            read_address: Default::default(),
            write_address: Default::default(),
        }
    }
}

impl Logic for OpalKellyPipeRAMTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);

        // Clock connections
        self.read_address.clock.next = self.ok_host.ti_clk.val();
        self.write_address.clock.next = self.ok_host.ti_clk.val();
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

#[cfg(test)]
pub fn test_opalkelly_pipe_ram_runtime(bitfile: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bitfile)?;
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

#[derive(LogicBlock)]
pub struct OpalKellyPipeFIFOTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub fifo: OKTestFIFO,
    pub i_pipe: PipeIn,
    pub o_pipe: PipeOut,
    pub delay_read: DFF<Bit>,
}

impl OpalKellyPipeFIFOTest {
    #[cfg(test)]
    pub fn new<B: OpalKellyBSP>() -> Self {
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            fifo: Default::default(),
            i_pipe: PipeIn::new(0x80),
            o_pipe: PipeOut::new(0xA0),
            delay_read: Default::default(),
        }
    }
}

impl Logic for OpalKellyPipeFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);

        // Clock connections
        self.fifo.clock.next = self.ok_host.ti_clk.val();
        self.delay_read.clock.next = self.ok_host.ti_clk.val();

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

#[cfg(test)]
pub fn test_opalkelly_pipe_fifo_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
    let data = (0..512).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
    let orig_data_16 = make_u16_buffer(&data);
    hnd.write_to_pipe_in(0x80, &data)?;
    let mut out = vec![0_u8; 512];
    hnd.read_from_pipe_out(0xa0, &mut out)?;
    let copy_data_16 = make_u16_buffer(&out);
    assert_eq!(orig_data_16, copy_data_16);
    Ok(())
}

#[cfg(test)]
pub fn test_opalkelly_pipe_afifo_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
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
