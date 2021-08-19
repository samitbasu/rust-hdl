use crate::ok_tools::ok_test_prelude;
use rust_hdl_core::bits::bit_cast;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::ddr_fifo::DDRFIFO;
use rust_hdl_ok::mcb_if::MCBInterface;
use rust_hdl_ok::ok_hi::OpalKellyHostInterface;
use rust_hdl_ok::ok_host::OpalKellyHost;
use rust_hdl_ok::ok_pipe::BTPipeOut;
use rust_hdl_ok::ok_wire::WireIn;
use rust_hdl_ok::pins::{xem_6010_base_clock, xem_6010_leds};
use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};
use rust_hdl_widgets::prelude::*;
use std::thread::sleep;
use std::time::{Duration, Instant};

declare_sync_fifo!(DFIFO, Bits<16>, 8192, 512);

const BENCH_CLOCK: u64 = 156_256_000;

#[derive(LogicBlock)]
struct OpalKellyDDRFIFOStressTest {
    mcb: MCBInterface,
    hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    ddr_fifo: DDRFIFO,
    count_in: DFF<Bits<32>>,
    fifo_out: DFIFO,
    o_pipe: BTPipeOut<0xA0>,
    read_delay: DFF<Bit>,
    raw_sys_clock: Signal<In, Clock>,
    strobe: Strobe<BENCH_CLOCK, 32>,
    will_write: Signal<Local, Bit>,
    will_transfer: Signal<Local, Bit>,
    led: Signal<Out, Bits<8>>,
    reset: WireIn<0x0>,
    enable: WireIn<0x1>,
    toggle: DFF<Bit>,
    pulse: Signal<Out, Bit>,
}

impl Default for OpalKellyDDRFIFOStressTest {
    fn default() -> Self {
        let mut pulse = Signal::default();
        pulse.add_location(0, "A11");
        pulse.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        Self {
            mcb: MCBInterface::xem_6010(),
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::default(),
            ddr_fifo: Default::default(),
            count_in: Default::default(),
            fifo_out: Default::default(),
            o_pipe: Default::default(),
            read_delay: Default::default(),
            raw_sys_clock: xem_6010_base_clock(),
            strobe: Strobe::new(12_000_000.0),
            will_write: Default::default(),
            will_transfer: Default::default(),
            led: xem_6010_leds(),
            reset: Default::default(),
            enable: Default::default(),
            toggle: Default::default(),
            pulse,
        }
    }
}

impl Logic for OpalKellyDDRFIFOStressTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        self.mcb.link(&mut self.ddr_fifo.mcb);
        self.ddr_fifo.reset.next = self.reset.dataout.val().any();
        self.ddr_fifo.raw_sys_clock.next = self.raw_sys_clock.val();
        self.count_in.clk.next = self.ddr_fifo.o_clock.val();
        self.fifo_out.clock.next = self.ok_host.ti_clk.val();
        self.read_delay.clk.next = self.ok_host.ti_clk.val();
        self.strobe.clock.next = self.ddr_fifo.o_clock.val();
        self.toggle.clk.next = self.ddr_fifo.o_clock.val();
        self.ddr_fifo.write_clock.next = self.ddr_fifo.o_clock.val();
        self.ddr_fifo.read_clock.next = self.ok_host.ti_clk.val();
        // Data source - counts on each strobe pulse and writes it to the input FIFO.
        self.will_write.next =
            self.strobe.strobe.val() & !self.ddr_fifo.full.val() & self.enable.dataout.val().any();
        self.count_in.d.next = self.count_in.q.val() + self.will_write.val();
        self.ddr_fifo.data_in.next = self.count_in.q.val();
        self.ddr_fifo.write.next = self.will_write.val();
        // Link the DDR fifo to the output fifo
        self.will_transfer.next = !self.ddr_fifo.empty.val() & !self.fifo_out.full.val();
        self.fifo_out.write.next = self.will_transfer.val();
        self.ddr_fifo.read.next = self.will_transfer.val();
        // TODO - fix this so we get the full 32 bits...
        self.fifo_out.data_in.next = bit_cast::<16, 32>(self.ddr_fifo.data_out.val());
        self.fifo_out.read.next = self.read_delay.q.val();
        self.read_delay.d.next = self.o_pipe.read.val();
        self.o_pipe.ready.next = !self.fifo_out.almost_empty.val();
        self.o_pipe.datain.next = self.fifo_out.data_out.val();
        self.o_pipe.ok1.next = self.ok_host.ok1.val();
        self.reset.ok1.next = self.ok_host.ok1.val();
        self.enable.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.o_pipe.ok2.val();
        self.strobe.enable.next = self.enable.dataout.val().any();
        self.led.next = !self.ddr_fifo.status.val();
        self.toggle.d.next = self.toggle.q.val();
        if self.strobe.strobe.val() {
            self.toggle.d.next = !self.toggle.q.val();
        }
        self.pulse.next = self.toggle.q.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_ddr_stress() {
    let mut uut = OpalKellyDDRFIFOStressTest::default();
    uut.hi.link_connect();
    uut.mcb.link_connect();
    uut.ddr_fifo.mcb.link_connect();
    uut.raw_sys_clock.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_ddr_stress");
}

#[test]
fn test_opalkelly_xem_6010_ddr_stress_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_ddr_stress/top.bit")?;
    hnd.reset_firmware(0);
    sleep(Duration::from_millis(100));
    hnd.set_wire_in(1, 1);
    hnd.update_wire_ins();
    // Read the data in 256*2 = 512 byte blocks
    let mut counter = 0;
    let mut drain_count = 0;
    let mut drain = true;
    for _ in 0..8 {
        let mut data = vec![0_u8; 1024 * 1024];
        let now = Instant::now();
        hnd.read_from_block_pipe_out(0xA0, 256, &mut data).unwrap();
        let elapsed = (Instant::now() - now).as_micros();
        println!(
            "Download rate is {} mbps",
            (data.len() as f32 * 8.0) / (elapsed as f32 * 1e-6) / 1e6
        );
        let data_shorts = make_u16_buffer(&data);
        for (ndx, val) in data_shorts.iter().enumerate() {
            if drain & (*val != 0) {
                drain_count += 1;
            } else if drain {
                println!("Drain completed with {} elements", drain_count);
                drain = false;
                counter += 1;
            } else {
                assert_eq!(((counter as u128) & 0xFFFF_u128) as u16, *val);
                counter += 1;
            }
        }
    }
    hnd.set_wire_in(1, 0);
    hnd.update_wire_ins();
    hnd.close();
    Ok(())
}
