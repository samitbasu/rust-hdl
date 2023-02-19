use std::fmt::Debug;

/// Fundamentally a Clock signal in RustHDL is simply a transparent wrapper around a boolean
/// valued signal.  So it could be thought of as a simple 1-bit wide signal.  However, semantically,
/// clocks are rarely treated like other signals, and typically connect only to dedicated clock
/// ports on synchronous logic (like [DFF] or [RAM]).
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Clock {
    /// The clock signal itself.  Available using this field of the struct.
    pub clk: bool,
}

/// The number of nanoseconds per femtosecond.
pub const NANOS_PER_FEMTO: f64 = 1_000_000.0;

/// Convert a frequency in Hz to a period in femtoseconds.
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

/// The [clock!] macro is used to connect a set of devices to a common clock.
/// The macro takes a variable number of arguments:
///  * `self` - the struct containing the items to connect, normally just `self`
///  * `clock` - the name of the field of the struct that holds the clock source
///  * `subs` - the set of fields that should be clocked with `clock`.
/// Note that the macro assumes that the sub-circuits being clocked all have clock inputs
/// named `clock`.
///
/// For example:
/// ```
/// use rust_hdl_private_core::prelude::*;
///
/// #[derive(LogicBlock)]
/// pub struct Widget {
///    pub clock: Signal<In, Clock>,
///    pub dff_1: DFF<Bit>,
///    pub dff_2: DFF<Bit>,
/// }
///
/// impl Logic for Widget {
///    #[hdl_gen]
///    fn update(&mut self) {
///        // This is equivalent to:
///        // self.dff_1.clock.next = self.clock.val();
///        // self.dff_2.clock.next = self.clock.val();
///        clock!(self, clock, dff_1, dff_2);
///    }
/// }
/// ```
///
/// When you have many [DFF]s or several complex sub-circuits, the `clock!` macro can make it
/// easier to read the source code.
#[macro_export]
macro_rules! clock {
    ($self: ident, $clock: ident, $($subs: ident), +) => {
        $($self.$subs.clock.next = $self.$clock.val());+;
    }
}
