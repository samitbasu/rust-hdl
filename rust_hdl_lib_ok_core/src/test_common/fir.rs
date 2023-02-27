use crate::core::prelude::*;
use crate::test_common::tools::ok_test_prelude;
use rust_hdl_lib_core::prelude::*;
use rust_hdl_lib_ok_frontpanel_sys::{make_u16_buffer, OkError};
use rust_hdl_lib_widgets::prelude::*;
use std::thread::sleep;
use std::time::Duration;

declare_sync_fifo!(OKTestFIFO, Signed<16>, 256, 1);

#[derive(LogicBlock)]
pub struct OpalKellyFIRTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub i_fifo: OKTestFIFO,
    pub o_fifo: OKTestFIFO,
    pub fir: MultiplyAccumulateSymmetricFiniteImpulseResponseFilter<6>,
    pub i_pipe: PipeIn,
    pub o_pipe: PipeOut,
    pub delay_read: DFF<Bit>,
    pub will_feed: Signal<Local, Bit>,
}

impl Logic for OpalKellyFIRTest {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        // connect the OK busses
        self.i_pipe.ok1.next = self.ok_host.ok1.val();
        self.o_pipe.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.i_pipe.ok2.val() | self.o_pipe.ok2.val();
        // Connect up the clocks.
        self.i_fifo.clock.next = self.ok_host.ti_clk.val();
        self.o_fifo.clock.next = self.ok_host.ti_clk.val();
        self.fir.clock.next = self.ok_host.ti_clk.val();
        self.delay_read.clock.next = self.ok_host.ti_clk.val();
        // Connect the input pipe to the input fifo
        self.i_fifo.data_in.next = signed_cast(self.i_pipe.dataout.val());
        self.i_fifo.write.next = self.i_pipe.write.val();
        // For now, bridge the two fifos
        self.fir.data_in.next = self.i_fifo.data_out.val();
        self.will_feed.next = !self.fir.busy.val() & !self.i_fifo.empty.val();
        self.fir.strobe_in.next = self.will_feed.val();
        self.i_fifo.read.next = self.will_feed.val();
        self.o_fifo.data_in.next = self.fir.data_out.val().get_bits::<16>(0_usize);
        self.o_fifo.write.next = self.fir.strobe_out.val();
        self.o_fifo.read.next = self.delay_read.q.val();
        self.delay_read.d.next = self.o_pipe.read.val();
        self.o_pipe.datain.next = unsigned_cast(self.o_fifo.data_out.val());
    }
}

impl OpalKellyFIRTest {
    pub fn new<B: OpalKellyBSP>() -> Self {
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            i_fifo: Default::default(),
            o_fifo: Default::default(),
            fir: MultiplyAccumulateSymmetricFiniteImpulseResponseFilter::new(&[
                1, -2, 3, -5, 7, -5, 3, -2, 1,
            ]),
            i_pipe: PipeIn::new(0x80),
            o_pipe: PipeOut::new(0xA0),
            delay_read: Default::default(),
            will_feed: Default::default(),
        }
    }
}

pub fn make_i16_buffer(data: &[u8]) -> Vec<i16> {
    make_u16_buffer(data).iter().map(|x| *x as i16).collect()
}

pub fn test_opalkelly_fir_runtime(bit_name: &str, serial_number: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_name, serial_number)?;
    let mut data = [0_u8; 256];
    data[128] = 0xFF;
    data[129] = 0xFF;
    let data16 = make_i16_buffer(&data);
    for (ndx, val) in data16.iter().enumerate() {
        if *val != 0 {
            println!("ival {} -> {}", ndx, val);
        }
    }
    hnd.write_to_pipe_in(0x80, &data)?;
    sleep(Duration::from_secs(1));
    let mut data_out = [0_u8; 256];
    hnd.read_from_pipe_out(0xA0, &mut data_out)?;
    let data_out = make_i16_buffer(&data_out);
    for (ndx, val) in data_out.iter().enumerate() {
        if *val != 0 {
            println!("val {} -> {}", ndx, val);
        }
    }
    Ok(())
}
