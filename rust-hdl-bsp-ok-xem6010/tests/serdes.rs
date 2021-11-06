use rust_hdl_core::prelude::*;
use rust_hdl_bsp_ok_xem6010::serdes::{DCMClockDoubler, ClockSplitter};
use rust_hdl_widgets::pulser::Pulser;
use rust_hdl_bsp_ok_xem6010::XEM6010;
use rust_hdl_bsp_ok_xem6010::pins::{xem_6010_base_clock, xem_6010_leds};
use std::time::Duration;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_test_core::target_path;

#[derive(LogicBlock)]
pub struct OpalKellySerdesTest {
    doubler: DCMClockDoubler,
    splitter: ClockSplitter<4, false>,
    raw_sys_clock: Signal<In, Clock>,
    doubled_clock: Signal<Local, Clock>,
    slow_pulser: Pulser,
    fast_pulser: Pulser,
    led: Signal<Out, Bits<8>>,
//    sim: Signal<Out, Clock>,
}

impl OpalKellySerdesTest {
    pub fn new() -> Self {
        /*
        let mut sim = Signal::default();
        sim.add_location(0, "V22");
        sim.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);*/
        Self {
            doubler: DCMClockDoubler::new(10.0),
            splitter: Default::default(),
            raw_sys_clock: xem_6010_base_clock(),
            doubled_clock: Default::default(),
            slow_pulser: Pulser::new(50_000_000, 2.0, Duration::from_secs(1)),
            fast_pulser: Pulser::new(200_000_000, 2.0, Duration::from_secs(1)),
            led: xem_6010_leds(),
//            sim,
        }
    }
}

impl Logic for OpalKellySerdesTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.doubler.clock_in.next = self.raw_sys_clock.val();
        self.doubled_clock.next = self.doubler.clock_out.val();
        //self.slow_pulser.clock.next = self.raw_sys_clock.val();
        self.fast_pulser.clock.next = self.doubled_clock.val();
        self.slow_pulser.enable.next = true;
        self.fast_pulser.enable.next = true;
        self.splitter.clock_in.next = self.doubled_clock.val();
        self.slow_pulser.clock.next = self.splitter.div_clock_out.val();
        //self.sim.next = self.doubled_clock.val();
        self.led.next =
            bit_cast::<8, 1>(self.slow_pulser.pulse.val().into()) |
                (bit_cast::<8, 1>(self.fast_pulser.pulse.val().into()) << 5_usize);
    }
}

#[test]
fn test_opalkelly_xem_6010_synth_clock_double() {
    let mut uut = OpalKellySerdesTest::new();
    uut.raw_sys_clock.connect();
    uut.connect_all();
    check_connected(&uut);
    XEM6010::synth(uut, target_path!("xem_6010/clock_double"))
}
