use crate::atom::Atom;
use crate::block::Block;
use crate::probe::Probe;
use crate::synth::VCDValue;
use crate::type_descriptor::TypeDescriptor;
use crate::type_descriptor::TypeKind;
use std::collections::HashMap;
use std::io::Write;

#[derive(Clone, Debug)]
enum VCDIDCode {
    Singleton(vcd::IdCode),
    Composite(Vec<Box<VCDIDCode>>),
}

pub struct VCDProbe<W: Write> {
    vcd: vcd::Writer<W>,
    id_map: HashMap<usize, VCDIDCode>,
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

fn register_signal<W: Write>(
    name: &str,
    descriptor: &TypeDescriptor,
    vcd: &mut vcd::Writer<W>,
) -> VCDIDCode {
    match &descriptor.kind {
        TypeKind::Bits(width) | TypeKind::Signed(width) => {
            VCDIDCode::Singleton(vcd.add_wire(*width as u32, name).unwrap())
        }
        TypeKind::Enum(_) => VCDIDCode::Singleton(vcd.add_wire(0, name).unwrap()),
        TypeKind::Composite(k) => {
            let mut ret = vec![];
            for field in k {
                let sub_name = format!("{}${}", name, field.fieldname);
                let code = register_signal(&sub_name, &field.kind, vcd);
                ret.push(Box::new(code));
            }
            VCDIDCode::Composite(ret)
        }
    }
}

impl<W: Write> Probe for VCDHeader<W> {
    fn visit_start_scope(&mut self, name: &str, _node: &dyn Block) {
        self.0.vcd.add_module(name).unwrap();
    }

    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.0.vcd.add_module(name).unwrap();
    }

    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        self.0.id_map.insert(
            signal.id(),
            register_signal(name, &signal.descriptor(), &mut self.0.vcd),
        );
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
            do_vcd_change(
                &mut self.0.val_map,
                &mut self.0.vcd,
                idc,
                &signal.vcd(),
                false,
            );
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
        if let Some(idc) = &self.0.id_map.get(&signal.id()) {
            do_vcd_change(
                &mut self.0.val_map,
                &mut self.0.vcd,
                idc,
                &signal.vcd(),
                true,
            );
        }
    }
}

fn do_vcd_change<W: Write>(
    val_map: &mut HashMap<vcd::IdCode, VCDValue>,
    vcd: &mut vcd::Writer<W>,
    idc: &VCDIDCode,
    val: &VCDValue,
    dump: bool,
) {
    match idc {
        VCDIDCode::Singleton(idc) => {
            if !dump {
                if let Some(old_val) = val_map.get(idc) {
                    if val.eq(old_val) {
                        return;
                    }
                }
            }
            let _ = val_map.insert(*idc, val.clone());
            match val {
                VCDValue::Single(s) => {
                    vcd.change_scalar(*idc, s.clone()).unwrap();
                }
                VCDValue::Vector(v) => {
                    if v.len() == 1 {
                        vcd.change_scalar(*idc, v[0]).unwrap();
                    } else {
                        vcd.change_vector(*idc, &v).unwrap();
                    }
                }
                VCDValue::String(t) => {
                    vcd.change_string(*idc, &t).unwrap();
                }
                VCDValue::Composite(_) => {
                    panic!("Composite data received for singleton type");
                }
            }
        }
        VCDIDCode::Composite(idcs) => match val {
            VCDValue::Composite(vals) => {
                assert_eq!(
                    idcs.len(),
                    vals.len(),
                    "Mismatch in values versus type information"
                );
                for n in 0..idcs.len() {
                    do_vcd_change(val_map, vcd, &idcs[n], &vals[n], dump);
                }
            }
            _ => {
                panic!("Scalar data received for composite type");
            }
        },
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
