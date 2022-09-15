use crate::core::atom::Atom;
use crate::core::block::Block;
use crate::core::check_error::{CheckError, PathedName, OpenMap};
use crate::core::named_path::NamedPath;
use crate::core::probe::Probe;

#[derive(Default)]
struct CheckConnected {
    path: NamedPath,
    namespace: NamedPath,
    failures: OpenMap,
}

impl Probe for CheckConnected {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        self.path.push(name);
        self.namespace.reset();
    }

    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.namespace.push(name);
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        let is_top_scope = self.path.to_string().eq("uut");
        if !(signal.connected() | is_top_scope) {
            self.failures.insert(
                signal.id(),
                PathedName {
                    path: self.path.to_string(),
                    name: name.to_string(),
                },
            );
        }
    }

    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.namespace.pop();
    }

    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }
}


/// Check to see if a circuit is properly connected (no undriven inputs, or
/// multiply-driven outputs).  You can call this directly on a circuit of yours
/// if you want to check that it is correctly connected internally.  
/// ```rust
/// use rust_hdl::prelude::*;
/// 
/// #[derive(LogicBlock, Default)]
/// struct Broken {
///     pub I: Signal<In, Bit>,
///     pub O: Signal<Out, Bit>,
/// }
/// 
/// impl Logic for Broken {
///    #[hdl_gen]
///    fn update(&mut self) {
///       // Purposely left blank... circuit is broken!
///    }
/// }
/// 
/// let mut uut = TopWrap::new(Broken::default()); // <- we use TopWrap since we want Broken to not be the top
/// uut.connect_all();
/// assert!(check_connected(&uut).is_err())
/// ```
pub fn check_connected(uut: &dyn Block) -> Result<(), CheckError> {
    let mut visitor = CheckConnected::default();
    uut.accept("uut", &mut visitor);
    if visitor.failures.is_empty() {
        Ok(())
    } else {
        Err(CheckError::OpenSignal(visitor.failures))
    }
}
