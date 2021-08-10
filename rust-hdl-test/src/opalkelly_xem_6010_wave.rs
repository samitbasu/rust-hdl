use rust_hdl_core::prelude::*;
use rust_hdl_ok::MHz48;
use rust_hdl_ok::ok_hi::OpalKellyHostInterface;
use rust_hdl_ok::ok_host::OpalKellyHost;
use rust_hdl_ok::pins::xem_6010_leds;

use crate::alchitry_cu_pwm_vec_srom::FaderWithSyncROM;

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010Wave {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>, Async>,
    pub local: Signal<Local, Bits<8>, Async>,
    pub faders: [FaderWithSyncROM<MHz48>; 8],
}

impl Logic for OpalKellyXEM6010Wave {
    #[hdl_gen]
    fn update(&mut self) {
        self.ok_host.hi.sig_in.next = self.hi.sig_in.val();
        self.hi.sig_out.next = self.ok_host.hi.sig_out.val();
        link!(self.hi.sig_inout, self.ok_host.hi.sig_inout);
        link!(self.hi.sig_aa, self.ok_host.hi.sig_aa);
        for i in 0_usize..8_usize {
            self.faders[i].clock.next = self.ok_host.ti_clk.val();
            self.faders[i].enable.next = true.into();
        }
        self.local.next = 0x00_u8.into();
        for i in 0_usize..8_usize {
            self.local.next = self.local
                .val()
                .raw()
                .replace_bit(i, !self.faders[i].active.val().raw())
                .into();
        }
        self.led.next = self.local.val();
    }
}

impl Default for OpalKellyXEM6010Wave {
    fn default() -> Self {
        let faders: [FaderWithSyncROM<MHz48>; 8] = [
            FaderWithSyncROM::new(0),
            FaderWithSyncROM::new(18),
            FaderWithSyncROM::new(36),
            FaderWithSyncROM::new(54),
            FaderWithSyncROM::new(72),
            FaderWithSyncROM::new(90),
            FaderWithSyncROM::new(108),
            FaderWithSyncROM::new(128),
        ];
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: Default::default(),
            local: Signal::default(),
            faders,
            led: xem_6010_leds(),
        }
    }
}

#[test]
fn test_opalkelly_xem_6010_wave() {
    let mut uut = OpalKellyXEM6010Wave::default();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_inout.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_wave");
}