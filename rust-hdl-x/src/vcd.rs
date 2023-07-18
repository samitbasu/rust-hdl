use std::io::Write;

use rust_hdl::prelude::Bits;
use rust_hdl::prelude::ToBits;
use vcd::IdCode;

use crate::{bit_iter::BitIter, bit_slice::BitSlice};

trait VCDWriteable {
    fn register(&self, w: &mut impl VCDWriter) -> anyhow::Result<()>;
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()>;
}

trait VCDWriter {
    fn push_scope(&mut self, name: &str);
    fn pop_scope(&mut self);
    fn allocate(&mut self, width: u32) -> anyhow::Result<()>;
    fn serialize_scalar(&mut self, value: bool) -> anyhow::Result<()>;
    fn serialize_vector(&mut self, iter: impl Iterator<Item = bool>) -> anyhow::Result<()>;
    fn serialize_string(&mut self, value: &str) -> anyhow::Result<()>;
}

struct VCD<W: Write, S: VCDWriteable> {
    initialized: bool,
    scope: Vec<String>,
    ids: Vec<IdCode>,
    vcd: vcd::Writer<W>,
    id_ptr: usize,
    phantom: std::marker::PhantomData<S>,
}

impl<W: Write, S: VCDWriteable> VCD<W, S> {
    pub fn new(top: &str, w: W) -> Self {
        Self {
            initialized: false,
            scope: vec![top.to_string()],
            ids: vec![],
            vcd: vcd::Writer::new(w),
            id_ptr: 0,
            phantom: std::marker::PhantomData,
        }
    }
    pub fn setup(&mut self) -> anyhow::Result<()> {
        self.vcd.add_module(&self.scope[0])?;
        Ok(())
    }
    pub fn write(&mut self, s: &S) -> anyhow::Result<()> {
        if !self.initialized {
            s.register(self)?;
            self.initialized = true;
        }
        self.id_ptr = 0;
        s.serialize(self)?;
        Ok(())
    }
}

impl<W: Write, S: VCDWriteable> VCDWriter for VCD<W, S> {
    fn push_scope(&mut self, name: &str) {
        self.scope.push(name.to_string());
    }
    fn pop_scope(&mut self) {
        self.scope.pop();
    }
    fn allocate(&mut self, width: u32) -> anyhow::Result<()> {
        let my_path = self.scope.join(".");
        self.ids.push(self.vcd.add_wire(width, &my_path)?);
        Ok(())
    }
    fn serialize_scalar(&mut self, value: bool) -> anyhow::Result<()> {
        let id = self.ids[self.id_ptr];
        self.vcd.change_scalar(id, value)?;
        self.id_ptr += 1;
        Ok(())
    }
    fn serialize_vector(&mut self, iter: impl Iterator<Item = bool>) -> anyhow::Result<()> {
        let id = self.ids[self.id_ptr];
        self.vcd.change_vector(id, iter.map(|x| x.into()))?;
        self.id_ptr += 1;
        Ok(())
    }
    fn serialize_string(&mut self, value: &str) -> anyhow::Result<()> {
        let id = self.ids[self.id_ptr];
        self.vcd.change_string(id, value)?;
        self.id_ptr += 1;
        Ok(())
    }
}

pub struct TwoBits {
    bit_1: bool,
    bit_2: bool,
    part_3: u8,
    nibble_4: Bits<4>,
}

pub struct NestedBits {
    nest_1: bool,
    nest_2: u8,
    nest_3: TwoBits,
}

pub enum MyState {
    Idle,
    Running,
    Faulted,
    Sleeping,
}

pub struct Mixed {
    state: MyState,
    bits: TwoBits,
}

impl VCDWriteable for Mixed {
    fn register(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.push_scope("state");
        self.state.register(w)?;
        w.pop_scope();
        w.push_scope("bits");
        self.bits.register(w)?;
        w.pop_scope();
        Ok(())
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        self.state.serialize(w)?;
        self.bits.serialize(w)?;
        Ok(())
    }
}

