use rust_hdl_private_core::prelude::*;

use crate::open_drain::{OpenDrainDriver, OpenDrainReceiver};

#[derive(LogicInterface, Default)]
#[join = "I2CBusReceiver"]
pub struct I2CBusDriver {
    pub sda: OpenDrainDriver,
    pub scl: OpenDrainDriver,
}

#[derive(LogicInterface, Default)]
#[join = "I2CBusDriver"]
pub struct I2CBusReceiver {
    pub sda: OpenDrainReceiver,
    pub scl: OpenDrainReceiver,
}
