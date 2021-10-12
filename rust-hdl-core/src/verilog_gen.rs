use crate::ast::{
    VerilogBlock, VerilogBlockOrConditional, VerilogCase, VerilogConditional, VerilogExpression,
    VerilogLink, VerilogLiteral, VerilogLoop, VerilogMatch, VerilogOp, VerilogOpUnary,
};
use crate::code_writer::CodeWriter;
use crate::verilog_visitor::{walk_block, VerilogVisitor};
use evalexpr::ContextWithMutableVariables;
use num_bigint::BigUint;
use regex::Regex;

struct LoopVariable {
    variable: String,
    value: usize,
}

pub struct VerilogCodeGenerator {
    io: CodeWriter,
    loops: Vec<LoopVariable>,
    links: Vec<VerilogLink>,
}

impl VerilogCodeGenerator {
    pub fn new() -> VerilogCodeGenerator {
        Self {
            io: CodeWriter::new(),
            loops: vec![],
            links: vec![],
        }
    }

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

pub fn verilog_combinatorial(code: &VerilogBlock) -> String {
    let mut gen = VerilogCodeGenerator::new();
    gen.visit_block(code);

    // add forward links to the code
    let links = gen
        .links
        .iter()
        .map(|x| {
            match x {
                VerilogLink::Forward(x) => {
                    format!(
                        "always @(*) {}${} = {}${};",
                        x.other_name, x.my_name, x.owner_name, x.my_name
                    )
                }
                VerilogLink::Backward(x) => {
                    format!(
                        "always @(*) {}${} = {}${};",
                        x.owner_name, x.my_name, x.other_name, x.my_name
                    )
                }
                VerilogLink::Bidirectional(x) => {
                    format!(
                        "assign {}${} = {}${};",
                        x.owner_name, x.my_name, x.other_name, x.my_name
                    )
                }
            }
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");

    /*
    let links = gen
        .links
        .iter()
        .map(|x| {
            x.replace("link!(", "")
                .replace(")", "")
                .replace(",", "=")
                .replace("self.", "")
                .replace(".", "_")
        })
        .map(|x| format!("assign {};", x))
        .collect::<Vec<_>>()
        .join("\n");
     */
    format!("always @(*) {}\n{}", gen.to_string(), links)
    //    format!("always @(*) {}\n", gen.to_string())
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

    fn visit_link(&mut self, l: &[VerilogLink]) {
        for link in l {
            self.links.push(link.clone());
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
    context.set_value("i".to_string(), 5.into());
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
