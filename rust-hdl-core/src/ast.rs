use crate::bits::Bits;
use num_bigint::BigUint;
use std::fmt::{Display, Formatter, LowerHex};

#[derive(Debug, Clone)]
pub struct BlackBox {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Wrapper {
    pub code: String,
    pub cores: String,
}

#[derive(Debug, Clone)]
pub enum Verilog {
    Empty,
    Combinatorial(VerilogBlock),
    Custom(String),
    Blackbox(BlackBox),
    Wrapper(Wrapper),
}

impl Default for Verilog {
    fn default() -> Self {
        Self::Empty
    }
}

pub type VerilogBlock = Vec<VerilogStatement>;

#[derive(Debug, Clone)]
pub enum VerilogStatement {
    Assignment(VerilogExpression, VerilogExpression),
    SliceAssignment {
        base: String,
        width: usize,
        offset: VerilogExpression,
        replacement: VerilogExpression,
    },
    If(VerilogConditional),
    Match(VerilogMatch),
    Loop(VerilogLoop),
    Comment(String),
    Link(Vec<VerilogLink>),
}

#[derive(Debug, Clone)]
pub enum VerilogLink {
    Forward(VerilogLinkDetails),
    Backward(VerilogLinkDetails),
    Bidirectional(VerilogLinkDetails),
}

#[derive(Debug, Clone)]
pub struct VerilogLinkDetails {
    pub my_name: String,
    pub owner_name: String,
    pub other_name: String,
}

#[derive(Debug, Clone)]
pub struct VerilogIndexAssignment {
    pub target: VerilogExpression,
    pub index: VerilogExpression,
    pub value: VerilogExpression,
}

#[derive(Debug, Clone)]
pub struct VerilogConditional {
    pub test: VerilogExpression,
    pub then: VerilogBlock,
    pub otherwise: VerilogBlockOrConditional,
}

#[derive(Debug, Clone)]
pub struct VerilogLoop {
    pub index: String,
    pub from: VerilogLiteral,
    pub to: VerilogLiteral,
    pub block: VerilogBlock,
}

#[derive(Debug, Clone)]
pub enum VerilogBlockOrConditional {
    Block(VerilogBlock),
    Conditional(Box<VerilogStatement>),
    None,
}

#[derive(Debug, Clone)]
pub struct VerilogMatch {
    pub test: VerilogExpression,
    pub cases: Vec<VerilogCase>,
}

#[derive(Debug, Clone)]
pub struct VerilogCase {
    pub condition: String,
    pub block: VerilogBlock,
}

#[derive(Debug, Clone)]
pub struct VerilogLiteral {
    val: BigUint,
    bits: usize,
}

impl VerilogLiteral {
    pub fn as_usize(&self) -> usize {
        let m = self.val.to_u32_digits();
        match m.len() {
            0 => 0,
            1 => m[0] as usize,
            _ => panic!("Loop index is too large!"),
        }
    }
}

impl From<bool> for VerilogLiteral {
    fn from(x: bool) -> Self {
        let bi: BigUint = if x { 1_u32 } else { 0_u32 }.into();
        VerilogLiteral { val: bi, bits: 1 }
    }
}

macro_rules! define_literal_from_uint {
    ($name: ident, $width: expr) => {
        impl From<$name> for VerilogLiteral {
            fn from(x: $name) -> Self {
                let bi: BigUint = x.into();
                VerilogLiteral {
                    val: bi,
                    bits: $width,
                }
            }
        }
    };
}

define_literal_from_uint!(u128, 128);
define_literal_from_uint!(u64, 64);
define_literal_from_uint!(u32, 32);
define_literal_from_uint!(u16, 16);
define_literal_from_uint!(u8, 8);
#[cfg(target_pointer_width = "64")]
define_literal_from_uint!(usize, 64);
#[cfg(target_pointer_width = "32")]
define_literal_from_uint!(usize, 32);

impl<const N: usize> From<Bits<N>> for VerilogLiteral {
    fn from(x: Bits<N>) -> Self {
        let mut z = BigUint::default();
        for i in 0..N {
            z.set_bit(i as u64, x.get_bit(i));
        }
        VerilogLiteral { val: z, bits: N }
    }
}

impl Display for VerilogLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bits = self.bits;
        Display::fmt(&bits, f)?;
        Display::fmt("'", f)?;
        if bits % 4 != 0 && self.bits < 20 {
            Display::fmt("b", f)?;
            std::fmt::Binary::fmt(&self.val, f)
        } else {
            Display::fmt("h", f)?;
            std::fmt::LowerHex::fmt(&self.val, f)
        }
    }
}

impl LowerHex for VerilogLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bits = self.bits;
        Display::fmt(&bits, f)?;
        Display::fmt("'h", f)?;
        LowerHex::fmt(&self.val, f)
    }
}

#[derive(Debug, Clone)]
pub enum VerilogExpression {
    Signal(String),
    Literal(VerilogLiteral),
    Cast(Box<VerilogExpression>, usize),
    Paren(Box<VerilogExpression>),
    Binary(Box<VerilogExpression>, VerilogOp, Box<VerilogExpression>),
    Unary(VerilogOpUnary, Box<VerilogExpression>),
    Index(Box<VerilogExpression>, Box<VerilogExpression>),
    Slice(Box<VerilogExpression>, usize, Box<VerilogExpression>),
    IndexReplace(
        Box<VerilogExpression>,
        Box<VerilogExpression>,
        Box<VerilogExpression>,
    ),
}

#[derive(Debug, Clone)]
pub enum VerilogOp {
    Add,
    Sub,
    Mul,
    LogicalAnd,
    LogicalOr,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

#[derive(Debug, Clone)]
pub enum VerilogOpUnary {
    Not,
    Neg,
    All,
    Any,
    Xor,
}
