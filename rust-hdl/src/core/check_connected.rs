use crate::core::atom::Atom;
use crate::core::block::Block;
use crate::core::check_error::{CheckError, OpenConnection, OpenMap};
use crate::core::named_path::NamedPath;
use crate::core::probe::Probe;

#[derive(Default)]
struct CheckConnected {
    path: NamedPath,
    failures: OpenMap,
}

impl Probe for CheckConnected {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        self.path.push(name);
    }

    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.path.push(name);
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        if !signal.connected() {
            self.failures.insert(
                signal.id(),
                OpenConnection {
                    path: self.path.to_string(),
                    name: name.to_string(),
                },
            );
        }
    }

    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }

    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }
}

pub fn check_connected(uut: &dyn Block) -> Result<(), CheckError> {
    let mut visitor = CheckConnected::default();
    uut.accept("uut", &mut visitor);
    if visitor.failures.is_empty() {
        Ok(())
    } else {
        Err(CheckError::OpenSignal(visitor.failures))
    }
}
