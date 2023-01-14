use crate::ast::{
    Verilog, VerilogConditional, VerilogExpression, VerilogLink, VerilogLinkDetails, VerilogMatch,
};
use crate::atom::{Atom, AtomKind};
use crate::block::Block;
use crate::named_path::NamedPath;
use crate::probe::Probe;
use crate::type_descriptor::TypeKind;
use crate::verilog_visitor::VerilogVisitor;
use petgraph::algo::{connected_components, is_cyclic_directed};
use petgraph::dot::Dot;
use petgraph::prelude::*;
use petgraph::unionfind::UnionFind;
use petgraph::visit::NodeIndexable;
use std::collections::HashMap;

#[derive(Clone, Debug, Copy, PartialEq)]
enum SignalNodeKind {
    Normal,
    Bidirectional,
    Source,
    Sink,
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum SignalEdgeKind {
    Assign,
    Input,
    Clock,
    Reset,
    Output,
    Extern,
    Constant,
}

#[derive(Clone, Debug, PartialEq)]
struct SignalNode {
    pub name: String,
    pub kind: SignalNodeKind,
}

#[derive(Clone, Debug, Default)]
struct SignalGraph {
    pub graph: Graph<SignalNode, SignalEdgeKind, Directed>,
}

impl SignalGraph {
    fn dot(&self) -> String {
        format!("{:?}", Dot::new(&self.graph))
    }
    fn add_signal_node(&mut self, node: &SignalNode) -> NodeIndex {
        let index = self.graph.node_indices().find(|i| self.graph[*i].eq(node));
        match index {
            Some(n) => n,
            None => self.graph.add_node(node.clone()),
        }
    }
    fn add_signal_edge(&mut self, from: &SignalNode, to: NodeIndex, kind: SignalEdgeKind) {
        let from_index = self.add_signal_node(from);
        if !self.graph.contains_edge(from_index, to) {
            self.graph.add_edge(from_index, to, kind);
        }
    }
}

#[derive(Clone, Debug, Copy)]
enum ExpressionMode {
    Write,
    Read,
}

type ReadScope = Vec<SignalNode>;

struct TimingChecker {
    path: NamedPath,
    namespace: NamedPath,
    mode: ExpressionMode,
    write_name: String,
    read_names: Vec<ReadScope>,
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
            graph: Default::default(),
        }
    }
}

impl TimingChecker {
    fn set_write_mode(&mut self) {
        self.mode = ExpressionMode::Write;
    }
    fn set_read_mode(&mut self) {
        self.mode = ExpressionMode::Read;
    }
    fn push_read_scope(&mut self) {
        self.read_names.push(vec![]);
    }
    fn pop_read_scope(&mut self) {
        self.read_names.pop();
    }
    fn clear_scope(&mut self) {
        self.read_names.clear();
        self.read_names.push(Default::default());
    }
    fn add_read(&mut self, name: &str, kind: SignalNodeKind) {
        if self.read_names.is_empty() {
            self.read_names.push(ReadScope::default());
        }
        self.read_names.last_mut().unwrap().push(SignalNode {
            name: name.into(),
            kind: kind,
        })
    }
    fn add_code(&mut self, module: &str, code: Verilog) {
        if let Verilog::Combinatorial(code) = &code {
            self.visit_block(code);
        }
    }
    fn add_write(&mut self, write_name: &str, kind: SignalNodeKind, edge: SignalEdgeKind) {
        assert!(!write_name.is_empty());
        let write_node = SignalNode {
            name: write_name.into(),
            kind,
        };
        let write_id = self.graph.add_signal_node(&write_node);
        for scope in &self.read_names {
            for read in scope {
                println!("Adding edge from {:?} -> {:?}", read, write_node);
                self.graph.add_signal_edge(read, write_id, edge);
            }
        }
    }
    fn link_fixup(&self, x: &VerilogLinkDetails) -> (String, String) {
        let v1 = format!(
            "{}${}${}",
            self.path.to_string(),
            x.other_name.replace('[', "$").replace(']', ""),
            x.my_name
        );
        let v2 = format!(
            "{}${}${}",
            self.path.to_string(),
            x.owner_name.replace('[', "$").replace(']', ""),
            x.my_name
        );
        (v1, v2)
    }
}

