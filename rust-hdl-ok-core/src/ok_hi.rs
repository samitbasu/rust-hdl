use rust_hdl_core::constraint::InputTimingConstraint;
use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, LogicInterface)]
pub struct OpalKellyHostInterface {
    pub sig_in: Signal<In, Bits<8>>,
    pub sig_out: Signal<Out, Bits<2>>,
    pub sig_inout: Signal<InOut, Bits<16>>,
    pub sig_aa: Signal<InOut, Bit>,
    pub sig_mux: Signal<Out, Bit>,
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
            if ndx != 0 {
                hi_in.add_constraint(PinConstraint {
                    index: ndx,
                    constraint: Constraint::Timing(Timing::InputTiming(InputTimingConstraint {
                        offset_nanoseconds: 14.3,
                        valid_duration_nanoseconds: 20.83,
                        relative: TimingRelative::Before,
                        edge_sense: TimingRelativeEdge::Rising,
                        to_signal_id: hi_in.id(),
                        to_signal_bit: Some(0),
                    })),
                })
            } else {
                hi_in.add_constraint(PinConstraint {
                    index: 0,
                    constraint: Constraint::Timing(Periodic(PeriodicTiming {
                        net: "okHostClk".into(),
                        period_nanoseconds: 20.83,
                        duty_cycle: 50.0,
                    })),
                });
            }
        }
        let mut hi_out = Signal::default();
        for (ndx, name) in ["Y19", "AA8"].iter().enumerate() {
            hi_out.add_location(ndx, name);
            hi_out.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
            hi_out.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Timing(Timing::OutputTiming(OutputTimingConstraint {
                    offset_nanoseconds: 11.93,
                    relative: TimingRelative::After,
                    edge_sense: TimingRelativeEdge::Rising,
                    to_signal_id: hi_in.id(),
                    to_signal_bit: Some(0),
                })),
            })
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
            hi_inout.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Timing(Timing::OutputTiming(OutputTimingConstraint {
                    offset_nanoseconds: 11.63,
                    relative: TimingRelative::After,
                    edge_sense: TimingRelativeEdge::Rising,
                    to_signal_id: hi_in.id(),
                    to_signal_bit: Some(0),
                })),
            });
            hi_inout.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Timing(Timing::InputTiming(InputTimingConstraint {
                    offset_nanoseconds: 9.83,
                    valid_duration_nanoseconds: 9.83,
                    relative: TimingRelative::Before,
                    edge_sense: TimingRelativeEdge::Rising,
                    to_signal_id: hi_in.id(),
                    to_signal_bit: Some(0),
                })),
            })
        }
        let mut hi_aa = Signal::default();
        hi_aa.add_location(0, "W11");
        hi_aa.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        let mut hi_mux = Signal::default();
        hi_mux.add_location(0, "AA22");
        hi_mux.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        Self {
            sig_in: hi_in,
            sig_out: hi_out,
            sig_inout: hi_inout,
            sig_aa: hi_aa,
            sig_mux: hi_mux,
        }
    }

    pub fn xem_7010() -> OpalKellyHostInterface {
        let mut hi_in = Signal::default();
        for (ndx, name) in ["Y18", "V17", "AA19", "V20", "W17", "AB20", "V19", "AA18"]
            .iter()
            .enumerate()
        {
            hi_in.add_location(ndx, name);
            hi_in.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
            hi_in.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Slew(SlewType::Fast),
            });
            if ndx != 0 {
                hi_in.add_constraint(PinConstraint {
                    index: ndx,
                    constraint: Constraint::Timing(Timing::VivadoInputTiming(
                        VivadoInputTimingConstraint {
                            min_nanoseconds: 0.0,
                            max_nanoseconds: 6.7,
                            multicycle: 2,
                            clock: "okHostClk".to_string(),
                        },
                    )),
                })
            } else {
                hi_in.add_constraint(PinConstraint {
                    index: 0,
                    constraint: Constraint::Timing(Periodic(PeriodicTiming {
                        net: "okHostClk".into(),
                        period_nanoseconds: 20.83,
                        duty_cycle: 50.0,
                    })),
                });
                hi_in.add_constraint(PinConstraint {
                    index: 0,
                    constraint: Constraint::Timing(VivadoClockGroup(vec![
                        vec!["okHostClk".to_string()],
                        vec!["sys_clock_p".to_string()],
                    ])),
                })
            }
        }
        let mut hi_out = Signal::default();
        for (ndx, name) in ["Y21", "U20"].iter().enumerate() {
            hi_out.add_location(ndx, name);
            hi_out.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
            hi_out.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Timing(VivadoOutputTiming(VivadoOutputTimingConstraint {
                    delay_nanoseconds: 8.9,
                    clock: "okHostClk".to_string(),
                })),
            })
        }
        let mut hi_inout = Signal::default();
        for (ndx, name) in [
            "AB22", "AB21", "Y22", "AA21", "AA20", "W22", "W21", "T20", "R19", "P19", "U21", "T21",
            "R21", "P21", "R22", "P22",
        ]
        .iter()
        .enumerate()
        {
            hi_inout.add_location(ndx, name);
            hi_inout.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
            hi_inout.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Timing(VivadoInputTiming(VivadoInputTimingConstraint {
                    min_nanoseconds: 0.0,
                    max_nanoseconds: 11.0,
                    multicycle: 2,
                    clock: "okHostClk".to_string(),
                })),
            });
            hi_inout.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Timing(VivadoOutputTiming(VivadoOutputTimingConstraint {
                    delay_nanoseconds: 9.2,
                    clock: "okHostClk".to_string(),
                })),
            });
        }
        let mut hi_aa = Signal::default();
        hi_aa.add_location(0, "V22");
        hi_aa.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        let mut hi_mux = Signal::default();
        hi_mux.add_location(0, "P20");
        hi_mux.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        Self {
            sig_in: hi_in,
            sig_out: hi_out,
            sig_inout: hi_inout,
            sig_aa: hi_aa,
            sig_mux: hi_mux,
        }
    }
}
