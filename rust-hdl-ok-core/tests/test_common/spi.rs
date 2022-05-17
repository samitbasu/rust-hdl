use crate::test_common::tools::{ok_do_spi_txn, ok_reg_read, ok_reg_write, ok_test_prelude};
use rust_hdl_ok_core::core::prelude::*;
use rust_hdl::core::prelude::*;
use rust_hdl::sim::prelude::*;
use rust_hdl::widgets::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use std::thread::sleep;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct OpalKellySPITest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub adc: AD7193Simulator,
    pub spi: OKSPIMaster,
}

impl OpalKellySPITest {
    pub fn new<B: OpalKellyBSP>() -> Self {
        let adc_config = AD7193Config::hw();
        Self {
            hi: B::hi(),
            ok_host: B::ok_host(),
            adc: AD7193Simulator::new(adc_config),
            spi: OKSPIMaster::new(Default::default(), adc_config.spi),
        }
    }
}

impl Logic for OpalKellySPITest {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        self.spi.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.spi.ok2.val();
        self.spi.clock.next = self.ok_host.ti_clk.val();
        self.adc.clock.next = self.ok_host.ti_clk.val();
        SPIWiresMaster::join(&mut self.spi.wires, &mut self.adc.wires);
    }
}

pub fn test_opalkelly_spi_reg_read_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
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

pub fn test_opalkelly_spi_reg_write_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
    ok_do_spi_txn(&hnd, 64, 0xFFFFFFFFFFFFFFFF_u64, false).unwrap();
    let expected = [0x40, 0x80060, 0x117, 0, 0xa2, 0, 0x800000, 0x5544d0];
    for reg in 0..8 {
        let x = ok_reg_read(&hnd, reg)?;
        println!("Read of reg {} is {:x}", reg, x);
        assert_eq!(x, expected[reg as usize]);
    }
    ok_reg_write(&hnd, 5, 0x2d)?;
    let x = ok_reg_read(&hnd, 5)?;
    assert_eq!(x, 0x2d);
    hnd.close();
    Ok(())
}

pub fn test_opalkelly_spi_single_conversion_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
    ok_do_spi_txn(&hnd, 64, 0xFFFFFFFFFFFFFFFF_u64, false).unwrap();
    sleep(Duration::from_millis(100));
    for i in 0..4 {
        ok_do_spi_txn(&hnd, 32, 0x8382006, true).unwrap();
        sleep(Duration::from_millis(100));
        let reply = ok_reg_read(&hnd, 3).unwrap();
        let reply = reply & 0xFFFFFF_u64;
        assert_eq!(reply, i * 0x100);
    }
    hnd.close();
    Ok(())
}
