#[derive(Debug, Clone)]
pub enum Verilog {
    Empty,
    Combinatorial(VerilogBlock),
    Custom(String),
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
pub enum VerilogExpression {
    Signal(String),
    Literal(u128, usize),
    Cast(Box<VerilogExpression>, usize),
    Paren(Box<VerilogExpression>),
    Binary(Box<VerilogExpression>, VerilogOp, Box<VerilogExpression>),
    Unary(VerilogOpUnary, Box<VerilogExpression>),
    Index(String, Box<VerilogExpression>),
    Slice(String, usize, Box<VerilogExpression>),
    IndexReplace(String, Box<VerilogExpression>, Box<VerilogExpression>),
    PopBit(String),
    PushBit(String, Box<VerilogExpression>),
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
