use crate::ast::{VerilogBlock, VerilogBlockOrConditional, VerilogCase, VerilogConditional, VerilogExpression, VerilogLiteral, VerilogMatch, VerilogOp, VerilogOpUnary, VerilogLoop};
use crate::code_writer::CodeWriter;
use crate::verilog_visitor::{walk_block, VerilogVisitor};
use num_bigint::BigUint;

struct LoopVariable {
    variable: String,
    value: usize,
}

pub struct VerilogCodeGenerator {
    io: CodeWriter,
    loops: Vec<LoopVariable>
}

impl VerilogCodeGenerator {
    pub fn new() -> VerilogCodeGenerator {
        Self {
            io: CodeWriter::new(),
            loops: vec![],
        }
    }

    fn ident_fixup(&self, a: &str) -> String {
        let mut x = a.to_owned();
        if x.starts_with(".") {
            x.remove(0);
        }
        x = x.replace(".", "_")
            .replace("::", "_")
            .trim_end_matches("_next")
            .to_owned();
        for index in &self.loops {
            x = x.replace(&format!("${}", index.variable), &format!("_{}", index.value));
        }
        x = x.replace("$", "_");
        x
    }

}

impl ToString for VerilogCodeGenerator {
    fn to_string(&self) -> String {
        self.io.to_string()
    }
}

pub fn verilog_combinatorial(code: &VerilogBlock) -> String {
    let mut gen = VerilogCodeGenerator::new();
    gen.visit_block(code);
    format!("always @(*) {}", gen.to_string())
}


impl VerilogVisitor for VerilogCodeGenerator {
    fn visit_block(&mut self, b: &VerilogBlock) {
        self.io.writeln("begin");
        self.io.push();
        walk_block(self, b);
        self.io.pop();
        self.io.add_line("end");
    }

    fn visit_loop(&mut self, a: &VerilogLoop) {
        let start = a.from.as_usize();
        let end = a.to.as_usize();
        for i in start..end {
            self.loops.push(LoopVariable {
                variable: a.index.clone(),
                value: i
            });
            walk_block(self, &a.block);
            self.loops.pop();
        }
    }

    fn visit_slice_assignment(
        &mut self,
        base: &str,
        width: &usize,
        offset: &VerilogExpression,
        replacement: &VerilogExpression,
    ) {
        self.io.write(format!("{}[(", base));
        self.visit_expression(offset);
        self.io.write(format!(")+:({})] = ", width));
        self.visit_expression(replacement);
        self.io.writeln(";");
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
                self.io.write("else ");
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

    fn visit_signal(&mut self, sig: &str) {
        self.io.write(self.ident_fixup(sig));
    }

    fn visit_literal(&mut self, v: &VerilogLiteral) {
        self.io.write(v.to_string());
    }

    fn visit_case(&mut self, c: &VerilogCase) {
        self.io.write(self.ident_fixup(&c.condition));
        self.io.writeln(":");
        self.io.push();
        self.visit_block(&c.block);
        self.io.pop();
    }

    fn visit_binop(&mut self, l: &VerilogExpression, o: &VerilogOp, r: &VerilogExpression) {
        self.visit_expression(l);
        self.io.write(" ");
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
        self.io.write(" ");
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

    fn visit_paren(&mut self, e: &VerilogExpression) {
        self.io.write("(");
        self.visit_expression(e);
        self.io.write(")");
    }

    fn visit_cast(&mut self, e: &VerilogExpression, bits: &usize) {
        self.io.write("(");
        self.visit_expression(e);
        let mask = (BigUint::from(1_u32) << bits) - 1_u32;
        self.io.write(format!(") & {}'h{:x}", bits, mask))
    }

    fn visit_index(&mut self, a: &str, b: &VerilogExpression) {
        self.visit_signal(a);
        self.io.write("[");
        self.visit_expression(b);
        self.io.write("]");
    }

    fn visit_slice(&mut self, sig: &str, width: &usize, offset: &VerilogExpression) {
        self.visit_signal(sig);
        self.io.write("[(");
        self.visit_expression(offset);
        self.io.write(format!(")+:({})]", width));
    }

    fn visit_index_replace(&mut self, sig: &str, ndx: &VerilogExpression, val: &VerilogExpression) {
        self.io.write("(");
        self.visit_signal(sig);
        self.io.write(" & ~(1 << (");
        self.visit_expression(ndx);
        self.io.write(")) | ((");
        self.visit_expression(val);
        self.io.write(") << (");
        self.visit_expression(ndx);
        self.io.write(")))");
    }
}
