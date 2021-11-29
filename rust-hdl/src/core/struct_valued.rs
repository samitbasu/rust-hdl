use crate::core::ast::VerilogLiteral;
use crate::core::bits::{bit_cast, clog2, Bit, Bits};
use crate::core::synth::{Synth, VCDValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CmdType {
    Noop,
    Read,
    Write,
}

pub fn raw_cast<S: Synth, T: Synth + From<S>>(x: S) -> T {
    x.into()
}

// Auto generated (?)
impl Default for CmdType {
    fn default() -> Self {
        CmdType::Noop
    }
}

// Auto generated
impl Synth for CmdType {
    const BITS: usize = clog2(3);
    fn vcd(self) -> VCDValue {
        match self {
            CmdType::Noop => VCDValue::String("Noop".into()),
            CmdType::Read => VCDValue::String("Read".into()),
            CmdType::Write => VCDValue::String("Write".into()),
        }
    }
    fn verilog(self) -> VerilogLiteral {
        match self {
            CmdType::Noop => 0_u32.into(),
            CmdType::Read => 1_u32.into(),
            CmdType::Write => 2_u32.into(),
        }
    }
}

// Auto generated
impl From<CmdType> for Bits<2> {
    fn from(x: CmdType) -> Self {
        match x {
            CmdType::Noop => 0_usize.into(),
            CmdType::Read => 1_usize.into(),
            CmdType::Write => 2_usize.into(),
        }
    }
}

// Auto generated
impl From<Bits<2>> for CmdType {
    fn from(x: Bits<2>) -> Self {
        let xval: usize = x.into();
        match xval {
            0 => CmdType::Noop,
            1 => CmdType::Read,
            2 => CmdType::Write,
            _ => panic!("Illegal conversion"),
        }
    }
}

#[test]
fn test_struct_value() {
    let states = [CmdType::Noop, CmdType::Read, CmdType::Write];
    for state in states {
        let b: Bits<2> = state.into();
        let c: CmdType = b.into();
        assert_eq!(c, state);
    }

    for state in states {
        let b = raw_cast::<CmdType, Bits<2>>(state);
        let c = raw_cast::<Bits<2>, CmdType>(b);
        assert_eq!(c, state);
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
struct MIGCmd {
    pub cmd: CmdType,
    pub active: Bit,
    pub len: Bits<6>,
}

impl Synth for MIGCmd {
    const BITS: usize = CmdType::BITS + Bit::BITS + Bits::<6>::BITS;

    fn vcd(self) -> VCDValue {
        let t: Bits<{ MIGCmd::BITS }> = self.into();
        t.into()
    }

    fn verilog(self) -> VerilogLiteral {
        let t: Bits<{ MIGCmd::BITS }> = self.into();
        t.into()
    }
}

// Auto generated
impl From<MIGCmd> for Bits<{ MIGCmd::BITS }> {
    fn from(x: MIGCmd) -> Self {
        let x2 = bit_cast::<{ MIGCmd::BITS }, { Bits::<6>::BITS }>(x.len.into());
        let x1 =
            bit_cast::<{ MIGCmd::BITS }, { bool::BITS }>(x.active.into()) | x2 << { bool::BITS };
        let x0 =
            bit_cast::<{ MIGCmd::BITS }, { CmdType::BITS }>(x.cmd.into()) | x1 << { CmdType::BITS };
        x0
    }
}

// Auto generated
impl From<Bits<{ MIGCmd::BITS }>> for MIGCmd {
    fn from(x: Bits<{ MIGCmd::BITS }>) -> Self {
        let cmd: CmdType = x.get_bits::<{ CmdType::BITS }>(0).into();
        let x = x >> { CmdType::BITS };
        let active: bool = x.get_bits::<{ bool::BITS }>(0).into();
        let x = x >> { bool::BITS };
        let len: Bits<6> = x.get_bits::<{ Bits::<6>::BITS }>(0).into();
        MIGCmd { cmd, active, len }
    }
}

#[test]
fn test_composite() {
    assert_eq!(MIGCmd::BITS, 9);
    let x = MIGCmd {
        cmd: CmdType::Read,
        active: true,
        len: 35_usize.into(),
    };

    let y: Bits<9> = x.into();
    assert_eq!(y.get_bits::<{ CmdType::BITS }>(0), 1u32);
    assert_eq!(y.get_bits::<{ bool::BITS }>(2), true);
    assert_eq!(y.get_bits::<6>(3), 35_u32);

    let z0: Bits<9> = 2_usize.into();
    let z1: Bits<9> = 0_usize.into();
    let z2: Bits<9> = 42_usize.into();
    let z: Bits<9> = z0 | z1 << 2_usize | z2 << 3_usize;
    let x: MIGCmd = z.into();
    assert_eq!(x.active, false);
    assert_eq!(x.cmd, CmdType::Write);
    assert_eq!(x.len, 42_u32);
}
