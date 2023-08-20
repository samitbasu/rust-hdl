use std::marker::PhantomData;

use rust_hdl::prelude::freq_hz_to_period_femto;
use serde::{Deserialize, Serialize};

use crate::loggable::Loggable;

#[derive(Debug)]
pub struct TagID<T: Loggable> {
    pub context: usize,
    pub id: usize,
    pub _marker: PhantomData<*const T>,
}

impl<T: Loggable> Clone for TagID<T> {
    fn clone(&self) -> Self {
        Self {
            context: self.context,
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T: Loggable> Copy for TagID<T> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockDetails {
    pub name: String,
    pub period_in_fs: u64,
    pub offset_in_fs: u64,
    pub initial_state: bool,
}

impl ClockDetails {
    pub fn new(name: &str, period_in_fs: u64, offset_in_fs: u64, initial_state: bool) -> Self {
        Self {
            name: name.to_string(),
            period_in_fs,
            offset_in_fs,
            initial_state,
        }
    }
    pub fn pos_edge_at(&self, time: u64) -> bool {
        if time < self.offset_in_fs {
            return false;
        }
        let time = time - self.offset_in_fs;
        let period = self.period_in_fs;
        time % period == 0
    }
    pub fn neg_edge_at(&self, time: u64) -> bool {
        if time < self.offset_in_fs {
            return false;
        }
        let time = time - self.offset_in_fs;
        let period = self.period_in_fs;
        time % period == period / 2
    }
    pub fn next_edge_after(&self, time: u64) -> u64 {
        if time < self.offset_in_fs {
            return self.offset_in_fs;
        }
        let time = time - self.offset_in_fs;
        let period = self.period_in_fs / 2;
        (time / period + 1) * period + self.offset_in_fs
    }
}

#[test]
fn test_clock_details() {
    let clock = ClockDetails::new("clk", 10, 0, false);
    assert!(clock.pos_edge_at(0));
    assert!(!clock.pos_edge_at(5));
    assert!(clock.pos_edge_at(10));
    assert!(!clock.pos_edge_at(15));
    assert!(!clock.neg_edge_at(0));
    assert!(clock.neg_edge_at(5));
    assert!(!clock.neg_edge_at(10));
    assert_eq!(clock.next_edge_after(1), 5);
    assert_eq!(clock.next_edge_after(0), 5);
    assert_eq!(clock.next_edge_after(5), 10);
}

pub trait LogBuilder {
    type SubBuilder: LogBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder;
    fn tag<T: Loggable>(&mut self, name: &str) -> TagID<T>;
    fn allocate<L: Loggable>(&self, tag: TagID<L>, width: usize);
    fn namespace(&self, name: &str) -> Self::SubBuilder;
    fn add_clock(&mut self, clock: ClockDetails);
    fn add_simple_clock(&mut self, period_in_fs: u64) {
        self.add_clock(ClockDetails {
            name: "clock".to_string(),
            period_in_fs,
            offset_in_fs: 0,
            initial_state: false,
        });
    }
}

impl<T: LogBuilder> LogBuilder for &mut T {
    type SubBuilder = T::SubBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder {
        (**self).scope(name)
    }
    fn tag<L: Loggable>(&mut self, name: &str) -> TagID<L> {
        (**self).tag(name)
    }
    fn allocate<L: Loggable>(&self, tag: TagID<L>, width: usize) {
        (**self).allocate(tag, width)
    }
    fn namespace(&self, name: &str) -> Self::SubBuilder {
        (**self).namespace(name)
    }
    fn add_clock(&mut self, clock: ClockDetails) {
        (**self).add_clock(clock)
    }
}
