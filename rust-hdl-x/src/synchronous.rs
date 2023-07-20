use num_traits::Num;
use std::{
    io::Write,
    ops::{BitAnd, Shr},
    path::PathBuf,
};

use rust_hdl::prelude::Bits;
use serde::{
    ser::{SerializeSeq, SerializeTuple},
    Serialize, Serializer,
};

use crate::{bit_iter::BitIter, vcd::VCDWriteable};

#[derive(Clone, PartialEq, Debug)]
pub enum VCDValue {
    Single(vcd::Value),
    Vector(Vec<vcd::Value>),
    String(String),
    Composite(Vec<VCDValue>),
}

impl From<bool> for VCDValue {
    fn from(x: bool) -> Self {
        if x {
            VCDValue::Single(vcd::Value::V1)
        } else {
            VCDValue::Single(vcd::Value::V0)
        }
    }
}

pub trait Synthesizable: Copy + PartialEq {
    const BITS: usize;
    fn vcd(self) -> VCDValue;
    fn bits(self) -> usize {
        Self::BITS
    }
}

impl<const N: usize> Synthesizable for Bits<N> {
    const BITS: usize = N;

    fn vcd(self) -> VCDValue {
        self.into()
    }
}

impl<const N: usize> From<Bits<N>> for VCDValue {
    fn from(val: Bits<N>) -> Self {
        if N == 1 {
            if val.get_bit(0) {
                VCDValue::Single(vcd::Value::V1)
            } else {
                VCDValue::Single(vcd::Value::V0)
            }
        } else {
            let mut x = vec![];
            for i in 0..N {
                if val.get_bit(N - 1 - i) {
                    x.push(vcd::Value::V1)
                } else {
                    x.push(vcd::Value::V0)
                }
            }
            VCDValue::Vector(x)
        }
    }
}

impl Synthesizable for bool {
    const BITS: usize = 1;

    fn vcd(self) -> VCDValue {
        if self {
            VCDValue::Single(vcd::Value::V1)
        } else {
            VCDValue::Single(vcd::Value::V0)
        }
    }
}

// Todo - need RAII pattern
pub trait Tracer: Sized {
    fn enter(&self, name: &str);
    fn write(&self, tag: &str, value: impl VCDWriteable);
    fn exit(&self);
    fn module(&self, name: &str) -> Scope<Self> {
        Scope { tracer: self }
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct NoTrace {}

impl Tracer for NoTrace {
    fn enter(&self, _name: &str) {}
    fn write(&self, _tag: &str, _value: impl VCDWriteable) {}
    fn exit(&self) {}
}

impl<T: Tracer> Tracer for &T {
    fn enter(&self, name: &str) {
        (*self).enter(name)
    }
    fn write(&self, tag: &str, value: impl VCDWriteable) {
        (*self).write(tag, value)
    }
    fn exit(&self) {
        (*self).exit()
    }
}

pub struct Scope<'a, T: Tracer> {
    tracer: &'a T,
}

impl<'a, T: Tracer> Scope<'a, T> {
    fn new(tracer: &'a T, name: &str) -> Self {
        tracer.enter(name);
        Self { tracer }
    }
}

impl<'a, T: Tracer> Drop for Scope<'a, T> {
    fn drop(&mut self) {
        self.tracer.exit();
    }
}

pub trait Synchronous {
    type Input: Copy;
    type Output: Copy;
    type State: Copy + Default;
    fn update(
        &self,
        tracer: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State);
    fn default_output(&self) -> Self::Output;
}

// To write a value to a VCD file, we need an ID for it and enough type information
// to know how to write it.  In general, writing to a VCD requires two steps:
//  1. Register the type with the VCD file, and get an ID for it.
//  2. Use the ID to write the value to the VCD file.

// To write a complex type, we
//   1. Call vcd.add_module
//   2. Call vcd.add_wire for each field

// A new idea is to use serde

pub struct TwoBits {
    bit_1: bool,
    bit_2: bool,
}

impl BitSerialize for TwoBits {
    fn register<S: BitSerializer>(&self, serializer: &mut S) -> anyhow::Result<()> {
        serializer.serialize_start_scope("bit_1");
        serializer.register(1)?;
        serializer.serialize_end_scope();
        serializer.serialize_start_scope("bit_2");
        serializer.register(1)?;
        serializer.serialize_end_scope();
        Ok(())
    }

    fn write<S: BitSerializer>(&self, serializer: &mut S) -> anyhow::Result<()> {
        serializer.serialize_scalar(self.bit_1)?;
        serializer.serialize_scalar(self.bit_2)?;
        Ok(())
    }
}

pub trait BitSerialize {
    fn register<S: BitSerializer>(&self, serializer: &mut S) -> anyhow::Result<()>;
    fn write<S: BitSerializer>(&self, serializer: &mut S) -> anyhow::Result<()>;
}

pub trait BitSerializer {
    fn register(&mut self, width: u32) -> anyhow::Result<()>;
    fn serialize_scalar(&mut self, value: bool) -> anyhow::Result<()>;
    //    fn serialize_i8(&mut self, value: i8) -> anyhow::Result<()>;
    fn serialize_start_scope(&mut self, name: &str);
    fn serialize<T: BitSerialize>(&mut self, value: &T) -> anyhow::Result<()>;
    fn serialize_end_scope(&mut self);
}

pub struct VCDSerializer<S: BitSerialize, W: Write> {
    vcd: vcd::Writer<W>,
    ids: Vec<vcd::IdCode>,
    phantom: std::marker::PhantomData<S>,
    scope: Vec<String>,
    initialized: bool,
    index: usize,
}

impl<S: BitSerialize, W: Write> VCDSerializer<S, W> {
    pub fn new(top: &str, w: W) -> Self {
        Self {
            vcd: vcd::Writer::new(w),
            ids: vec![],
            phantom: std::marker::PhantomData,
            scope: vec![],
            initialized: false,
            index: 0,
        }
    }
}

impl<S: BitSerialize, W: Write> BitSerializer for VCDSerializer<S, W> {
    fn register(&mut self, width: u32) -> anyhow::Result<()> {
        let my_path = self.scope.join(".");
        self.ids.push(self.vcd.add_wire(width, &my_path)?);
        Ok(())
    }

    fn serialize_start_scope(&mut self, name: &str) {
        if !self.initialized {
            self.scope.push(name.to_string());
        }
    }

    fn serialize_end_scope(&mut self) {
        if !self.initialized {
            self.scope.pop();
        }
    }

    fn serialize<T: BitSerialize>(&mut self, value: &T) -> anyhow::Result<()> {
        if !self.initialized {
            value.register(self)?;
            return Ok(());
        }
        value.write(self)
    }

    fn serialize_scalar(&mut self, value: bool) -> anyhow::Result<()> {
        if !self.initialized {
            self.register(1)?;
            return Ok(());
        }
        self.vcd.change_scalar(
            self.ids[self.index],
            if value {
                vcd::Value::V1
            } else {
                vcd::Value::V0
            },
        )?;
        self.index += 1;
        Ok(())
    }
}
