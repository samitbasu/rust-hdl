use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use crate::{
    log::{LogBuilder, TagID},
    loggable::Loggable,
    logger::Logger,
    synchronous::Synchronous,
};

#[derive(Debug, Clone)]
enum LogValues {
    Short(Vec<u64>),
    Long(Vec<Vec<bool>>),
    Enum(Vec<&'static str>),
}

impl LogValues {
    fn len(&self) -> usize {
        match self {
            LogValues::Short(v) => v.len(),
            LogValues::Long(v) => v.len(),
            LogValues::Enum(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone)]
struct LogSignal {
    name: String,
    width: usize,
    values: LogValues,
}

impl LogSignal {
    fn new(name: &str, width: usize) -> LogSignal {
        LogSignal {
            name: name.to_string(),
            width,
            values: if width == 0 {
                LogValues::Enum(vec![])
            } else if width <= 64 {
                LogValues::Short(vec![])
            } else {
                LogValues::Long(vec![])
            },
        }
    }
}

#[derive(Debug, Clone)]
struct TaggedSignal {
    tag: String,
    data: Vec<LogSignal>,
}

impl Display for TaggedSignal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for signal in &self.data {
            write!(
                f,
                "{}::{} [{}] --> {}",
                self.tag,
                signal.name,
                signal.width,
                signal.values.len()
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ScopeRecord {
    name: String,
    tags: Vec<TaggedSignal>,
}

impl Display for ScopeRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for tag in &self.tags {
            write!(f, "{}::{}", self.name, tag)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct BasicLogger {
    scopes: Vec<ScopeRecord>,
    field_index: usize,
}

impl BasicLogger {
    fn signal<L: Loggable>(&mut self, tag_id: TagID<L>) -> &mut LogSignal {
        let scope = &mut self.scopes[tag_id.context];
        let tag = &mut scope.tags[tag_id.id];
        &mut tag.data[self.field_index]
    }
}

impl Logger for BasicLogger {
    fn write_bool<L: Loggable>(&mut self, tag_id: TagID<L>, value: bool) {
        if let LogValues::Short(ref mut values) = self.signal(tag_id).values {
            values.push(value as u64);
        } else {
            panic!("Wrong type");
        }
    }
    fn write_small<L: Loggable>(&mut self, tag_id: TagID<L>, value: u64) {
        if let LogValues::Short(ref mut values) = self.signal(tag_id).values {
            values.push(value);
        } else {
            panic!("Wrong type");
        }
    }
    fn write_large<L: Loggable>(&mut self, tag_id: TagID<L>, val: &[bool]) {
        if let LogValues::Long(ref mut values) = self.signal(tag_id).values {
            values.push(val.to_vec());
        } else {
            panic!("Wrong type");
        }
    }
    fn write_string<L: Loggable>(&mut self, tag_id: TagID<L>, val: &'static str) {
        if let LogValues::Enum(ref mut values) = self.signal(tag_id).values {
            values.push(val);
        } else {
            panic!("Wrong type");
        }
    }
}

// I don't like the use of interior mutability here.
// I need to redesign the API so it is not required.
#[derive(Clone, Debug)]
pub struct BasicLoggerBuilder {
    scopes: Rc<RefCell<Vec<ScopeRecord>>>,
    path: Vec<String>,
}

impl Display for BasicLoggerBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for scope in self.scopes.borrow().iter() {
            writeln!(f, "{}", scope)?;
        }
        Ok(())
    }
}

impl Default for BasicLoggerBuilder {
    fn default() -> Self {
        Self {
            scopes: Rc::new(RefCell::new(vec![ScopeRecord {
                name: "root".to_string(),
                tags: Vec::new(),
            }])),
            path: vec![],
        }
    }
}

impl LogBuilder for BasicLoggerBuilder {
    type SubBuilder = Self;
    fn scope(&self, name: &str) -> Self {
        let name = format!("{}::{}", self.scopes.borrow().last().unwrap().name, name);
        self.scopes.borrow_mut().push(ScopeRecord {
            name,
            tags: Vec::new(),
        });
        Self {
            scopes: self.scopes.clone(),
            path: vec![],
        }
    }

    fn tag<L: Loggable>(&mut self, name: &str) -> TagID<L> {
        let context_id: usize = self.scopes.borrow().len() - 1;
        let tag = {
            let scope = &mut self.scopes.borrow_mut()[context_id];
            scope.tags.push(TaggedSignal {
                tag: name.to_string(),
                data: Vec::new(),
            });
            TagID {
                context: context_id,
                id: scope.tags.len() - 1,
                _marker: Default::default(),
            }
        };
        L::allocate(tag, self);
        tag
    }

    fn allocate<L: Loggable>(&self, tag: TagID<L>, width: usize) {
        let name = self.path.join("::");
        let signal = LogSignal::new(&name, width);
        let context_id: usize = tag.context;
        let scope = &mut self.scopes.borrow_mut()[context_id];
        let tag_id: usize = tag.id;
        let tag = &mut scope.tags[tag_id];
        tag.data.push(signal);
    }

    fn namespace(&self, name: &str) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(name.to_string());
        Self {
            scopes: self.scopes.clone(),
            path: new_path,
        }
    }
}

impl BasicLoggerBuilder {
    pub fn build(self) -> BasicLogger {
        BasicLogger {
            scopes: self.scopes.take(),
            field_index: 0,
        }
    }
}
