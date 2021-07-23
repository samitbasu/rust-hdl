use std::fmt::Debug;

use crate::ast::VerilogLiteral;
use crate::bits::{Bit, Bits};
use crate::clock::{Clock, Domain};

#[derive(Clone, PartialEq, Debug)]
pub enum VCDValue {
    Single(vcd::Value),
    Vector(Vec<vcd::Value>),
    String(String),
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

pub trait Synth: Default + Copy + PartialEq + Debug {
    const BITS: usize;
    const ENUM_TYPE: bool = false;
    const TYPE_NAME: &'static str = "Bits";
    fn name(_ndx: usize) -> &'static str {
        ""
    }
    fn vcd(self) -> VCDValue;
    fn verilog(self) -> VerilogLiteral;
}

impl<const N: usize> Synth for Bits<N> {
    const BITS: usize = N;

    fn vcd(self) -> VCDValue {
        self.into()
    }

    fn verilog(self) -> VerilogLiteral {
        self.into()
    }
}

impl Synth for Bit {
    const BITS: usize = 1;

    fn vcd(self) -> VCDValue {
        if self {
            VCDValue::Single(vcd::Value::V1)
        } else {
            VCDValue::Single(vcd::Value::V0)
        }
    }

    fn verilog(self) -> VerilogLiteral {
        self.into()
    }
}

impl<D: Domain> Synth for Clock<D> {
    const BITS: usize = 1;

    fn vcd(self) -> VCDValue {
        self.0.into()
    }

    fn verilog(self) -> VerilogLiteral {
        self.0.into()
    }
}
