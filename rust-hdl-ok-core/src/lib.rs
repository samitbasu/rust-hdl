#![allow(non_camel_case_types)]

use std::time::Duration;

use ok_hi::OpalKellyHostInterface;
use ok_host::OpalKellyHost;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::pulser::Pulser;

pub mod bsp;
pub mod ddr_fifo;
pub mod ddr_fifo7;
pub mod mcb_if;
pub mod mig;
pub mod mig7;
pub mod ok_download;
pub mod ok_download_ddr;
pub mod ok_download_ddr7;
pub mod ok_hi;
pub mod ok_host;
pub mod ok_pipe;
pub mod ok_sys_clock7;
pub mod ok_trigger;
pub mod ok_wire;
pub mod prelude;
pub mod spi;

pub const MHZ48: u64 = 48_000_000;
