use crate::core::atom::Atom;
use crate::core::block::Block;
use crate::core::named_path::NamedPath;
use crate::core::probe::Probe;

#[derive(Default)]
pub struct TimingChecker {
    path: NamedPath,
    namespace: NamedPath,
}

impl Probe for TimingChecker {
    fn visit_start_scope(&mut self, name: &str, node: &dyn Block) {
        let top_level = self.path.to_string();
        self.path.push(name);
        self.namespace.reset();
    }
    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.namespace.push(name);
    }
    fn visit_atom(&mut self, _name: &str, _signal: &dyn Atom) {
        let module_path = self.path.to_string();
        let module_name = self.path.last();
        let namespace = self.namespace.flat("$");
        let name = if namespace.is_empty() {
            name.to_owned()
        } else {
            format!("{}${}", namespace, name)
        };
        // Atoms could be inputs, outputs or local
    }
    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
    }
    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
    }
}