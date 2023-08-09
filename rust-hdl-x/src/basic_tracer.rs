use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use crate::{
    tracer::{TraceID, TraceTag, TraceType, Tracer},
    TracerBuilder,
};

#[derive(Debug, Clone)]
enum TraceValues {
    Short(Vec<u64>),
    Long(Vec<Vec<bool>>),
    Enum(Vec<&'static str>),
}

impl TraceValues {
    fn len(&self) -> usize {
        match self {
            TraceValues::Short(v) => v.len(),
            TraceValues::Long(v) => v.len(),
            TraceValues::Enum(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone)]
struct TraceSignal {
    name: String,
    width: usize,
    values: TraceValues,
}

impl TraceSignal {
    fn new(name: &str, width: usize) -> TraceSignal {
        TraceSignal {
            name: name.to_string(),
            width,
            values: if width == 0 {
                TraceValues::Enum(vec![])
            } else if width <= 64 {
                TraceValues::Short(vec![])
            } else {
                TraceValues::Long(vec![])
            },
        }
    }
}

#[derive(Debug, Clone)]
struct ScopeRecord {
    name: String,
    inputs: Vec<TraceSignal>,
    outputs: Vec<TraceSignal>,
    state_q: Vec<TraceSignal>,
    state_d: Vec<TraceSignal>,
}

impl Display for ScopeRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for input in &self.inputs {
            writeln!(
                f,
                "{}::input::{} [{}] --> {}",
                self.name,
                input.name,
                input.width,
                input.values.len()
            )?;
        }
        for output in &self.outputs {
            writeln!(
                f,
                "{}::output::{} [{}] --> {}",
                self.name,
                output.name,
                output.width,
                output.values.len()
            )?;
        }
        for state_q in &self.state_q {
            writeln!(
                f,
                "{}::state_q::{} [{}] --> {}",
                self.name,
                state_q.name,
                state_q.width,
                state_q.values.len()
            )?;
        }
        for state_d in &self.state_d {
            writeln!(
                f,
                "{}::state_d::{} [{}] --> {}",
                self.name,
                state_d.name,
                state_d.width,
                state_d.values.len()
            )?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct BasicTracer {
    scopes: Vec<ScopeRecord>,
    scope_index: usize,
    field_index: usize,
    tag: TraceTag,
}

impl BasicTracer {
    fn signal(&mut self) -> &mut TraceSignal {
        let scope = &mut self.scopes[self.scope_index];
        match self.tag {
            TraceTag::Input => &mut scope.inputs[self.field_index],
            TraceTag::Output => &mut scope.outputs[self.field_index],
            TraceTag::StateD => &mut scope.state_d[self.field_index],
            TraceTag::StateQ => &mut scope.state_q[self.field_index],
        }
    }
}

impl Tracer for BasicTracer {
    fn write_bool(&mut self, value: bool) {
        if let TraceValues::Short(ref mut values) = self.signal().values {
            values.push(value as u64);
        } else {
            panic!("Wrong type");
        }
    }
    fn write_small(&mut self, value: u64) {
        if let TraceValues::Short(ref mut values) = self.signal().values {
            values.push(value);
        } else {
            panic!("Wrong type");
        }
    }
    fn write_large(&mut self, val: &[bool]) {
        if let TraceValues::Long(ref mut values) = self.signal().values {
            values.push(val.to_vec());
        } else {
            panic!("Wrong type");
        }
    }
    fn write_string(&mut self, val: &'static str) {
        if let TraceValues::Enum(ref mut values) = self.signal().values {
            values.push(val);
        } else {
            panic!("Wrong type");
        }
    }

    fn set_context(&mut self, id: TraceID) {
        self.scope_index = id.into();
    }

    fn set_tag(&mut self, tag: TraceTag) {
        self.field_index = 0;
        self.tag = tag;
    }
}

impl Display for BasicTracer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.scopes[self.scope_index].fmt(f)
    }
}

// I don't like the use of interior mutability here.
// I need to redesign the API so it is not required.
#[derive(Clone, Debug)]
pub struct BasicTracerBuilder {
    scopes: Rc<RefCell<Vec<ScopeRecord>>>,
    current_scope: usize,
    current_kind: Option<TraceType>,
    path: Vec<String>,
}

impl Display for BasicTracerBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for scope in self.scopes.borrow().iter() {
            writeln!(f, "{}", scope)?;
        }
        Ok(())
    }
}

impl Default for BasicTracerBuilder {
    fn default() -> Self {
        Self {
            scopes: Rc::new(RefCell::new(vec![ScopeRecord {
                name: "root".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                state_q: Vec::new(),
                state_d: Vec::new(),
            }])),
            current_scope: 0,
            current_kind: None,
            path: vec![],
        }
    }
}

impl TracerBuilder for BasicTracerBuilder {
    type SubBuilder = Self;
    fn scope(&self, name: &str) -> Self {
        let name = format!(
            "{}::{}",
            self.scopes.borrow()[self.current_scope].name,
            name
        );
        self.scopes.borrow_mut().push(ScopeRecord {
            name,
            inputs: Vec::new(),
            outputs: Vec::new(),
            state_q: Vec::new(),
            state_d: Vec::new(),
        });
        Self {
            scopes: self.scopes.clone(),
            current_scope: self.scopes.borrow().len() - 1,
            current_kind: None,
            path: vec![],
        }
    }

    fn trace_id(&self) -> TraceID {
        self.current_scope.into()
    }

    fn set_kind(&mut self, kind: TraceType) {
        self.current_kind = Some(kind);
    }

    fn register(&self, width: usize) {
        let name = self.path.join("::");
        let signal = TraceSignal::new(&name, width);
        let kind = self.current_kind.unwrap();
        match kind {
            TraceType::Input => {
                self.scopes.borrow_mut()[self.current_scope]
                    .inputs
                    .push(signal);
            }
            TraceType::Output => {
                self.scopes.borrow_mut()[self.current_scope]
                    .outputs
                    .push(signal);
            }
            TraceType::State => {
                self.scopes.borrow_mut()[self.current_scope]
                    .state_q
                    .push(signal.clone());
                self.scopes.borrow_mut()[self.current_scope]
                    .state_d
                    .push(signal);
            }
        }
    }

    fn namespace(&self, name: &str) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(name.to_string());
        Self {
            scopes: self.scopes.clone(),
            current_scope: self.current_scope,
            current_kind: self.current_kind,
            path: new_path,
        }
    }
}

impl BasicTracerBuilder {
    pub fn build(self) -> BasicTracer {
        BasicTracer {
            scopes: self.scopes.take(),
            scope_index: 0,
            field_index: 0,
            tag: TraceTag::Input,
        }
    }
}
