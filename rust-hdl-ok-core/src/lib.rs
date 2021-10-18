#![allow(non_camel_case_types)]

use ok_hi::OpalKellyHostInterface;
use ok_host::OpalKellyHost;

pub mod bsp;
pub mod ok_download;
pub mod ok_hi;
pub mod ok_host;
pub mod ok_pipe;
pub mod ok_trigger;
pub mod ok_wire;
pub mod prelude;
pub mod spi;

pub const MHZ48: u64 = 48_000_000;
