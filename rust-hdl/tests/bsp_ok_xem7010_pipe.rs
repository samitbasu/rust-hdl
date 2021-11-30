use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
mod test_common;

use rust_hdl::bsp::ok_xem7010::pins::{xem_7010_leds, xem_7010_neg_clock, xem_7010_pos_clock};
use rust_hdl::bsp::ok_xem7010::sys_clock::OpalKellySystemClock7;
use rust_hdl::bsp::ok_xem7010::XEM7010;
#[cfg(feature = "frontpanel")]
use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};
#[cfg(feature = "frontpanel")]
use test_common::pipe::*;
#[cfg(feature = "frontpanel")]
use test_common::tools::*;

declare_async_fifo!(OKTestAFIFO2, Bits<16>, 1024, 256);

#[derive(LogicBlock)]
pub struct OpalKellyBTPipeOut7Test {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub fifo_out: OKTestAFIFO2,
    pub o_pipe: BTPipeOut,
    pub delay_read: DFF<Bit>,
    pub clock_p: Signal<In, Clock>,
    pub clock_n: Signal<In, Clock>,
    pub fast_clock: Signal<Local, Clock>,
    pub clock_div: OpalKellySystemClock7,
    pub counter: DFF<Bits<16>>,
    pub strobe: Strobe<32>,
    pub can_run: Signal<Local, Bit>,
    pub led: Signal<Out, Bits<8>>,
}

impl Logic for OpalKellyBTPipeOut7Test {
    #[hdl_gen]
    fn update(&mut self) {
        // Link the interfaces
        self.hi.link(&mut self.ok_host.hi);

        // Connect the clock up
        self.clock_div.clock_p.next = self.clock_p.val();
        self.clock_div.clock_n.next = self.clock_n.val();
        self.fast_clock.next = self.clock_div.sys_clock.val();

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

impl OpalKellyBTPipeOut7Test {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_7010(),
            ok_host: OpalKellyHost::xem_7010(),
            fifo_out: Default::default(),
            o_pipe: BTPipeOut::new(0xA0),
            delay_read: Default::default(),
            clock_p: xem_7010_pos_clock(),
            clock_n: xem_7010_neg_clock(),
            fast_clock: Default::default(),
            clock_div: Default::default(),
            counter: Default::default(),
            strobe: Strobe::new(100_000_000, 1_000_000.0),
            can_run: Default::default(),
            led: xem_7010_leds(),
        }
    }
}

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_7010_synth_btpipe() {
    let mut uut = OpalKellyBTPipeOut7Test::new();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/btpipe"));
    test_opalkelly_xem_7010_btpipe_runtime().unwrap();
}

#[cfg(feature = "frontpanel")]
#[cfg(test)]
fn test_opalkelly_xem_7010_btpipe_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude(target_path!("xem_7010/btpipe/top.bit"))?;
    // Read the data in 256*2 = 512 byte blocks
    let mut data = vec![0_u8; 1024 * 128];
    hnd.read_from_block_pipe_out(0xA0, 256, &mut data).unwrap();
    let data_shorts = make_u16_buffer(&data);
    for (ndx, val) in data_shorts.iter().enumerate() {
        assert_eq!(((ndx as u128) & 0xFFFF_u128) as u16, *val);
    }
    Ok(())
}
