use crate::block::Block;
use crate::atom::Atom;
use crate::scoped_visitor::ScopedVisitor;
use std::path::PathBuf;

struct CheckConnected {
    path: PathBuf,
}

impl CheckConnected {
    fn new() -> Self {
        Self {
            path: PathBuf::new(),
        }
    }
}

impl ScopedVisitor for CheckConnected {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        self.path.push(name);
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        if !signal.connected() {
            panic!(
                "Signal {}::{} has no driver!",
                self.path.to_str().unwrap(),
                name
            )
        }
        println!("Signal {}::{} is driven", self.path.to_str().unwrap(), name);
    }

    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }
}

pub fn check_connected(uut: &dyn Block) {
    let mut visitor = CheckConnected::new();
    uut.accept_scoped("uut", &mut visitor);
}
