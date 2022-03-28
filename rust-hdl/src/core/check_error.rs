use crate::core::block::Block;
use crate::core::check_logic_loops::check_logic_loops;
use crate::core::prelude::check_connected;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct OpenConnection {
    pub path: String,
    pub name: String,
}

pub type OpenMap = HashMap<usize, OpenConnection>;

#[derive(Clone, Debug, PartialEq)]
pub struct LogicLoop {
    pub path: String,
    pub name: String,
}

pub type LoopMap = Vec<LogicLoop>;

#[derive(Debug, Clone, PartialEq)]
pub enum CheckError {
    OpenSignal(OpenMap),
    LogicLoops(LoopMap),
}

pub fn check_all(uut: &dyn Block) -> Result<(), CheckError> {
    check_connected(uut)?;
    check_logic_loops(uut)?;
    Ok(())
}
