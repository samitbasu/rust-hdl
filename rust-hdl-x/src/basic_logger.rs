use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use crate::{
    log::{ClockDetails, LogBuilder, TagID},
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
pub(crate) struct ScopeRecord {
    pub(crate) name: String,
    pub(crate) tags: Vec<TaggedSignal>,
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
    pub(crate) scopes: Vec<ScopeRecord>,
    pub(crate) clocks: Vec<ClockDetails>,
    pub(crate) field_index: usize,
    pub(crate) time_in_fs: u64,
}

impl BasicLogger {
    fn signal<L: Loggable>(&mut self, tag_id: TagID<L>) -> &mut LogSignal {
        let scope = &mut self.scopes[tag_id.context];
        let tag = &mut scope.tags[tag_id.id];
        &mut tag.data[self.field_index]
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
