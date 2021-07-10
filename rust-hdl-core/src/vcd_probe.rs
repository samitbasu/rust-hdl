use std::io::Write;
use crate::probe::Probe;
use crate::block::Block;
use crate::atom::Atom;
use crate::named_path::NamedPath;
use std::collections::HashMap;
use num_bigint::BigUint;

struct VCDProbe<W: Write> {
    path: NamedPath,
    vcd: vcd::Writer<W>,
    id_map: HashMap<String, vcd::IdCode>,
    val_map: HashMap<vcd::IdCode, BigUint>,
}

impl<W: Write> VCDProbe<W> {
    pub fn new(w: W) -> VCDProbe<W> {
        Self {
            path: NamedPath::default(),
            vcd: vcd::Writer::new(w),
            id_map: HashMap::default(),
        }
    }

    pub fn timestamp(&mut self, ts: u64) -> std::io::Result<()> {
        self.vcd.timestamp(ts)
    }
}


impl<W: Write> Probe for VCDProbe<W> {
    fn visit_start_scope(&mut self, name: &str, node: &dyn Block) {
        self.path.push(name);
    }

    fn visit_start_namespace(&mut self, name: &str, node: &dyn Block) {
        self.path.push(name);
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        let entry = self.id_map.get()
    }

    fn visit_end_namespace(&mut self, name: &str, node: &dyn Block) {
        self.path.pop();
    }

    fn visit_end_scope(&mut self, name: &str, node: &dyn Block) {
        self.path.pop();
    }
}
