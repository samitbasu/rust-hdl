use crate::ast::{
    VerilogBlock, VerilogBlockOrConditional, VerilogCase, VerilogConditional, VerilogExpression,
    VerilogIndexAssignment, VerilogMatch, VerilogOp, VerilogOpUnary, VerilogStatement,
};

pub trait VerilogVisitor {
    fn visit_block(&mut self, b: &VerilogBlock) {
        walk_block(self, b);
    }

    fn visit_statement(&mut self, s: &VerilogStatement) {
        walk_statement(self, s);
    }

    fn visit_index_assignment(&mut self, a: &VerilogIndexAssignment) {
        walk_index_assignment(self, a);
    }

    fn visit_slice_assignment(
        &mut self,
        base: &str,
        width: &usize,
        offset: &VerilogExpression,
        replacement: &VerilogExpression,
    ) {
        walk_slice_assignment(self, base, width, offset, replacement);
    }

    fn visit_conditional(&mut self, c: &VerilogConditional) {
        walk_conditional(self, c);
    }

    fn visit_block_or_conditional(&mut self, c: &VerilogBlockOrConditional) {
        walk_block_or_conditional(self, c);
    }

    fn visit_match(&mut self, m: &VerilogMatch) {
        walk_match(self, m);
    }

    fn visit_comment(&mut self, c: &str) {
        // Terminal
    }

    fn visit_signal(&mut self, c: &str) {
        // Terminal
    }

    fn visit_literal(&mut self, a: &u128, b: &usize) {
        // Terminal
    }

    fn visit_case(&mut self, c: &VerilogCase) {
        walk_case(self, c);
    }

    fn visit_lhs_expression(&mut self, e: &VerilogExpression) {
        walk_lhs_expression(self, e);
    }

    fn visit_expression(&mut self, e: &VerilogExpression) {
        walk_expression(self, e);
    }

    fn visit_binop(&mut self, l: &VerilogExpression, o: &VerilogOp, r: &VerilogExpression) {
        walk_binop(self, l, o, r);
    }

    fn visit_unop(&mut self, o: &VerilogOpUnary, ex: &VerilogExpression) {
        walk_unop(self, o, ex);
    }

    fn visit_assignment(&mut self, l: &VerilogExpression, r: &VerilogExpression) {
        walk_assignment(self, l, r);
    }

    fn visit_paren(&mut self, p: &VerilogExpression) {
        walk_paren(self, p);
    }

    fn visit_cast(&mut self, a: &VerilogExpression, b: &usize) {
        walk_cast(self, a, b);
    }

    fn visit_index(&mut self, a: &str, b: &VerilogExpression) {
        walk_index(self, a, b);
    }

    fn visit_slice(&mut self, a: &str, b: &usize, c: &VerilogExpression) {
        walk_slice(self, a, b, c);
    }

    fn visit_index_replace(&mut self, a: &str, b: &VerilogExpression, c: &VerilogExpression) {
        walk_index_replacement(self, a, b, c);
    }
}

pub fn walk_index_replacement<V: VerilogVisitor + ?Sized>(visitor: &mut V, a: &str, b: &VerilogExpression, c: &VerilogExpression) {
    visitor.visit_signal(a);
    visitor.visit_expression(b);
    visitor.visit_expression(c);
}

pub fn walk_slice<V: VerilogVisitor + ?Sized>(visitor: &mut V, a: &str, b: &usize, c: &VerilogExpression) {
    visitor.visit_signal(a);
    visitor.visit_expression(c);
}

pub fn walk_index<V: VerilogVisitor + ?Sized>(visitor: &mut V, a: &str, b: &VerilogExpression) {
    visitor.visit_signal(a);
    visitor.visit_expression(b);
}

pub fn walk_cast<V: VerilogVisitor + ?Sized>(visitor: &mut V, a: &VerilogExpression, b: &usize) {
    visitor.visit_expression(a)
}

pub fn walk_paren<V: VerilogVisitor + ?Sized>(visitor: &mut V, p: &VerilogExpression) {
    visitor.visit_expression(p);
}

pub fn walk_block<V: VerilogVisitor + ?Sized>(visitor: &mut V, b: &VerilogBlock) {
    for s in b {
        visitor.visit_statement(s)
    }
}

