use std::time::Instant;

use rust_hdl_bsp_ok_xem7010::ddr_fifo7::DDR7FIFO;
use rust_hdl_bsp_ok_xem7010::download::OpalKellyDDRBackedDownloadFIFO7Series;
use rust_hdl_bsp_ok_xem7010::pins::xem_7010_leds;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_test_ok_common::prelude::*;
use rust_hdl_widgets::dff::DFF;
use rust_hdl_widgets::prelude::*;
use rust_hdl_bsp_ok_xem7010::XEM7010;
use rust_hdl_test_core::target_path;
use rust_hdl_bsp_ok_xem7010::mcb_if::MCBInterface4GDDR3;

#[derive(LogicBlock)]
struct OpalKellyDDR7Test {
    mcb: MCBInterface4GDDR3,
    hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    ddr_fifo: DDR7FIFO<16>,
    sys_clock_p: Signal<In, Clock>,
    sys_clock_n: Signal<In, Clock>,
    reset: WireIn,
    pipe_in: PipeIn,
    pipe_out: PipeOut,
    delay_read: DFF<Bit>,
    leds: Signal<Out, Bits<8>>,
}

impl Default for OpalKellyDDR7Test {
    fn default() -> Self {
        Self {
            mcb: MCBInterface4GDDR3::xem_7010(),
            hi: OpalKellyHostInterface::xem_7010(),
            ok_host: OpalKellyHost::xem_7010(),
            ddr_fifo: DDR7FIFO::default(),
            sys_clock_p: Default::default(),
            sys_clock_n: Default::default(),
            reset: WireIn::new(0x00),
            pipe_in: PipeIn::new(0x80),
            pipe_out: PipeOut::new(0xA0),
            delay_read: Default::default(),
            leds: xem_7010_leds(),
        }
    }
}

impl Logic for OpalKellyDDR7Test {
    #[hdl_gen]
    fn update(&mut self) {
        // DDR and clocks
        self.mcb.link(&mut self.ddr_fifo.mcb);
        self.hi.link(&mut self.ok_host.hi);
        self.ddr_fifo.sys_clock_p.next = self.sys_clock_p.val();
        self.ddr_fifo.sys_clock_n.next = self.sys_clock_n.val();
        // Pipe in connects to DDR FIFO input
        self.ddr_fifo.data_in.next = self.pipe_in.dataout.val();
        self.ddr_fifo.write.next = self.pipe_in.write.val();
        self.ddr_fifo.write_clock.next = self.ok_host.ti_clk.val();
        // Pipe out connects to DDR FIFO output
        self.pipe_out.datain.next = self.ddr_fifo.data_out.val();
        self.delay_read.d.next = self.pipe_out.read.val();
        self.ddr_fifo.read.next = self.delay_read.q.val();
        self.ddr_fifo.read_clock.next = self.ok_host.ti_clk.val();
        // Connect the reset
        self.ddr_fifo.reset.next = self.reset.dataout.val().any();
        // Wire the OK busses
        self.reset.ok1.next = self.ok_host.ok1.val();
        self.pipe_in.ok1.next = self.ok_host.ok1.val();
        self.pipe_out.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.pipe_in.ok2.val() | self.pipe_out.ok2.val();
        self.delay_read.clk.next = self.ok_host.ti_clk.val();
        self.leds.next = !self.ddr_fifo.status.val();
    }
}

#[test]
fn test_synthesis_of_xem7010_ddr() {
    let mut uut = OpalKellyDDR7Test::default();
    uut.hi.link_connect_dest();
    uut.mcb.link_connect_dest();
    uut.sys_clock_p.connect();
    uut.sys_clock_n.connect();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/ddr"));
    test_opalkelly_xem_7010_ddr_runtime().unwrap()
}

#[cfg(test)]
fn test_opalkelly_xem_7010_ddr_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude(target_path!("xem_7010/ddr/top.bit"))?;
    hnd.reset_firmware(0);
    let test_size = 1024 * 1024 * 10;
    let mut data = (0..test_size)
        .map(|_| rand::random::<u8>())
        .collect::<Vec<_>>();
    let now = Instant::now();
    hnd.write_to_pipe_in(0x80, &data).unwrap();
    let elapsed = Instant::now() - now;
    println!("Elapsed write time: {}", elapsed.as_micros());
    let mut out_data = vec![0; test_size];
    let now = Instant::now();
    hnd.read_from_pipe_out(0xA0, &mut out_data).unwrap();
    let elapsed = Instant::now() - now;
    println!(
        "Elapsed read time: {} -> {}",
        elapsed.as_micros(),
        (test_size as f64 * 8.0) / (elapsed.as_micros() as f64 / 1.0e6)
    );
    for i in 0..data.len() {
        assert_eq!(data[i], out_data[i]);
    }
    Ok(())
}
