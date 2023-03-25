use crate::block::Block;
use crate::check_connected::check_connected;
use crate::check_logic_loops::check_logic_loops;
use crate::check_write_inputs::check_inputs_not_written;

use std::collections::HashMap;

/// A map of open connections, hashed on the signal ID
pub type OpenMap = HashMap<usize, PathedName>;

/// Struct to capture a signal in the design for human consumption
#[derive(Clone, Debug, PartialEq)]
pub struct PathedName {
    /// The path to the signal (i.e., the hierarchical namespace such as `uut:flasher:blah`)
    pub path: String,
    /// The name of the signal that is being referenced, such as `pulse_in`.
    pub name: String,
}

/// A list of [PathedName]
pub type PathedNameList = Vec<PathedName>;

/// The enum models the errors that can be returned from "checking"
/// a circuit using [check_all].
#[derive(Debug, Clone, PartialEq)]
pub enum CheckError {
    /// The check failed because of one or more open signals (described by the [OpenMap])
    OpenSignal(OpenMap),
    /// The circuit contains logical loops (i.e., `A <- B <- A`), and will not simulate
    LogicLoops(PathedNameList),
    /// The circuit attempts to write to the inputs, which is not allowed in RustHDL.
    WritesToInputs(PathedNameList),
}

/// This is a helper function used to check a [Block] for connection, loops, and
/// writes to the inputs.  
/// ```rust
/// use rust_hdl_core::prelude::*;
///
/// #[derive(LogicBlock, Default)]
/// struct Circuit {
///    pub in1: Signal<In, Bit>,
///    pub out1: Signal<Out, Bit>,
/// }
///
/// impl Logic for Circuit {
///    #[hdl_gen]
///    fn update(&mut self) {
///         self.out1.next = !self.in1.val();
///    }
/// }
///
/// let mut uut = Circuit::default();  uut.connect_all();
/// assert!(check_all(&uut).is_ok());
/// ```
pub fn check_all(uut: &dyn Block) -> Result<(), CheckError> {
    check_connected(uut)?;
    check_logic_loops(uut)?;
    check_inputs_not_written(uut)?;
    Ok(())
}
