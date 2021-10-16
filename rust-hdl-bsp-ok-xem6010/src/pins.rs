use rust_hdl_core::bits::Bits;
use rust_hdl_core::clock::Clock;
use rust_hdl_core::constraint::{Constraint, PeriodicTiming, PinConstraint, SignalType, Timing};
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::signal::Signal;

pub fn xem_6010_leds() -> Signal<Out, Bits<8>> {
    let mut x = Signal::default();
    for (ndx, name) in [
        "Y17", "AB17", "AA14", "AB14", "AA16", "AB16", "AA10", "AB10",
    ]
    .iter()
    .enumerate()
    {
        x.add_location(ndx, name);
        x.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
    }
    x
}

pub fn xem_6010_base_clock() -> Signal<In, Clock> {
    let mut x = Signal::default();
    x.add_location(0, "AB13");
    x.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
    x.add_constraint(PinConstraint {
        index: 0,
        constraint: Constraint::Timing(Timing::Periodic(PeriodicTiming {
            net: "SystemClk".into(),
            period_nanoseconds: 10.0,
            duty_cycle: 50.0,
        })),
    });
    x
}
