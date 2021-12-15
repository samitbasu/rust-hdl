use rust_hdl::bsp::ok_core::ok_hi::OpalKellyHostInterface;
use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::bsp::ok_xem6010::XEM6010;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
pub struct OpalKellyTristateBuffer {
    pub bus_pin: Signal<InOut, Bit>,
    pub buffer: TristateBuffer<Bit>,
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub control: WireIn,
    pub readout: WireOut,
}

impl Logic for OpalKellyTristateBuffer {
    #[hdl_gen]
    fn update(&mut self) {
        self.bus_pin.link(&mut self.buffer.bus);
        self.hi.link(&mut self.ok_host.hi);
        self.control.ok1.next = self.ok_host.ok1.val();
        self.readout.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.readout.ok2.val();
        self.buffer.write_enable.next = self.control.dataout.val().get_bit(0);
        self.buffer.write_data.next = self.control.dataout.val().get_bit(1);
        self.readout.datain.next = bit_cast::<16, 1>(self.buffer.read_data.val().into());
    }
}

impl Default for OpalKellyTristateBuffer {
    fn default() -> Self {
        let mut x = Signal::default();
        x.add_location(0, "K20");
        x.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        Self {
            bus_pin: x,
            buffer: Default::default(),
            hi: XEM6010::hi(),
            ok_host: XEM6010::ok_host(),
            control: WireIn::new(0),
            readout: WireOut::new(0x20),
        }
    }
}

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_6010_synth_tribuffer() {
    let mut uut = OpalKellyTristateBuffer::default();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/tristate"));
}