impl VerilogVisitor for TimingChecker {
    fn visit_conditional(&mut self, c: &VerilogConditional) {
        self.push_read_scope();
        self.visit_expression(&c.test);
        self.visit_block(&c.then);
        self.visit_block_or_conditional(&c.otherwise);
        self.pop_read_scope();
    }
    fn visit_match(&mut self, m: &VerilogMatch) {
        self.visit_expression(&m.test);
        self.push_read_scope();
        for case in &m.cases {
            self.visit_case(case);
        }
        self.pop_read_scope();
    }
    fn visit_assignment(&mut self, l: &VerilogExpression, r: &VerilogExpression) {
        self.push_read_scope();
        self.write_name = Default::default();
        self.mode = ExpressionMode::Write;
        self.visit_expression(l);
        self.mode = ExpressionMode::Read;
        self.visit_expression(r);
        let write_name = self.write_name.clone();
        self.add_write(&write_name, SignalNodeKind::Normal, SignalEdgeKind::Assign);
        self.pop_read_scope();
    }
    fn visit_slice_assignment(
        &mut self,
        base: &VerilogExpression,
        width: &usize,
        offset: &VerilogExpression,
        replacement: &VerilogExpression,
    ) {
        self.push_read_scope();
        self.write_name = Default::default();
        self.mode = ExpressionMode::Write;
        self.visit_expression(base);
        self.mode = ExpressionMode::Read;
        self.visit_expression(offset);
        self.visit_expression(replacement);
        let write_name = self.write_name.clone();
        self.add_write(&write_name, SignalNodeKind::Normal, SignalEdgeKind::Assign);
        self.pop_read_scope();
    }
    fn visit_signal(&mut self, c: &str) {
        let c = format!("{}${}", self.path.to_string(), c).replace("$next", "");
        match self.mode {
            ExpressionMode::Write => self.write_name = c,
            ExpressionMode::Read => self.add_read(&c, SignalNodeKind::Normal),
        }
    }
    fn visit_link(&mut self, c: &[VerilogLink]) {
        for link in c {
            match link {
                VerilogLink::Forward(x) => {
                    self.push_read_scope();
                    let (w, r) = self.link_fixup(x);
                    self.add_read(&r, SignalNodeKind::Normal);
                    self.add_write(&w, SignalNodeKind::Normal, SignalEdgeKind::Assign);
                    self.pop_read_scope();
                }
                VerilogLink::Backward(x) => {
                    self.push_read_scope();
                    let (r, w) = self.link_fixup(x);
                    self.add_read(&r, SignalNodeKind::Normal);
                    self.add_write(&w, SignalNodeKind::Normal, SignalEdgeKind::Assign);
                    self.pop_read_scope();
                }
                VerilogLink::Bidirectional(x) => {
                    let (w, r) = self.link_fixup(x);
                    self.push_read_scope();
                    self.add_read(&r, SignalNodeKind::Bidirectional);
                    self.add_write(&w, SignalNodeKind::Bidirectional, SignalEdgeKind::Assign);
                    self.pop_read_scope();
                }
            }
        }
    }
}

