use rust_hdl_core::prelude::*;

use crate::MHz48;

#[derive(Clone, Debug, LogicInterface)]
pub struct OpalKellyHostInterface {
    pub sig_in: Signal<In, Bits<8>, MHz48>,
    pub sig_out: Signal<Out, Bits<2>, MHz48>,
    pub sig_inout: Signal<InOut, Bits<16>, MHz48>,
    pub sig_aa: Signal<InOut, Bit, MHz48>,
}

impl OpalKellyHostInterface {
    pub fn xem_6010() -> OpalKellyHostInterface {
        let mut hi_in = Signal::default();
        for (ndx, name) in ["Y12", "AB20", "AB7", "AB8", "AA4", "AB4", "Y3", "AB3"]
            .iter()
            .enumerate()
        {
            hi_in.add_location(ndx, name);
            hi_in.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
        }
        hi_in.add_constraint(PinConstraint {
            index: 0,
            constraint: Constraint::Timing(Periodic(PeriodicTiming {
                net: "okHostClk".into(),
                period_nanoseconds: 20.83,
                duty_cycle: 50.0,
            })),
        });
        let mut hi_out = Signal::default();
        for (ndx, name) in ["Y19", "AA8"].iter().enumerate() {
            hi_out.add_location(ndx, name);
            hi_out.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
        }
        let mut hi_inout = Signal::default();
        for (ndx, name) in [
            "AB12", "AA12", "Y13", "AB18", "AA18", "V15", "AB2", "AA2", "Y7", "Y4", "W4", "AB6",
            "AA6", "U13", "U14", "AA20",
        ]
        .iter()
        .enumerate()
        {
            hi_inout.add_location(ndx, name);
            hi_inout.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
        }
        let mut hi_aa = Signal::default();
        hi_aa.add_location(0, "W11");
        hi_aa.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        Self {
            sig_in: hi_in,
            sig_out: hi_out,
            sig_inout: hi_inout,
            sig_aa: hi_aa,
        }
    }
}

