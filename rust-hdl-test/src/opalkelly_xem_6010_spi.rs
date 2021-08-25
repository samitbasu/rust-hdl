use crate::ad7193_sim::{AD7193Simulator, AD7193_SPI_CONFIG};
use crate::ok_tools::{ok_do_spi_txn, ok_reg_read, ok_test_prelude};
use rust_hdl_core::prelude::*;
use rust_hdl_ok::prelude::*;
use rust_hdl_ok::spi::OKSPIMaster;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_widgets::prelude::*;
use rust_hdl_widgets::spi_master::{SPIConfig, SPIWires};

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010SPITest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub adc: AD7193Simulator,
    pub spi: OKSPIMaster,
}

impl Default for OpalKellyXEM6010SPITest {
    fn default() -> Self {
        let spi_config = SPIConfig {
            clock_speed: 48_000_000,
            cs_off: true,
            mosi_off: true,
            speed_hz: 400_000,
            cpha: true,
            cpol: true,
        };
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: Default::default(),
            adc: AD7193Simulator::new(spi_config),
            spi: OKSPIMaster::new(Default::default(), spi_config),
        }
    }
}

impl Logic for OpalKellyXEM6010SPITest {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        self.spi.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.spi.ok2.val();
        self.spi.clock.next = self.ok_host.ti_clk.val();
        self.adc.clock.next = self.ok_host.ti_clk.val();
        self.adc.mosi.next = self.spi.wires.mosi.val();
        self.adc.mclk.next = self.spi.wires.mclk.val();
        self.adc.msel.next = self.spi.wires.msel.val();
        self.spi.wires.miso.next = self.adc.miso.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_spi() {
    let mut uut = OpalKellyXEM6010SPITest::default();
    uut.hi.link_connect();
    uut.spi.wires.link_connect(); // TODO - this should not be needed
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_spi");
}

#[test]
fn test_opalkelly_xem_6010_spi_reg_read_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_spi/top.bit")?;
    ok_do_spi_txn(&hnd, 64, 0xFFFFFFFFFFFFFFFF_u64, false).unwrap();
    let expected = [0x40, 0x80060, 0x117, 0, 0xa2, 0, 0x800000, 0x5544d0];
    for reg in 0..8 {
        let x = ok_reg_read(&hnd, reg).unwrap();
        println!("Read of reg {} is {:x}", reg, x);
        assert_eq!(x, expected[reg as usize]);
    }
    hnd.close();
    Ok(())
}
