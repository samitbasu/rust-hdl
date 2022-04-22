use std::fmt::Debug;

// We don't want clock types to be multibit or other weird things...
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Clock {
    pub clk: bool,
}

pub const NANOS_PER_FEMTO: f64 = 1_000_000.0;

pub fn freq_hz_to_period_femto(freq: f64) -> f64 {
    (1.0e15 / freq).round()
}

#[test]
fn test_freq_to_period_mapping() {
    assert_eq!(freq_hz_to_period_femto(1.0), 1.0e15)
}

impl std::ops::Not for Clock {
    type Output = Clock;

    fn not(self) -> Self::Output {
        Clock { clk: !self.clk }
    }
}

impl From<bool> for Clock {
    fn from(x: bool) -> Clock {
        Clock { clk: x }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Reset {
    pub rst: bool,
}

pub const NO_RESET: Reset = Reset { rst: false };
pub const RESET: Reset = Reset { rst: true };

impl Into<bool> for Reset {
    fn into(self) -> bool {
        self.rst
    }
}

/*
impl From<bool> for Reset {
    fn from(x: bool) -> Reset {
        Reset { rst: x }
    }
}
*/

impl std::ops::Not for Reset {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.rst
    }
}

#[macro_export]
macro_rules! clock_reset {
    ($self: ident, $clock: ident, $reset: ident, $($subs: ident), +) => {
        $($self.$subs.clock.next = $self.$clock.val());+;
        $($self.$subs.reset.next = $self.$reset.val());+;
    }
}
