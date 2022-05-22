use rust_hdl_ok_core::core::prelude::*;

use rust_hdl_ok_core::xem6010::pins::{xem_6010_base_clock, xem_6010_leds};

use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
use rust_hdl_ok_core::xem6010::XEM6010;

use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};

mod test_common;

use test_common::tools::ok_test_prelude;

declare_async_fifo!(OKTestAFIFO2, Bits<16>, 1024, 256);

#[derive(LogicBlock)]
pub struct OpalKellyBTPipeOutTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub fifo_out: OKTestAFIFO2,
    pub o_pipe: BTPipeOut,
    pub delay_read: DFF<Bit>,
    pub fast_clock: Signal<In, Clock>,
    pub counter: DFF<Bits<16>>,
    pub strobe: Strobe<32>,
    pub can_run: Signal<Local, Bit>,
    pub led: Signal<Out, Bits<8>>,
}

impl Logic for OpalKellyBTPipeOutTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Link the interfaces
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);

        // Connect the clocks
        // Read side objects
        self.fifo_out.read_clock.next = self.ok_host.ti_clk.val();
        self.delay_read.clock.next = self.ok_host.ti_clk.val();
        // Write side objects
        self.fifo_out.write_clock.next = self.fast_clock.val();
        self.counter.clock.next = self.fast_clock.val();
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

impl OpalKellyBTPipeOutTest {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::xem_6010(),
            fifo_out: Default::default(),
            o_pipe: BTPipeOut::new(0xA0),
            delay_read: Default::default(),
            fast_clock: xem_6010_base_clock(),
            counter: Default::default(),
            strobe: Strobe::new(100_000_000, 1_000_000.0),
            can_run: Default::default(),
            led: xem_6010_leds(),
        }
    }
}

#[test]
fn test_opalkelly_xem_6010_synth_btpipe() {
    let mut uut = OpalKellyBTPipeOutTest::new();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/btpipe"));
    test_opalkelly_xem_6010_btpipe_runtime().unwrap();
}

#[cfg(test)]
fn test_opalkelly_xem_6010_btpipe_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude(target_path!("xem_6010/btpipe/top.bit"))?;
    // Read the data in 256*2 = 512 byte blocks
    let mut data = vec![0_u8; 1024 * 128];
    hnd.read_from_block_pipe_out(0xA0, 256, &mut data).unwrap();
    let data_shorts = make_u16_buffer(&data);
    for (ndx, val) in data_shorts.iter().enumerate() {
        assert_eq!(((ndx as u128) & 0xFFFF_u128) as u16, *val);
    }
    Ok(())
}