impl Probe for TimingChecker {
    fn visit_start_scope(&mut self, name: &str, node: &dyn Block) {
        println!("Start scope {}", name);
        self.path.push(name);
        self.namespace.reset();
        self.add_code(&self.path.to_string(), node.hdl());
        for info in &node.timing() {
            // The timing info represents a register.  A register
            // adds a write dependency based on the clock
            // The use of the clock decouples the outputs from the inputs.
            //
            // We capture this by adding a fake node to the signal graph
            self.push_read_scope();
            self.add_read(
                &format!("{}${}", self.path.to_string(), info.name),
                SignalNodeKind::Source,
            );
            for output in &info.outputs {
                self.add_write(
                    &format!("{}${}", self.path.to_string(), output),
                    SignalNodeKind::Normal,
                    SignalEdgeKind::Output,
                );
            }
            self.pop_read_scope();
            let write_name = format!("{}${}", self.path.to_string(), info.name);
            for input in &info.inputs {
                self.push_read_scope();
                self.add_read(
                    &format!("{}${}", self.path.to_string(), input),
                    SignalNodeKind::Normal,
                );
                self.add_write(&write_name, SignalNodeKind::Sink, SignalEdgeKind::Input);
                self.pop_read_scope();
            }
            self.push_read_scope();
            self.add_read(
                &format!("{}${}", self.path.to_string(), info.clock),
                SignalNodeKind::Normal,
            );
            self.add_write(&write_name, SignalNodeKind::Sink, SignalEdgeKind::Clock);
            self.pop_read_scope();
        }
        self.clear_scope();
    }
    fn visit_start_namespace(&mut self, name: &str, _node: &dyn Block) {
        println!("Start Namespace {}", name);
        self.namespace.push(name);
    }
    fn visit_atom(&mut self, name: &str, signal: &dyn Atom) {
        let module_path = self.path.to_string();
        let namespace = self.namespace.flat("$");
        let name = if namespace.is_empty() {
            name.to_owned()
        } else {
            format!("{}${}", namespace, name)
        };
        // Add an async source for all input parameters at the top scope
        let is_top_scope = self.path.to_string().eq("top");
        let global_signal_name = format!("{}${}", self.path.to_string(), name);
        match signal.kind() {
            AtomKind::InputParameter | AtomKind::InOutParameter => {
                if is_top_scope {
                    let my_id = self.graph.add_signal_node(&SignalNode {
                        name: global_signal_name.clone(),
                        kind: SignalNodeKind::Normal,
                    });
                    self.graph.add_signal_edge(
                        &SignalNode {
                            name: format!("extern${}", name),
                            kind: SignalNodeKind::Source,
                        },
                        my_id,
                        SignalEdgeKind::Extern,
                    );
                }
            }
            AtomKind::Constant => {
                let my_id = self.graph.add_signal_node(&SignalNode {
                    name: global_signal_name.clone(),
                    kind: SignalNodeKind::Normal,
                });
                self.graph.add_signal_edge(
                    &SignalNode {
                        name: format!("const${}", name),
                        kind: SignalNodeKind::Source,
                    },
                    my_id,
                    SignalEdgeKind::Constant,
                );
            }
            _ => {}
        }
        let descriptor = &signal.descriptor();
        if let TypeKind::Enum(x) = &descriptor.kind {
            for label in x {
                let label = label.replace("::", "$");
                let my_id = self.graph.add_signal_node(&SignalNode {
                    name: format!("{}${}", module_path, label),
                    kind: SignalNodeKind::Normal,
                });
                self.graph.add_signal_edge(
                    &SignalNode {
                        name: format!("const${}", label),
                        kind: SignalNodeKind::Source,
                    },
                    my_id,
                    SignalEdgeKind::Constant,
                );
                let my_id = self.graph.add_signal_node(&SignalNode {
                    name: format!("{}${}", self.path.parent(), label),
                    kind: SignalNodeKind::Normal,
                });
                self.graph.add_signal_edge(
                    &SignalNode {
                        name: format!("const${}", label),
                        kind: SignalNodeKind::Source,
                    },
                    my_id,
                    SignalEdgeKind::Constant,
                );
            }
        }
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
    println!("Graph is cyclic: {}", is_cyclic_directed(&scan.graph.graph));
    println!(
        "Number of connected components: {}",
        connected_components(&scan.graph.graph)
    );
    let g = &scan.graph.graph;
    let mut vertex_sets = UnionFind::new(g.node_count());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());
        vertex_sets.union(g.to_index(a), g.to_index(b));
    }
    let mut labels = vertex_sets.into_labeling();
    let mut unique_labels = labels.clone();
    unique_labels.sort_unstable();
    unique_labels.dedup();
    // For each label, we extract the vertices and build a new graph
    for subgraph in unique_labels {
        let mut remap: HashMap<_, _> = Default::default();
        let mut s: Graph<SignalNode, SignalEdgeKind> = Graph::default();
        for (ndx, label) in labels.iter().enumerate() {
            if *label == subgraph {
                let old_index = g.from_index(ndx);
                let old_weight = g[g.from_index(ndx)].clone();
                let new_index = s.add_node(old_weight);
                remap.insert(old_index, new_index);
            }
        }
        for edge in g.edge_references() {
            let (a, b) = (edge.source(), edge.target());
            assert!(!(remap.contains_key(&a) ^ remap.contains_key(&b)));
            if remap.contains_key(&a) {
                let a_new = remap.get(&a).unwrap();
                let b_new = remap.get(&b).unwrap();
                s.add_edge(*a_new, *b_new, *edge.weight());
            }
        }
        /*        println!("Number of elements in subgraph {}", remap.len());
        std::fs::write(
            format!("sub_graph_{}.dot", subgraph),
            format!("{:?}", Dot::new(&s)),
        )
        .unwrap();*/
    }
    //std::fs::write("dag.dot", dot).unwrap();
}
