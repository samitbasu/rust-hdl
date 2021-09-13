use crate::atom::Atom;
use crate::block::Block;
use crate::named_path::NamedPath;
use crate::probe::Probe;

#[derive(Default)]
struct CheckConnected {
    path: NamedPath,
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
            panic!(
                "Signal #{} {}::{} has no driver!",
                signal.id(),
                self.path.to_string(),
                name
            );
        }
        /* TODO - add logging
        else {
            println!("Signal {}::{} is driven", self.path.to_string(), name);
        }*/
    }

    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }

    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }
}

pub fn check_connected(uut: &dyn Block) {
    let mut visitor = CheckConnected::default();
    uut.accept("uut", &mut visitor);
}
