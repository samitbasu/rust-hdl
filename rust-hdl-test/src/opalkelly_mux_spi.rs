use crate::ad7193_sim::AD7193Config;
use crate::muxed_ad7193_sim::MuxedAD7193Simulators;
use crate::ok_tools::{ok_do_spi_txn, ok_reg_read, ok_reg_write, ok_test_prelude};
use rust_hdl_core::prelude::*;
use rust_hdl_ok::bsp::{OpalKellyBSP, XEM6010, XEM7010};
use rust_hdl_ok::ok_hi::OpalKellyHostInterface;
use rust_hdl_ok::ok_host::OpalKellyHost;
use rust_hdl_ok::ok_wire::WireIn;
use rust_hdl_ok::spi::OKSPIMaster;
use rust_hdl_ok_frontpanel_sys::OkError;
use std::thread::sleep;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct OpalKellySPIMuxTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub mux_adc: MuxedAD7193Simulators,
    pub spi: OKSPIMaster,
    pub addr: WireIn,
}

impl Logic for OpalKellySPIMuxTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        // Connect the clocks...
        self.mux_adc.clock.next = self.ok_host.ti_clk.val();
        self.spi.clock.next = self.ok_host.ti_clk.val();
        // Connect the SPI bus
        self.mux_adc.mosi.next = self.spi.wires.mosi.val();
        self.mux_adc.msel.next = self.spi.wires.msel.val();
        self.mux_adc.mclk.next = self.spi.wires.mclk.val();
        self.spi.wires.miso.next = self.mux_adc.miso.val();
        // Connect the ok busses
        self.spi.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.spi.ok2.val();
        self.addr.ok1.next = self.ok_host.ok1.val();
        // Connect the addr to the mux select
        self.mux_adc.addr.next = bit_cast::<3, 16>(self.addr.dataout.val());
    }
}

impl OpalKellySPIMuxTest {
    fn new<B: OpalKellyBSP>() -> Self {
        let adc_config = AD7193Config::hw();
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            mux_adc: MuxedAD7193Simulators::new(adc_config),
            spi: OKSPIMaster::new(Default::default(), adc_config.spi),
            addr: WireIn::new(0x03),
        }
    }
}

#[test]
fn test_opalkelly_xem_6010_mux_spi() {
    let mut uut = OpalKellySPIMuxTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    crate::ok_tools::synth_obj_6010(uut, "xem_6010_mux_spi");
}

#[test]
fn test_opalkelly_xem_7010_mux_spi() {
    let mut uut = OpalKellySPIMuxTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    crate::ok_tools::synth_obj_7010(uut, "xem_7010_mux_spi");
}

#[cfg(test)]
fn test_opalkelly_mux_spi_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
    for addr in 0..8 {
        hnd.set_wire_in(3, addr);
        hnd.update_wire_ins();
        sleep(Duration::from_millis(100));
        ok_do_spi_txn(&hnd, 64, 0xFFFFFFFFFFFFFFFF_u64, false).unwrap();
        let expected = [0x40, 0x80060, 0x117, 0, 0xa2, 0, 0x800000, 0x5544d0];
        for reg in 0..8 {
            let x = ok_reg_read(&hnd, reg)?;
            println!("Read of reg {} is {:x}", reg, x);
            assert_eq!(x, expected[reg as usize]);
        }
    }
    // Write IDs to each one
    for addr in 0..8 {
        hnd.set_wire_in(3, addr);
        hnd.update_wire_ins();
        sleep(Duration::from_millis(100));
        ok_reg_write(&hnd, 5, 0xd0 + addr as u64)?;
    }
    // Read the IDs back
    for addr in 0..8 {
        hnd.set_wire_in(3, addr);
        hnd.update_wire_ins();
        sleep(Duration::from_millis(100));
        let x = ok_reg_read(&hnd, 5)?;
        assert_eq!(x, 0xd0 + addr as u64);
    }
    hnd.close();
    Ok(())
}

#[test]
fn test_opalkelly_xem_6010_mux_spi_runtime() -> Result<(), OkError> {
    test_opalkelly_mux_spi_runtime("xem_6010_mux_spi/top.bit")
}

#[test]
fn test_opalkelly_xem_7010_mux_spi_runtime() -> Result<(), OkError> {
    test_opalkelly_mux_spi_runtime("xem_7010_mux_spi/top.bit")
}
