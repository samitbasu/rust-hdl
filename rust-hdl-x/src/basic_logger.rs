use std::{
    fmt::{Display, Formatter},
    io::Write,
};

use indexmap::IndexMap;

use crate::{
    log::{ClockDetails, TagID},
    loggable::Loggable,
    logger::Logger,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimedValue<T: Clone + PartialEq + Eq> {
    pub(crate) time_in_fs: u64,
    pub(crate) value: T,
}

#[derive(Debug, Clone)]
pub(crate) enum LogValues {
    Bool(Vec<TimedValue<bool>>),
    Short(Vec<TimedValue<u64>>),
    Long(Vec<TimedValue<Vec<bool>>>),
    Enum(Vec<TimedValue<&'static str>>),
}

impl LogValues {
    pub(crate) fn len(&self) -> usize {
        match self {
            LogValues::Bool(v) => v.len(),
            LogValues::Short(v) => v.len(),
            LogValues::Long(v) => v.len(),
            LogValues::Enum(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LogSignal {
    pub(crate) name: String,
    pub(crate) width: usize,
    pub(crate) values: LogValues,
}

impl LogSignal {
    pub(crate) fn new(name: &str, width: usize) -> LogSignal {
        LogSignal {
            name: name.to_string(),
            width,
            values: if width == 0 {
                LogValues::Enum(vec![])
            } else if width == 1 {
                LogValues::Bool(vec![])
            } else if width <= 64 {
                LogValues::Short(vec![])
            } else {
                LogValues::Long(vec![])
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TaggedSignal {
    pub(crate) tag: String,
    pub(crate) data: Vec<LogSignal>,
}

impl Display for TaggedSignal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for signal in &self.data {
            writeln!(
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

enum ScopeNode<'a> {
    Internal {
        children: IndexMap<String, ScopeNode<'a>>,
    },
    Leaf {
        width: usize,
        code: Option<vcd::IdCode>,
        signal: &'a LogSignal,
    },
}

impl<'a> ScopeNode<'a> {
    fn new_scope() -> Self {
        ScopeNode::Internal {
            children: IndexMap::new(),
        }
    }
    fn children(&mut self) -> &mut IndexMap<String, ScopeNode<'a>> {
        match self {
            ScopeNode::Internal { children } => children,
            ScopeNode::Leaf { .. } => panic!("Leaf node"),
        }
    }
    fn children_at(&mut self, path: &[&str]) -> &mut IndexMap<String, ScopeNode<'a>> {
        if let Some((&first, rest)) = path.split_first() {
            self.children()
                .entry(first.to_owned())
                .or_insert_with(ScopeNode::new_scope)
                .children_at(rest)
        } else {
            self.children()
        }
    }
}

fn build_scope_tree(scopes: &[ScopeRecord]) -> ScopeNode {
    let mut root = ScopeNode::new_scope();
    for scope in scopes {
        println!("scope name: {}", scope.name);
        let path: Vec<_> = scope.name.split("::").collect();
        for tag in &scope.tags {
            // There are two possibilities for tags.
            // One is a tag that stores a struct, in which case,
            // there are named elements beneath the tag.  In
            // the other case, the tag just holds a single data element.
            // We treat these differently - in the first case, we
            // treat the tag as a scope.  In the second, we treat it as a signal.
            if tag.data.len() == 1 {
                let signal = &tag.data[0];
                root.children_at(&path)
                    .entry(tag.tag.clone())
                    .or_insert_with(|| ScopeNode::Leaf {
                        width: signal.width,
                        code: None,
                        signal,
                    });
            } else {
                println!("Structured tag {}", tag.tag);
                let tag_root = root
                    .children_at(&path)
                    .entry(tag.tag.clone())
                    .or_insert_with(ScopeNode::new_scope);
                for signal in &tag.data {
                    println!("signal name: {}", signal.name);
                    let sub_path: Vec<_> = signal.name.split("::").collect();
                    if let Some((item, path)) = sub_path.split_last() {
                        tag_root
                            .children_at(path)
                            .entry(item.to_string())
                            .or_insert_with(|| ScopeNode::Leaf {
                                width: signal.width,
                                code: None,
                                signal,
                            });
                    }
                }
            }
        }
    }
    root
}

impl<'a> ScopeNode<'a> {
    fn dump(&self, indent_level: usize) {
        match self {
            ScopeNode::Internal { children } => {
                for (name, child) in children {
                    println!("{}{}", "  ".repeat(indent_level), name);
                    child.dump(indent_level + 1);
                }
            }
            ScopeNode::Leaf {
                width,
                code,
                signal,
            } => {
                println!("{}[{}] {:?}", "  ".repeat(indent_level), width, code);
            }
        }
    }
    fn register<W: Write>(&mut self, name: &str, v: &mut vcd::Writer<W>) {
        match self {
            ScopeNode::Internal { children } => {
                for (name, child) in children {
                    v.add_module(name);
                    child.register(name.as_str(), v);
                    v.upscope();
                }
            }
            ScopeNode::Leaf {
                width,
                code,
                signal: _,
            } => *code = Some(v.add_wire(*width as u32, name).unwrap()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScopeRecord {
    pub(crate) name: String,
    pub(crate) tags: Vec<TaggedSignal>,
}

impl Display for ScopeRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for tag in &self.tags {
            for signal in &tag.data {
                writeln!(
                    f,
                    "<{}>::{}::{} [{}] --> {}",
                    self.name,
                    tag.tag,
                    signal.name,
                    signal.width,
                    signal.values.len()
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct BasicLogger {
    pub(crate) scopes: Vec<ScopeRecord>,
    pub(crate) clocks: Vec<ClockDetails>,
    pub(crate) field_index: usize,
    pub(crate) time_in_fs: u64,
}

impl Display for BasicLogger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for scope in &self.scopes {
            writeln!(f, "{}", scope)?;
        }
        Ok(())
    }
}

// Uses the names of the scopes (which are seperated by ::) to
// build a tree of scope names for writing to the VCD file
// in a hierarchical manner.

impl BasicLogger {
    fn signal<L: Loggable>(&mut self, tag_id: TagID<L>) -> &mut LogSignal {
        let scope = &mut self.scopes[tag_id.context];
        let tag = &mut scope.tags[tag_id.id];
        &mut tag.data[self.field_index]
    }
    pub fn vcd<W: Write>(self, w: W) -> anyhow::Result<()> {
        let mut writer = vcd::Writer::new(w);
        writer.timescale(1, vcd::TimescaleUnit::FS)?;
        let mut tree = build_scope_tree(&self.scopes);
        tree.register("", &mut writer);
        writer.enddefinitions()?;
        writer.timestamp(0)?;
        writer.timestamp(1)?;
        Ok(())
    }
    pub(crate) fn dump(&self) {
        let tree = build_scope_tree(&self.scopes);
        tree.dump(0);
    }
}

impl Logger for BasicLogger {
    fn set_time_in_fs(&mut self, time: u64) {
        self.time_in_fs = time;
    }
    fn write_bool<L: Loggable>(&mut self, tag_id: TagID<L>, value: bool) {
        let time_in_fs = self.time_in_fs;
        if let LogValues::Bool(ref mut values) = self.signal(tag_id).values {
            values.push(TimedValue { time_in_fs, value });
        } else {
            panic!("Wrong type");
        }
    }
    fn write_small<L: Loggable>(&mut self, tag_id: TagID<L>, value: u64) {
        let time_in_fs = self.time_in_fs;
        if let LogValues::Short(ref mut values) = self.signal(tag_id).values {
            values.push(TimedValue { time_in_fs, value });
        } else {
            panic!("Wrong type");
        }
    }
    fn write_large<L: Loggable>(&mut self, tag_id: TagID<L>, val: &[bool]) {
        let time_in_fs = self.time_in_fs;
        if let LogValues::Long(ref mut values) = self.signal(tag_id).values {
            values.push(TimedValue {
                time_in_fs,
                value: val.to_vec(),
            });
        } else {
            panic!("Wrong type");
        }
    }
    fn write_string<L: Loggable>(&mut self, tag_id: TagID<L>, val: &'static str) {
        let time_in_fs = self.time_in_fs;
        if let LogValues::Enum(ref mut values) = self.signal(tag_id).values {
            values.push(TimedValue {
                time_in_fs,
                value: val,
            });
        } else {
            panic!("Wrong type");
        }
    }
}
