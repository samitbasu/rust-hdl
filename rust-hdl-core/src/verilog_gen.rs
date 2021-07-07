use crate::code_writer::CodeWriter;
use crate::verilog_visitor::{VerilogVisitor, walk_block};
use crate::ast::{VerilogStatement, VerilogExpression, VerilogConditional, VerilogBlockOrConditional, VerilogMatch, VerilogCase, VerilogOp, VerilogOpUnary, VerilogBlock};

pub struct VerilogCodeGenerator {
    io: CodeWriter,
}

impl VerilogCodeGenerator {
    pub fn new() -> VerilogCodeGenerator {
        Self {
            io: CodeWriter::new(),
        }
    }
}

impl ToString for VerilogCodeGenerator {
    fn to_string(&self) -> String {
        self.io.to_string()
    }
}

fn ident_fixup(a: &str) -> String {
    let mut x = a.to_owned();
    if x.starts_with(".") {
        x.remove(0);
    }
    x.replace(".","_").replace("::", "_")
}


fn verilog_literal(v: &u128, w: &usize) -> String {
    if w % 4 != 0 && *w < 20 {
        format!("{}'b{:b}", w, v)
    } else {
        format!("{}'h{:x}", w, v)
    }
}

impl VerilogVisitor for VerilogCodeGenerator {
    fn visit_signal(&mut self, sig: &str) {
        self.io.write(ident_fixup(sig));
    }

    fn visit_literal(&mut self, v: &u128, w: &usize) {
        self.io.write(verilog_literal(v, w));
    }

    fn visit_paren(&mut self, e: &VerilogExpression) {
        self.io.write("(");
        self.visit_expression(e);
        self.io.write(")");
    }

    fn visit_slice_assignment(&mut self, base: &str, width: &usize, offset: &VerilogExpression, replacement: &VerilogExpression) {
        self.io.write(format!("{}[(", base));
        self.visit_expression(offset);
        self.io.write(format!(")+:({})] = ", width));
        self.visit_expression(replacement);
        self.io.writeln(";");
    }

    fn visit_block(&mut self, b: &VerilogBlock) {
        self.io.writeln("begin");
        self.io.push();
        walk_block(self, b);
        self.io.pop();
        self.io.add_line("end");
    }

    fn visit_conditional(&mut self, c: &VerilogConditional) {
        self.io.write("if (");
        self.visit_expression(&c.test);
        self.io.write(") ");
        self.visit_block(&c.then);
        self.visit_block_or_conditional(&c.otherwise);
    }

    fn visit_block_or_conditional(&mut self, o: &VerilogBlockOrConditional) {
        match &o {
            VerilogBlockOrConditional::Block(b) => {
                self.io.write("else ");
                self.visit_block(&b);
            }
            VerilogBlockOrConditional::Conditional(c) => {
                self.io.write("else " );
                self.visit_statement(c);
            }
            VerilogBlockOrConditional::None => {}
        }
    }

    fn visit_match(&mut self, m: &VerilogMatch) {
        self.io.write("case (");
        self.visit_expression(&m.test);
        self.io.writeln(")");
        self.io.push();
        m.cases.iter().for_each(|x| self.visit_case(x));
        self.io.pop();
        self.io.writeln("endcase")
    }

    fn visit_comment(&mut self, x: &str) {
        self.io.add(format!("// {}", x));
    }

    fn visit_case(&mut self, c: &VerilogCase) {
        self.io.write(ident_fixup(&c.condition));
        self.io.writeln(":");
        self.io.push();
        self.visit_block(&c.block);
        self.io.pop();
    }

    fn visit_binop(&mut self, l: &VerilogExpression, o: &VerilogOp, r: &VerilogExpression) {
        self.visit_expression(l);
        self.io.write(match o {
            VerilogOp::Add => "+",
            VerilogOp::Sub => "-",
            VerilogOp::Mul => "*",
            VerilogOp::LogicalAnd => "&&",
            VerilogOp::LogicalOr => "||",
            VerilogOp::BitXor => "^",
            VerilogOp::BitAnd => "&",
            VerilogOp::BitOr => "|",
            VerilogOp::Shl => "<<",
            VerilogOp::Shr => ">>",
            VerilogOp::Eq => "==",
            VerilogOp::Lt => "<",
            VerilogOp::Le => "<=",
            VerilogOp::Ne => "!=",
            VerilogOp::Ge => ">=",
            VerilogOp::Gt => ">",
        });
        self.visit_expression(r);
    }

    fn visit_unop(&mut self, o: &VerilogOpUnary, r: &VerilogExpression) {
        self.io.write(match o {
            VerilogOpUnary::Not => "~",
            VerilogOpUnary::Neg => "-",
            VerilogOpUnary::All => "&",
            VerilogOpUnary::Any => "|",
        });
        self.visit_expression(r);
    }


    fn visit_assignment(&mut self, l: &VerilogExpression, r: &VerilogExpression) {
        self.visit_expression(l);
        self.io.write(" = ");
        self.visit_expression(r);
        self.io.writeln(";");
    }
}