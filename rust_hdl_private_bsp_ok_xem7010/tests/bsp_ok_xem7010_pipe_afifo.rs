use rust_hdl::prelude::*;
use rust_hdl_private_bsp_ok_xem7010::xem7010::sys_clock::OpalKellySystemClock7;
use rust_hdl_private_bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl_private_ok_core::core::prelude::*;
use rust_hdl_private_ok_core::test_common::pipe::test_opalkelly_pipe_afifo_runtime;

declare_async_fifo!(OKTestAFIFO, Bits<16>, 256, 1);

#[derive(LogicBlock)]
pub struct OpalKellyPipeAFIFOTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub fifo_in: OKTestAFIFO,
    pub fifo_out: OKTestAFIFO,
    pub i_pipe: PipeIn,
    pub o_pipe: PipeOut,
    pub delay_read: DFF<Bit>,
    pub fast_clock: Signal<Local, Clock>,
    pub clock_div: OpalKellySystemClock7,
    pub clock_p: Signal<In, Clock>,
    pub clock_n: Signal<In, Clock>,
}

impl OpalKellyPipeAFIFOTest {
    fn new<B: OpalKellyBSP>() -> Self {
        let clk = B::clocks();
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            fifo_in: Default::default(),
            fifo_out: Default::default(),
            i_pipe: PipeIn::new(0x80),
            o_pipe: PipeOut::new(0xA0),
            delay_read: Default::default(),
            fast_clock: Default::default(),
            clock_div: Default::default(),
            clock_p: clk[0].clone(),
            clock_n: clk[1].clone(),
        }
    }
}

impl Logic for OpalKellyPipeAFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);

        // Clock connections
        self.clock_div.clock_p.next = self.clock_p.val();
        self.clock_div.clock_n.next = self.clock_n.val();
        self.fast_clock.next = self.clock_div.sys_clock.val();
        self.fifo_in.read_clock.next = self.fast_clock.val();
        self.fifo_in.write_clock.next = self.ok_host.ti_clk.val();
        self.fifo_out.read_clock.next = self.ok_host.ti_clk.val();
        self.fifo_out.write_clock.next = self.fast_clock.val();
        self.delay_read.clock.next = self.ok_host.ti_clk.val();

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
        self.fifo_out.data_in.next = self.fifo_in.data_out.val() << 1;
        self.fifo_out.write.next = !self.fifo_in.empty.val() && !self.fifo_out.full.val();
    }
}

#[test]
fn test_opalkelly_xem_7010_synth_pipe_afifo() {
    let mut uut = OpalKellyPipeAFIFOTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.fast_clock.connect();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/pipe_afifo"));
    test_opalkelly_pipe_afifo_runtime(
        target_path!("xem_7010/pipe_afifo/top.bit"),
        env!("XEM7010_SERIAL"),
    )
    .unwrap()
}
