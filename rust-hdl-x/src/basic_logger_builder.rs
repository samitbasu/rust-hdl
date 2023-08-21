use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use rust_hdl::prelude::freq_hz_to_period_femto;

use crate::{
    basic_logger::{BasicLogger, LogSignal, ScopeRecord, TaggedSignal},
    log::{ClockDetails, LogBuilder, TagID},
    loggable::Loggable,
};

#[derive(Clone, Debug, Default)]
struct BasicLoggerBuilderInner {
    scopes: Vec<ScopeRecord<'static>>,
    clocks: Vec<ClockDetails>,
}

// I don't like the use of interior mutability here.
// I need to redesign the API so it is not required.
#[derive(Clone, Debug)]
pub struct BasicLoggerBuilder {
    inner: Rc<RefCell<BasicLoggerBuilderInner>>,
    path: Vec<String>,
    my_scope: usize,
}

impl Display for BasicLoggerBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for scope in self.inner.borrow().scopes.iter() {
            writeln!(f, "{}", scope)?;
        }
        Ok(())
    }
}

impl Default for BasicLoggerBuilder {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(BasicLoggerBuilderInner {
                scopes: vec![ScopeRecord {
                    name: "root".to_string(),
                    tags: Vec::new(),
                }],
                ..Default::default()
            })),
            path: vec![],
            my_scope: 0,
        }
    }
}

impl LogBuilder for BasicLoggerBuilder {
    type SubBuilder = Self;
    fn scope(&self, name: &str) -> Self {
        let name = format!(
            "{}::{}",
            self.inner.borrow().scopes[self.my_scope].name,
            name
        );
        self.inner.borrow_mut().scopes.push(ScopeRecord {
            name,
            tags: Vec::new(),
        });
        Self {
            inner: self.inner.clone(),
            path: vec![],
            my_scope: self.inner.borrow().scopes.len() - 1,
        }
    }

    fn tag<L: Loggable>(&mut self, name: &str) -> TagID<L> {
        let context_id: usize = self.my_scope;
        let tag = {
            let scope = &mut self.inner.borrow_mut().scopes[context_id];
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
        println!("Allocating tag: context: {}, id: {}", context_id, tag.id);
        L::allocate(tag, self);
        tag
    }

    fn allocate<L: Loggable>(&self, tag: TagID<L>, width: usize) {
        let name = self.path.join("$");
        let signal = LogSignal::new(name, width);
        let context_id: usize = tag.context;
        let scope = &mut self.inner.borrow_mut().scopes[context_id];
        let tag_id: usize = tag.id;
        let tag = &mut scope.tags[tag_id];
        tag.data.push(signal);
    }

    fn namespace(&self, name: &str) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(name.to_string());
        Self {
            inner: self.inner.clone(),
            path: new_path,
            my_scope: self.my_scope,
        }
    }

    fn add_clock(&mut self, clock: ClockDetails) {
        self.inner.borrow_mut().clocks.push(clock);
    }
}

impl BasicLoggerBuilder {
    pub fn build(self) -> BasicLogger<'static> {
        let inner = self.inner.take();
        BasicLogger {
            scopes: inner.scopes,
            clocks: inner.clocks,
            field_index: 0,
            time_in_fs: 0,
        }
    }
}
