use std::io::Write;

use rust_hdl::prelude::Bits;
use rust_hdl::prelude::ToBits;
use vcd::IdCode;

use crate::{bit_iter::BitIter, bit_slice::BitSlice};
use rust_hdl_x_macro::VCDWriteable;

trait VCDWriteable {
    fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()>;
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()>;
}

trait VCDWriter {
    fn push_scope(&mut self, name: &str);
    fn pop_scope(&mut self);
    fn allocate(&mut self, name: &str, width: u32) -> anyhow::Result<()>;
    fn serialize_scalar(&mut self, value: bool) -> anyhow::Result<()>;
    fn serialize_vector(&mut self, iter: impl Iterator<Item = bool>) -> anyhow::Result<()>;
    fn serialize_string(&mut self, value: &str) -> anyhow::Result<()>;
}

struct VCD<W: Write, S: VCDWriteable> {
    initialized: bool,
    ids: Vec<IdCode>,
    vcd: vcd::Writer<W>,
    id_ptr: usize,
    phantom: std::marker::PhantomData<S>,
}

impl<W: Write, S: VCDWriteable> VCD<W, S> {
    pub fn new(w: W) -> Self {
        Self {
            initialized: false,
            ids: vec![],
            vcd: vcd::Writer::new(w),
            id_ptr: 0,
            phantom: std::marker::PhantomData,
        }
    }
    pub fn write(&mut self, s: &S) -> anyhow::Result<()> {
        if !self.initialized {
            s.register("top", self)?;
            self.initialized = true;
        }
        self.id_ptr = 0;
        s.serialize(self)?;
        Ok(())
    }
}

impl<W: Write, S: VCDWriteable> VCDWriter for VCD<W, S> {
    fn push_scope(&mut self, name: &str) {
        self.vcd.add_module(name).unwrap();
    }
    fn pop_scope(&mut self) {
        self.vcd.upscope().unwrap();
    }
    fn allocate(&mut self, name: &str, width: u32) -> anyhow::Result<()> {
        self.ids.push(self.vcd.add_wire(width, &name)?);
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

#[derive(VCDWriteable)]
pub struct TwoBits {
    bit_1: bool,
    bit_2: bool,
    part_3: u8,
    nibble_4: Bits<4>,
}

#[derive(VCDWriteable)]
pub struct NestedBits {
    nest_1: bool,
    nest_2: u8,
    nest_3: TwoBits,
}

#[derive(VCDWriteable)]
pub enum MyState {
    Idle,
    Running,
    Faulted,
    Sleeping,
}

#[derive(VCDWriteable)]
pub struct Mixed {
    state: MyState,
    bits: TwoBits,
}

impl VCDWriteable for bool {
    fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.allocate(name, 1)
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.serialize_scalar(*self)
    }
}

impl VCDWriteable for u8 {
    fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.allocate(name, 8)
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.serialize_vector(BitIter::new(*self))
    }
}

impl<const N: usize> VCDWriteable for Bits<N> {
    fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.allocate(name, N as u32)
    }
    fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
        w.serialize_vector(BitIter::new(*self))
    }
}

#[test]
fn test_vcd_write() {
    let mut vcd = VCD::new(std::io::stdout());
    let bits = TwoBits {
        bit_1: true,
        bit_2: false,
        part_3: 0b1010_1010,
        nibble_4: 0b1010_u8.to_bits(),
    };
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
    let mut vcd = VCD::new(std::io::stdout());
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
    let mut vcd = VCD::new(std::io::stdout());
    let bits = Mixed {
        state: MyState::Running,
        bits: TwoBits {
            bit_1: true,
            bit_2: false,
            part_3: 0b1010_1010,
            nibble_4: 0b1010_u8.to_bits(),
        },
    };
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
