use num_bigint::BigUint;
use crate::bits::Bits;

#[derive(Debug, Clone)]
pub enum Verilog {
    Empty,
    Combinatorial(VerilogBlock),
    Custom(String),
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
    Comment(String),
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
pub struct VerilogLiteral(pub(crate) BigUint);

impl From<bool> for VerilogLiteral {
    fn from(x: bool) -> Self {
        let bi: BigUint = if x {
            1_u32
        } else {
            0_u32
        }.into();
        VerilogLiteral(bi)
    }
}

impl From<u32> for VerilogLiteral {
    fn from(x: u32) -> Self {
        let bi: BigUint = x.into();
        Self(bi)
    }
}

impl From<usize> for VerilogLiteral {
    fn from(x: usize) -> Self {
        let bi: BigUint = x.into();
        Self(bi)
    }
}

impl<const N: usize> From<Bits<N>> for VerilogLiteral {
    fn from(x: Bits<N>) -> Self {
        let mut z = BigUint::default();
        for i in 0..N {
            z.set_bit(i as u64, x.get_bit(i));
        }
        VerilogLiteral(z)
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
    Index(String, Box<VerilogExpression>),
    Slice(String, usize, Box<VerilogExpression>),
    IndexReplace(String, Box<VerilogExpression>, Box<VerilogExpression>),
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
}
