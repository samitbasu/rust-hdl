#[derive(Debug)]
pub enum Stmt {
    Local(Local),
    Expr(Expr),
    Semi(Expr),
}

#[derive(Debug)]
pub struct Local {
    pub pattern: LocalPattern,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub enum LocalPattern {
    Ident(LocalIdent),
    Tuple(Vec<LocalPattern>),
}

#[derive(Debug)]
pub struct LocalIdent {
    pub name: String,
    pub mutable: bool,
}

#[derive(Debug)]
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
    Block(Vec<Stmt>),
    Array(ExprArray),
    Range(ExprRange),
    Ident(String),
}

#[derive(Debug)]
pub struct ExprArray {
    pub elems: Vec<Expr>,
}

#[derive(Debug)]
pub struct ExprField {
    pub base: Box<Expr>,
    pub member: Member,
}

#[derive(Debug)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Debug)]
pub struct ExprAssign {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug)]
pub struct ExprBinary {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug)]
pub struct ExprIf {
    pub cond: Box<Expr>,
    pub then_branch: Vec<Stmt>,
    pub else_branch: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct ExprMatch {
    pub expr: Box<Expr>,
    pub arms: Vec<Arm>,
}

#[derive(Debug)]
pub struct Arm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

#[derive(Debug)]
pub enum Pattern {
    Lit(ExprLit),
    Or(Vec<Pattern>),
    Paren(Box<Pattern>),
}

#[derive(Debug)]
pub enum ExprLit {
    Int(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Debug)]
pub struct ExprForLoop {
    pub pat: Box<LocalPattern>,
    pub expr: Box<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct ExprRange {
    pub start: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub end: Option<Box<Expr>>,
}

#[derive(Debug)]
pub enum RangeLimits {
    HalfOpen,
    Closed,
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_fn() {
        fn jnk() -> Vec<Stmt> {
            vec![
                Stmt::Local(Local {
                    pattern: LocalPattern::Ident(stringify!(a).to_string()),
                    value: Box::new(Expr::Lit(ExprLit::Int("1".to_string()))),
                }),
                Stmt::Local(Local {
                    pattern: LocalPattern::Ident(stringify!(b).to_string()),
                    value: Box::new(Expr::Lit(ExprLit::Int("2".to_string()))),
                }),
                Stmt::Local(Local {
                    pattern: LocalPattern::Ident(stringify!(c).to_string()),
                    value: Box::new(Expr::Binary(ExprBinary {
                        op: BinOp::Add,
                        lhs: Box::new(Expr::Ident(stringify!(a).to_string())),
                        rhs: Box::new(Expr::Ident(stringify!(b).to_string())),
                    })),
                }),
                Stmt::Expr(Expr::Ident(stringify!(c).to_string())),
            ]
        }
        println!("{:#?}", jnk());
    }
}
