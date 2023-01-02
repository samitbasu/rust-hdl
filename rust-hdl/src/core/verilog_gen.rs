use evalexpr::ContextWithMutableVariables;
use num_bigint::BigUint;
use regex::Regex;

use crate::core::ast::{
    VerilogBlock, VerilogBlockOrConditional, VerilogCase, VerilogConditional, VerilogExpression,
    VerilogLink, VerilogLinkDetails, VerilogLiteral, VerilogLoop, VerilogMatch, VerilogOp,
    VerilogOpUnary,
};
use crate::core::code_writer::CodeWriter;
use crate::core::verilog_visitor::{walk_block, VerilogVisitor};

struct LoopVariable {
    variable: String,
    value: usize,
}

#[derive(Default)]
struct VerilogCodeGenerator {
    io: CodeWriter,
    loops: Vec<LoopVariable>,
    links: Vec<VerilogLink>,
}

impl VerilogCodeGenerator {
    fn array_index_simplification(&self, a: &str) -> String {
        let re = Regex::new(r"\[([^\]]*)\]").unwrap();
        let mut context = evalexpr::HashMapContext::new();
        for lvar in &self.loops {
            let _ = context.set_value(lvar.variable.clone(), (lvar.value as i64).into());
        }
        for x in re.captures(a) {
            if x.len() == 2 {
                if let Some(txt) = x.get(1) {
                    let arg = evalexpr::eval_with_context(txt.as_str(), &context).unwrap();
                    return re.replace(a, format!("$${}", arg)).to_string();
                }
            }
        }
        a.to_string()
    }

    fn link_fixup(&self, x: &VerilogLinkDetails) -> VerilogLinkDetails {
        VerilogLinkDetails {
            my_name: self.ident_fixup(&x.my_name),
            owner_name: self.ident_fixup(&x.owner_name),
            other_name: self.ident_fixup(&x.other_name),
        }
    }

    fn ident_fixup(&self, a: &str) -> String {
        let mut x = a.to_owned();
        for index in &self.loops {
            if x == index.variable {
                x = format!("{}", index.value);
            }
        }
        if x.starts_with(".") {
            x.remove(0);
        }
        x = x
            .replace(".", "$")
            .replace("::", "$")
            .trim_end_matches("$next")
            .to_owned();
        if x.contains('[') {
            x = self.array_index_simplification(&x);
        }
        x
    }
}

impl ToString for VerilogCodeGenerator {
    fn to_string(&self) -> String {
        self.io.to_string()
    }
}

pub fn verilog_link_extraction(code: &VerilogBlock) -> Vec<VerilogLink> {
    let mut gen = VerilogCodeGenerator::default();
    gen.visit_block(code);
    gen.links
}

pub fn verilog_combinatorial(code: &VerilogBlock) -> String {
    let mut gen = VerilogCodeGenerator::default();
    gen.visit_block(code);
    format!("always @(*) {}\n", gen.to_string())
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
                value: i,
            });
            walk_block(self, &a.block);
            self.loops.pop();
        }
    }

    fn visit_slice_assignment(
        &mut self,
        base: &VerilogExpression,
        width: &usize,
        offset: &VerilogExpression,
        replacement: &VerilogExpression,
    ) {
        self.visit_expression(base);
        self.io.write("[(");
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

    fn visit_link(&mut self, l: &[VerilogLink]) {
        for link in l {
            self.links.push(match link {
                VerilogLink::Forward(x) => VerilogLink::Forward(self.link_fixup(x)),
                VerilogLink::Backward(x) => VerilogLink::Backward(self.link_fixup(x)),
                VerilogLink::Bidirectional(x) => VerilogLink::Bidirectional(self.link_fixup(x)),
            })
        }
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
            VerilogOpUnary::Xor => "^",
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

    fn visit_signed(&mut self, a: &VerilogExpression) {
        self.io.write("$signed(");
        self.visit_expression(a);
        self.io.write(")");
    }

    fn visit_unsigned(&mut self, a: &VerilogExpression) {
        self.io.write("$unsigned(");
        self.visit_expression(a);
        self.io.write(")");
    }

    fn visit_index(&mut self, a: &VerilogExpression, b: &VerilogExpression) {
        self.visit_expression(a);
        self.io.write("[");
        self.visit_expression(b);
        self.io.write("]");
    }

    fn visit_slice(&mut self, sig: &VerilogExpression, width: &usize, offset: &VerilogExpression) {
        self.visit_expression(sig);
        self.io.write("[(");
        self.visit_expression(offset);
        self.io.write(format!(")+:({})]", width));
    }

    fn visit_index_replace(
        &mut self,
        sig: &VerilogExpression,
        ndx: &VerilogExpression,
        val: &VerilogExpression,
    ) {
        self.io.write("(");
        self.visit_expression(sig);
        self.io.write(" & ~(1 << (");
        self.visit_expression(ndx);
        self.io.write(")) | ((");
        self.visit_expression(val);
        self.io.write(") << (");
        self.visit_expression(ndx);
        self.io.write(")))");
    }
}

#[test]
fn test_array_replacement() {
    let re = Regex::new(r"\[([^\]]*)\]").unwrap();
    let test = "a[((i+1))]";
    let captures = re.captures(test);
    let mut context = evalexpr::HashMapContext::new();
    context.set_value("i".to_string(), 5.into()).unwrap();
    for x in re.captures(test) {
        println!("Match {:?}", x);
        if x.len() == 2 {
            if let Some(txt) = x.get(1) {
                let arg = evalexpr::eval_with_context(txt.as_str(), &context).unwrap();
                println!("Replace {} -> {}", txt.as_str(), arg);
                println!("Update {}", re.replace(test, format!("$${}", arg)))
            }
        }
    }
    assert!(captures.is_some());
}

pub fn filter_blackbox_directives(t: &str) -> String {
    let mut in_black_box = false;
    let mut ret = vec![];
    for line in t.split("\n") {
        in_black_box = in_black_box || line.starts_with("(* blackbox *)");
        if !in_black_box {
            ret.push(line);
        }
        if line.starts_with("endmodule") {
            in_black_box = false;
        }
    }
    ret.join("\n")
}

#[test]
fn test_filter_bb_directives() {
    let p = r#"
blah
more code
goes here

(* blackbox *)
module my_famous_module(
    super_secret_arg1,
    super_secret_arg2,
    super_secret_arg3);
/* Comment */
endmodule

stuff
"#;
    let q = filter_blackbox_directives(p);
    println!("{}", q);
    assert!(!q.contains("blackbox"));
    assert!(!q.contains("module"));
    assert!(!q.contains("endmodule"));
    assert!(q.contains("more code"));
    assert!(q.contains("stuff"));
}
