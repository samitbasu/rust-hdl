use std::fmt::{Display, Formatter};
use crate::core::ast::{Verilog, VerilogBlock, VerilogBlockOrConditional, VerilogCase, VerilogConditional, VerilogExpression, VerilogIndexAssignment, VerilogLink, VerilogLinkDetails, VerilogLiteral, VerilogLoop, VerilogMatch, VerilogOp, VerilogOpUnary, VerilogStatement};
use crate::core::atom::Atom;
use crate::core::block::Block;
use crate::core::logic::TimingMode;
use crate::core::named_path::NamedPath;
use crate::core::probe::Probe;
use crate::core::verilog_visitor::VerilogVisitor;
use std::fmt::Write;


#[derive(Clone, Debug, Default)]
pub struct SignalGraph {
    nodes: Vec<String>,
    edges: Vec<(usize, usize)>,
}

impl Display for SignalGraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for edge in &self.edges {
            let from = &self.nodes[edge.0];
            let to = &self.nodes[edge.1];
            writeln!(f, "{} <- {}", from, to)?
        }
        Ok(())
    }
}

impl SignalGraph {
    fn dot(&self) -> String {
        let mut dot = String::new();
        writeln!(dot, "digraph mygraph {{");
        writeln!(dot, "   node [shape=box]");
        for node in &self.nodes {
            writeln!(dot, "    \"{}\"", node);
        }
        for edge in &self.edges {
            writeln!(dot, "    \"{}\" -> \"{}\"", self.nodes[edge.1], self.nodes[edge.0]);
        }
        writeln!(dot, "}}");
        dot
    }
    fn add_signal_node(&mut self, name: &str) -> usize {
        for ndx in 0..self.nodes.len() {
            if self.nodes[ndx] == name {
                return ndx
            }
        }
        self.nodes.push(name.to_string());
        self.nodes.len() - 1
    }
    fn add_signal_edge(&mut self, from: &str, to: &str) {
        let from_index = self.add_signal_node(from);
        let to_index = self.add_signal_node(to);
        let edge = (from_index, to_index);
        for ndx in 0..self.edges.len() {
            if self.edges[ndx] == edge {
                return;
            }
        }
        self.edges.push(edge)
    }
}


#[derive(Clone, Debug, Copy)]
pub enum ExpressionMode {
    Write,
    Read,
}

pub struct TimingChecker {
    path: NamedPath,
    namespace: NamedPath,
    mode: ExpressionMode,
    write_name: String,
    read_names: Vec<String>,
    pub graph: SignalGraph,
}

impl Default for TimingChecker {
    fn default() -> Self {
        Self {
            path: Default::default(),
            namespace: Default::default(),
            mode: ExpressionMode::Write,
            write_name: "".to_string(),
            read_names: vec![],
            graph: Default::default()
        }
    }
}

impl TimingChecker {
    fn reset_read_write(&mut self) {
        self.write_name.clear();
        self.read_names.clear();
    }
    fn add_code(&mut self, module: &str, code: Verilog) {
        match &code {
            Verilog::Combinatorial(code) => {
                self.visit_block(code);
            },
            _ => {}
        }
    }
    fn capture_assignments(&mut self) {
        for read in &self.read_names {
            self.graph.add_signal_edge(&self.write_name, &read);
        }
        self.reset_read_write();
    }
    fn link_fixup(&self, x: &VerilogLinkDetails) -> (String, String) {
        let v1 = format!("{}${}${}", self.path.to_string(), x.other_name.replace("[","$").replace("]",""), x.my_name);
        let v2 = format!("{}${}${}", self.path.to_string(), x.owner_name.replace("[","$").replace("]",""), x.my_name);
        (v1, v2)
    }
}

impl VerilogVisitor for TimingChecker {
    fn visit_assignment(&mut self, l: &VerilogExpression, r: &VerilogExpression) {
        self.reset_read_write();
        self.mode = ExpressionMode::Write;
        self.visit_expression(l);
        self.mode = ExpressionMode::Read;
        self.visit_expression(r);
        self.capture_assignments();
    }
    fn visit_slice_assignment(&mut self, base: &VerilogExpression, width: &usize, offset: &VerilogExpression, replacement: &VerilogExpression) {
        self.reset_read_write();
        self.mode = ExpressionMode::Write;
        self.visit_expression(base);
        self.mode = ExpressionMode::Read;
        self.visit_expression(offset);
        self.visit_expression(replacement);
        self.capture_assignments();
    }
    fn visit_signal(&mut self, c: &str) {
        let c = format!("{}${}", self.path.to_string(), c).replace("$next", "");
        match self.mode {
            ExpressionMode::Write => {
                self.write_name = c
            }
            ExpressionMode::Read => {
                self.read_names.push(c)
            }
        }
    }
    fn visit_link(&mut self, c: &[VerilogLink]) {
        for link in c {
            self.reset_read_write();
            match link {
                VerilogLink::Forward(x) => {
                    let (w, r) = self.link_fixup(x);
                    self.write_name = w;
                    self.read_names = vec![r];
                    self.capture_assignments();
                }
                VerilogLink::Backward(x) => {
                    let (r, w) = self.link_fixup(x);
                    self.write_name = w;
                    self.read_names = vec![r];
                    self.capture_assignments();
                }
                VerilogLink::Bidirectional(x) => {
                    let (w, r) = self.link_fixup(x);
                    self.write_name = w;
                    self.read_names = vec![r];
                    self.capture_assignments();
                    self.reset_read_write();
                    let (w, r) = self.link_fixup(x);
                    self.write_name = r;
                    self.read_names = vec![w];
                    self.capture_assignments();
                }
            }
        }
    }
}


impl Probe for TimingChecker {
    fn visit_start_scope(&mut self, name: &str, node: &dyn Block) {
        self.path.push(name);
        self.namespace.reset();
        println!("Start Scope: {} timing {:?}", self.path.to_string(), node.timing_mode());
        match node.timing_mode() {
            TimingMode::Constant => {}
            TimingMode::Normal => {
                self.add_code(&self.path.to_string(), node.hdl());
            }
            TimingMode::DFF => {
                self.reset_read_write();
                self.write_name = format!("{}$q", self.path.to_string());
                self.read_names = vec![
                     format!("{}$d", self.path.to_string()),
                     format!("{}$clock", self.path.to_string()),
                     format!("{}$reset", self.path.to_string()),
                ];
                self.capture_assignments();
            }
        }
    }
    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        self.namespace.push(name);
    }
    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        let module_path = self.path.to_string();
        let module_name = self.path.last();
        let namespace = self.namespace.flat("$");
        let name = if namespace.is_empty() {
            name.to_owned()
        } else {
            format!("{}${}", namespace, name)
        };
        // Atoms could be inputs, outputs or local
        println!("Atom: {} {} {:?} {}", module_path.to_string(), name, signal.kind(), signal.id());
    }
    fn visit_end_namespace(&mut self, _name: &str, _node: &dyn Block) {
        self.namespace.pop();
    }
    fn visit_end_scope(&mut self, _name: &str, _node: &dyn Block) {
        self.path.pop();
    }
}

pub fn check_timing<U: Block>(uut: &U) {
    let mut scan = TimingChecker::default();
    uut.accept("top", &mut scan);
    //println!("{}",&scan.graph);
    let dot = scan.graph.dot();
    std::fs::write("dag.dot", dot);
//    println!("{}", dot);
}

