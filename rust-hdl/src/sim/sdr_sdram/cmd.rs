use crate::core::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum SDRAMCommand {
    LoadModeRegister,
    AutoRefresh,
    Precharge,
    BurstTerminate,
    Write,
    Read,
    Active,
    NOP,
}
