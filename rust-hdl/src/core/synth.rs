use std::fmt::Debug;

use crate::core::ast::VerilogLiteral;
use crate::core::bits::{Bit, Bits};
use crate::core::clock::Clock;
use crate::core::signed::Signed;

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
    const SIGNED: bool = false;
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

impl Synth for Clock {
    const BITS: usize = 1;

    fn vcd(self) -> VCDValue {
        self.0.into()
    }

    fn verilog(self) -> VerilogLiteral {
        self.0.into()
    }
}

impl<const N: usize> Synth for Signed<N> {
    const BITS: usize = N;
    const TYPE_NAME: &'static str = "Signed";
    const SIGNED: bool = true;
    fn vcd(self) -> VCDValue {
        self.inner().vcd()
    }
    fn verilog(self) -> VerilogLiteral {
        self.inner().into()
    }
}