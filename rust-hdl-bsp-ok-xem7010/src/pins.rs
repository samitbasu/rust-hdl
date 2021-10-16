use rust_hdl_core::bits::Bits;
use rust_hdl_core::clock::Clock;
use rust_hdl_core::constraint::{Constraint, PeriodicTiming, PinConstraint, SignalType, Timing};
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::signal::Signal;

pub fn xem_7010_leds() -> Signal<Out, Bits<8>> {
    let mut x = Signal::default();
    for (ndx, name) in ["N13", "N14", "P15", "P16", "N17", "P17", "R16", "R17"]
        .iter()
        .enumerate()
    {
        x.add_location(ndx, name);
        x.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
    }
    x
}

pub fn xem_7010_pos_clock() -> Signal<In, Clock> {
    let mut x = Signal::default();
    x.add_location(0, "K4");
    x.add_signal_type(0, SignalType::LowVoltageDifferentialSignal_2v5);
    x.connect();
    x.add_constraint(PinConstraint {
        index: 0,
        constraint: Constraint::Timing(Timing::Periodic(PeriodicTiming {
            net: "SystemClk".into(),
            period_nanoseconds: 5.0,
            duty_cycle: 50.0,
        })),
    });
    x
}

pub fn xem_7010_neg_clock() -> Signal<In, Clock> {
    let mut x = Signal::default();
    x.add_location(0, "J4");
    x.add_signal_type(0, SignalType::LowVoltageDifferentialSignal_2v5);
    x.connect();
    x
}
