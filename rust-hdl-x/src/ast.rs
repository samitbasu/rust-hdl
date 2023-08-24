pub struct Block {
    pub statements: Vec<Stmt>,
}

pub enum Stmt {
    Local(Local),
    Item(Item),
    Expr(Expr),
}

pub struct Local {
    pub pattern: LocalPattern,
    pub value: Box<Expr>,
}

pub enum LocalPattern {
    Ident(String),
    Tuple(Vec<LocalPattern>),
}

pub enum Item {
    Const(ItemConst),
}

pub struct ItemConst {
    pub name: String,
    pub value: Box<Expr>,
}

pub enum Expr {
    Binary(ExprBinary),
    Unary(ExprUnary),
    Match(ExprMatch),
    Return(Option<Box<Expr>>),
    If(ExprIf),
    Index(ExprIndex),
    Lit(ExprLit),
    Paren(Box<Expr>),
    Tuple(Vec<Expr>),
    ForLoop(ExprForLoop),
    Assign(ExprAssign),
    Group(Box<Expr>),
    Field(ExprField),
    Block(ExprBlock),
    Array(ExprArray),
    Range(ExprRange),
}

pub struct ExprArray {
    pub elems: Vec<Expr>,
}

pub struct ExprBlock {
    pub block: Block,
}

pub struct ExprField {
    pub base: Box<Expr>,
    pub member: Member,
}

pub enum Member {
    Named(String),
    Unnamed(u32),
}

pub struct ExprAssign {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

pub struct ExprBinary {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
    And,
    Or,
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
    AddAssign,
    SubAssign,
    MulAssign,
    BitXorAssign,
    BitAndAssign,
    BitOrAssign,
    ShlAssign,
    ShrAssign,
}

pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

pub enum UnOp {
    Neg,
    Not,
}

pub struct ExprIf {
    pub cond: Box<Expr>,
    pub then_branch: Block,
    pub else_branch: Option<Box<Expr>>,
}

pub struct ExprMatch {
    pub expr: Box<Expr>,
    pub arms: Vec<Arm>,
}

pub struct Arm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

pub enum Pattern {
    Lit(ExprLit),
    Or(Vec<Pattern>),
    Paren(Box<Pattern>),
}

pub enum ExprLit {
    Int(String),
    Bool(bool),
}

pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

pub struct ExprForLoop {
    pub pat: Box<LocalPattern>,
    pub expr: Box<Expr>,
    pub body: Block,
}

pub struct ExprRange {
    pub start: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub end: Option<Box<Expr>>,
}

pub enum RangeLimits {
    HalfOpen,
    Closed,
}