impl VCDWriteable for MyState {
    fn register(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.allocate(0)
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        match self {
            MyState::Idle => w.serialize_string("Idle"),
            MyState::Running => w.serialize_string("Running"),
            MyState::Faulted => w.serialize_string("Faulted"),
            MyState::Sleeping => w.serialize_string("Sleeping"),
        }
    }
}

impl VCDWriteable for NestedBits {
    fn register(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.push_scope("nest_1");
        self.nest_1.register(w)?;
        w.pop_scope();
        w.push_scope("nest_2");
        self.nest_2.register(w)?;
        w.pop_scope();
        w.push_scope("nest_3");
        self.nest_3.register(w)?;
        w.pop_scope();
        Ok(())
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        self.nest_1.serialize(w)?;
        self.nest_2.serialize(w)?;
        self.nest_3.serialize(w)?;
        Ok(())
    }
}

impl VCDWriteable for TwoBits {
    fn register(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.push_scope("bit_1");
        self.bit_1.register(w)?;
        w.pop_scope();
        w.push_scope("bit_2");
        self.bit_2.register(w)?;
        w.pop_scope();
        w.push_scope("part_3");
        self.part_3.register(w)?;
        w.pop_scope();
        w.push_scope("nibble_4");
        self.nibble_4.register(w)?;
        w.pop_scope();
        Ok(())
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        self.bit_1.serialize(w)?;
        self.bit_2.serialize(w)?;
        self.part_3.serialize(w)?;
        self.nibble_4.serialize(w)?;
        Ok(())
    }
}

impl VCDWriteable for bool {
    fn register(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.allocate(1)
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.serialize_scalar(*self)
    }
}

impl VCDWriteable for u8 {
    fn register(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.allocate(8)
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.serialize_vector(BitIter::new(*self))
    }
}

impl<const N: usize> VCDWriteable for Bits<N> {
    fn register(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.allocate(N as u32)
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.serialize_vector(BitIter::new(*self))
    }
}

#[test]
fn test_vcd_write() {
    let mut vcd = VCD::new("top", std::io::stdout());
    let bits = TwoBits {
        bit_1: true,
        bit_2: false,
        part_3: 0b1010_1010,
        nibble_4: 0b1010_u8.to_bits(),
    };
    vcd.setup().unwrap();
    vcd.write(&bits).unwrap();
    let bits = TwoBits {
        bit_1: false,
        bit_2: true,
        part_3: 0b0101_0101,
        nibble_4: 0b0101_u8.to_bits(),
    };
    vcd.write(&bits).unwrap();
}

#[test]
fn test_vcd_nested_write() {
    let mut vcd = VCD::new("top", std::io::stdout());
    let bits = NestedBits {
        nest_1: true,
        nest_2: 0b1010_1010,
        nest_3: TwoBits {
            bit_1: true,
            bit_2: false,
            part_3: 0b1010_1010,
            nibble_4: 0b1010_u8.to_bits(),
        },
    };
    vcd.setup().unwrap();
    vcd.write(&bits).unwrap();
    let bits = NestedBits {
        nest_1: false,
        nest_2: 0b0101_0101,
        nest_3: TwoBits {
            bit_1: false,
            bit_2: true,
            part_3: 0b0101_0101,
            nibble_4: 0b0101_u8.to_bits(),
        },
    };
    vcd.write(&bits).unwrap();
}

#[test]
fn test_vcd_mixed_write() {
    let mut vcd = VCD::new("top", std::io::stdout());
    let bits = Mixed {
        state: MyState::Running,
        bits: TwoBits {
            bit_1: true,
            bit_2: false,
            part_3: 0b1010_1010,
            nibble_4: 0b1010_u8.to_bits(),
        },
    };
    vcd.setup().unwrap();
    vcd.write(&bits).unwrap();
    let bits = Mixed {
        state: MyState::Faulted,
        bits: TwoBits {
            bit_1: false,
            bit_2: true,
            part_3: 0b0101_0101,
            nibble_4: 0b0101_u8.to_bits(),
        },
    };
    vcd.write(&bits).unwrap();
}
