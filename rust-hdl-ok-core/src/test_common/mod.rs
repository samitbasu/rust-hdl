#![allow(dead_code)]

use std::collections::BTreeMap;
use std::f64::consts::PI;

use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

pub mod blinky;

pub mod ddr;

pub mod download;
//pub mod fifo_tester;

pub mod fir;

pub mod mux_spi;

pub mod pipe;
pub mod soc;

pub mod spi;

pub mod tools;

pub mod wave;

pub mod wire;
