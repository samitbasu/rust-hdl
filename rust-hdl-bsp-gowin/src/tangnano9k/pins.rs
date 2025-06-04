use rust_hdl::core::prelude::*;

pub const CLOCK_SPEED_27MHZ: u64 = 27_000_000;

pub fn clock() -> Signal<In, Clock> {
    let mut signal = Signal::default();
    signal.add_location(0, "52");
    // signal.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
    signal.connect();
    signal
}

pub fn lamp() -> Signal<Out, Bits<6>> {
    let mut signal = Signal::default();
    let locs = ["10", "11", "13", "14", "15", "16"];
    for (i, loc) in locs.iter().enumerate() {
        signal.add_location(i, loc);
        // signal.add_signal_type(i, SignalType::LowVoltageCMOS_3v3);
    }
    signal
}

pub fn button() -> Signal<In, Bits<2>> {
    let mut signal = Signal::default();
    let locs = ["3", "4"];
    for (i, loc) in locs.iter().enumerate() {
        signal.add_location(i, loc);
        // signal.add_signal_type(i, SignalType::LowVoltageCMOS_3v3);
    }
    signal
}
