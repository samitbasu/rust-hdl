use crate::atom::Atom;
use crate::block::Block;
use crate::probe::Probe;
use crate::synth::VCDValue;
use std::collections::HashMap;
use std::io::Write;

pub struct VCDProbe<W: Write> {
    vcd: vcd::Writer<W>,
    id_map: HashMap<usize, vcd::IdCode>,
    val_map: HashMap<vcd::IdCode, VCDValue>,
}

impl<W: Write> VCDProbe<W> {
    pub fn new(w: W) -> VCDProbe<W> {
        Self {
            vcd: vcd::Writer::new(w),
            id_map: HashMap::default(),
            val_map: HashMap::default(),
        }
    }

    pub fn timestamp(&mut self, ts: u64) -> std::io::Result<()> {
        self.vcd.timestamp(ts)
    }
}

struct VCDHeader<W: Write>(VCDProbe<W>);

impl<W: Write> Probe for VCDHeader<W> {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        self.0.vcd.add_module(name).unwrap();
    }

    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.0.vcd.add_module(name).unwrap();
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        let width = if signal.is_enum() {
            0
        } else {
            signal.bits() as u32
        };
        let id = self.0.vcd.add_wire(width, name).unwrap();
        self.0.id_map.insert(signal.id(), id);
    }

    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.0.vcd.upscope().unwrap();
    }

    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.0.vcd.upscope().unwrap();
    }
}

pub fn write_vcd_header<W: Write>(writer: W, uut: &dyn Block) -> VCDProbe<W> {
    let mut visitor = VCDHeader(VCDProbe::new(writer));
    visitor.0.vcd.timescale(1, vcd::TimescaleUnit::PS).unwrap();
    uut.accept("uut", &mut visitor);
    visitor.0.vcd.enddefinitions().unwrap();
    visitor.0
}

struct VCDChange<W: Write>(VCDProbe<W>);

impl<W: Write> Probe for VCDChange<W> {
    fn visit_atom(&mut self, _name: &str, signal: &dyn Atom) {
        if let Some(idc) = self.0.id_map.get(&signal.id()) {
            let val = signal.vcd();
            if let Some(old_val) = self.0.val_map.get(idc) {
                if val == *old_val {
                    return;
                }
            }
            self.0.val_map.insert(*idc, val.clone());
            match val {
                VCDValue::Single(s) => {
                    self.0.vcd.change_scalar(*idc, s).unwrap();
                }
                VCDValue::Vector(v) => {
                    self.0.vcd.change_vector(*idc, &v).unwrap();
                }
                VCDValue::String(t) => {
                    self.0.vcd.change_string(*idc, &t).unwrap();
                }
            }
        }
    }
}

pub fn write_vcd_change<W: Write>(vcd: VCDProbe<W>, uut: &dyn Block) -> VCDProbe<W> {
    let mut visitor = VCDChange(vcd);
    uut.accept("uut", &mut visitor);
    visitor.0
}

struct VCDDump<W: Write>(VCDProbe<W>);

impl<W: Write> Probe for VCDDump<W> {
    fn visit_atom(&mut self, _name: &str, signal: &dyn Atom) {
        if let Some(&idc) = self.0.id_map.get(&signal.id()) {
            let val = signal.vcd();
            self.0.val_map.insert(idc, val.clone());
            match val {
                VCDValue::Single(s) => {
                    self.0.vcd.change_scalar(idc, s).unwrap();
                }
                VCDValue::Vector(v) => {
                    self.0.vcd.change_vector(idc, &v).unwrap();
                }
                VCDValue::String(t) => {
                    self.0.vcd.change_string(idc, &t).unwrap();
                }
            }
        }
    }
}

pub fn write_vcd_dump<W: Write>(vcd: VCDProbe<W>, uut: &dyn Block) -> VCDProbe<W> {
    let mut visitor = VCDDump(vcd);
    visitor
        .0
        .vcd
        .begin(vcd::SimulationCommand::Dumpvars)
        .unwrap();
    uut.accept("uut", &mut visitor);
    visitor.0.vcd.end().unwrap();
    visitor.0
}
