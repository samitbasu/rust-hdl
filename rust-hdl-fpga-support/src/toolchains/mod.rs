use rust_hdl_core::prelude::*;

pub fn map_signal_type_to_lattice_string(k: &SignalType) -> &str {
    match k {
        SignalType::LowVoltageCMOS_3v3 => "LVCMOS33",
        _ => panic!(
            "Unsupported mapping for signal type {:?} in Lattice mapping",
            k
        ),
    }
}

pub fn map_signal_type_to_gowin_string(k: &SignalType) -> &str {
    match k {
        SignalType::LowVoltageCMOS_1v8 => "LVCMOS18",
        SignalType::LowVoltageCMOS_3v3 => "LVCMOS33",
        SignalType::StubSeriesTerminatedLogic_II => "SSTL18_II",
        SignalType::DifferentialStubSeriesTerminatedLogic_II => "DIFF_SSTL18_II",
        SignalType::StubSeriesTerminatedLogic_II_No_Termination => "SSTL18_II | IN_TERM=NONE",
        SignalType::DifferentialStubSeriesTerminatedLogic_II_No_Termination => {
            "DIFF_SSTL18_II | IN_TERM=NONE"
        }
        SignalType::Custom(c) => c,
        SignalType::LowVoltageDifferentialSignal_2v5 => "LVDS_25",
        SignalType::StubSeriesTerminatedLogic_1v5 => "SSTL15",
        SignalType::LowVoltageCMOS_1v5 => "LVCMOS15",
        SignalType::DifferentialStubSeriesTerminatedLogic_1v5 => "DIFF_SSTL15",
    }
}

pub fn map_signal_type_to_xilinx_string(k: &SignalType) -> &str {
    match k {
        SignalType::LowVoltageCMOS_1v8 => "LVCMOS18",
        SignalType::LowVoltageCMOS_3v3 => "LVCMOS33",
        SignalType::StubSeriesTerminatedLogic_II => "SSTL18_II",
        SignalType::DifferentialStubSeriesTerminatedLogic_II => "DIFF_SSTL18_II",
        SignalType::StubSeriesTerminatedLogic_II_No_Termination => "SSTL18_II | IN_TERM=NONE",
        SignalType::DifferentialStubSeriesTerminatedLogic_II_No_Termination => {
            "DIFF_SSTL18_II | IN_TERM=NONE"
        }
        SignalType::Custom(c) => c,
        SignalType::LowVoltageDifferentialSignal_2v5 => "LVDS_25",
        SignalType::StubSeriesTerminatedLogic_1v5 => "SSTL15",
        SignalType::LowVoltageCMOS_1v5 => "LVCMOS15",
        SignalType::DifferentialStubSeriesTerminatedLogic_1v5 => "DIFF_SSTL15",
    }
}

pub mod ecp5;
pub mod icestorm;
pub mod ise;
pub mod vivado;