pub fn walk_slice_assignment<V: VerilogVisitor + ?Sized>(
    visitor: &mut V,
    base: &str,
    _width: &usize,
    offset: &VerilogExpression,
    replacement: &VerilogExpression,
) {
    visitor.visit_signal(base);
    visitor.visit_expression(offset);
    visitor.visit_expression(replacement);
}

pub fn walk_assignment<V: VerilogVisitor + ?Sized>(
    visitor: &mut V,
    l: &VerilogExpression,
    r: &VerilogExpression,
) {
    visitor.visit_expression(l);
    visitor.visit_expression(r);
}

pub fn walk_statement<V: VerilogVisitor + ?Sized>(visitor: &mut V, s: &VerilogStatement) {
    match s {
        VerilogStatement::Assignment(l, r) => {
            visitor.visit_assignment(l, r);
        }
        VerilogStatement::SliceAssignment {
            base,
            width,
            offset,
            replacement,
        } => {
            visitor.visit_slice_assignment(base, width, offset, replacement);
        }
        VerilogStatement::If(c) => {
            visitor.visit_conditional(c);
        }
        VerilogStatement::Match(m) => {
            visitor.visit_match(m);
        }
        VerilogStatement::Comment(x) => {
            visitor.visit_comment(x);
        }
    }
}

pub fn walk_index_assignment<V: VerilogVisitor + ?Sized>(visitor: &mut V, a: &VerilogIndexAssignment) {
    visitor.visit_expression(&a.value);
    visitor.visit_expression(&a.index);
    visitor.visit_expression(&a.target);
}

pub fn walk_conditional<V: VerilogVisitor + ?Sized>(visitor: &mut V, c: &VerilogConditional) {
    visitor.visit_expression(&c.test);
    visitor.visit_block(&c.then);
    visitor.visit_block_or_conditional(&c.otherwise);
}

pub fn walk_block_or_conditional<V: VerilogVisitor + ?Sized>(
    visitor: &mut V,
    o: &VerilogBlockOrConditional,
) {
    match o {
        VerilogBlockOrConditional::Block(b) => {
            visitor.visit_block(b);
        }
        VerilogBlockOrConditional::Conditional(c) => {
            visitor.visit_statement(&c);
        }
        VerilogBlockOrConditional::None => {
            // No-op
        }
    }
}

pub fn walk_match<V: VerilogVisitor + ?Sized>(visitor: &mut V, m: &VerilogMatch) {
    visitor.visit_expression(&m.test);
    for case in &m.cases {
        visitor.visit_case(case)
    }
}

pub fn walk_case<V: VerilogVisitor + ?Sized>(visitor: &mut V, c: &VerilogCase) {
    visitor.visit_block(&c.block)
}

pub fn walk_lhs_expression<V: VerilogVisitor + ?Sized>(visitor: &mut V, e: &VerilogExpression) {
    visitor.visit_expression(e)
}

pub fn walk_binop<V: VerilogVisitor + ?Sized>(
    visitor: &mut V,
    l: &VerilogExpression,
    _op: &VerilogOp,
    r: &VerilogExpression,
) {
    visitor.visit_expression(l);
    visitor.visit_expression(r);
}

pub fn walk_unop<V: VerilogVisitor + ?Sized>(
    visitor: &mut V,
    _op: &VerilogOpUnary,
    e: &VerilogExpression,
) {
    visitor.visit_expression(e);
}

pub fn walk_expression<V: VerilogVisitor + ?Sized>(visitor: &mut V, e: &VerilogExpression) {
    match e {
        VerilogExpression::Signal(s) => {
            visitor.visit_signal(s);
        }
        VerilogExpression::Literal(a, b) => {
            visitor.visit_literal(a, b);
        }
        VerilogExpression::Cast(a, b) => {
            visitor.visit_cast(a, b);
        }
        VerilogExpression::Paren(x) => {
            visitor.visit_paren(x);
        }
        VerilogExpression::Binary(l, op, r) => {
            visitor.visit_binop(l, op, r);
        }
        VerilogExpression::Unary(op, ex) => {
            visitor.visit_unop(op, ex);
        }
        VerilogExpression::Index(a, b) => {
            visitor.visit_index(a, b);
        }
        VerilogExpression::Slice(a, b, c) => {
            visitor.visit_slice(a, b, c);
        }
        VerilogExpression::IndexReplace(a, b, c) => {
            visitor.visit_index_replace(a, b, c);
        }
    }
}
