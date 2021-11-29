use rust_hdl_core::prelude::*;

#[derive(Clone, LogicInterface, Default)]
pub struct MCBInterface4GDDR3 {
    data_bus: Signal<InOut, Bits<16>>,
    address: Signal<Out, Bits<15>>,
    bank_select: Signal<Out, Bits<3>>,
    row_address_strobe_not: Signal<Out, Bit>,
    column_address_strobe_not: Signal<Out, Bit>,
    write_enable_not: Signal<Out, Bit>,
    on_die_termination: Signal<Out, Bit>,
    clock_enable: Signal<Out, Bit>,
    data_mask: Signal<Out, Bits<2>>,
    data_strobe_signal: Signal<InOut, Bits<2>>,
    data_strobe_signal_neg: Signal<InOut, Bits<2>>,
    dram_clock: Signal<Out, Bit>,
    dram_clock_neg: Signal<Out, Bit>,
    reset_not: Signal<Out, Bit>,
}

fn mk_pin<D: Direction>(loc: &str, kind: SignalType) -> Signal<D, Bit> {
    let mut p = Signal::default();
    p.add_location(0, loc);
    p.add_signal_type(0, kind);
    p
}

fn mk_pin7<D: Direction>(loc: &str) -> Signal<D, Bit> {
    let mut p = Signal::default();
    p.add_location(0, loc);
    p.add_signal_type(0, SignalType::StubSeriesTerminatedLogic_1v5);
    p.add_constraint(PinConstraint {
        index: 0,
        constraint: Constraint::Slew(SlewType::Fast),
    });
    p
}

impl MCBInterface4GDDR3 {
    pub fn xem_7010() -> Self {
        Default::default()
    }
    pub fn xem_7010_constrained() -> Self {
        let mut data_bus = Signal::default();
        for (ndx, name) in [
            "AB1", "Y4", "AB2", "V4", "AB5", "AA5", "AB3", "AA4", "U3", "W2", "U2", "Y2", "U1",
            "Y1", "T1", "W1",
        ]
        .iter()
        .enumerate()
        {
            data_bus.add_location(ndx, name);
            data_bus.add_signal_type(ndx, SignalType::StubSeriesTerminatedLogic_1v5);
            data_bus.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Slew(SlewType::Fast),
            });
        }
        let mut address = Signal::default();
        for (ndx, name) in [
            "W6", "U7", "W7", "Y6", "U6", "AB7", "Y8", "AB8", "Y7", "AA8", "T4", "V7", "T6", "Y9",
            "W9",
        ]
        .iter()
        .enumerate()
        {
            address.add_location(ndx, name);
            address.add_signal_type(ndx, SignalType::StubSeriesTerminatedLogic_1v5);
            address.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Slew(SlewType::Fast),
            });
        }
        let mut bank_select = Signal::default();
        for (ndx, name) in ["AB6", "R6", "AA6"].iter().enumerate() {
            bank_select.add_location(ndx, name);
            bank_select.add_signal_type(ndx, SignalType::StubSeriesTerminatedLogic_1v5);
            bank_select.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Slew(SlewType::Fast),
            });
        }
        let row_address_strobe_not = mk_pin7("V5");
        let column_address_strobe_not = mk_pin7("U5");
        let write_enable_not = mk_pin7("T5");
        let mut reset_not = mk_pin("T3", SignalType::LowVoltageCMOS_1v5);
        reset_not.add_constraint(PinConstraint {
            index: 0,
            constraint: Constraint::Slew(SlewType::Fast),
        });
        let on_die_termination = mk_pin7("W5");
        let clock_enable = mk_pin7("R4");
        let mut data_mask = Signal::default();
        for (ndx, name) in ["AA1", "V2"].iter().enumerate() {
            data_mask.add_location(ndx, name);
            data_mask.add_signal_type(ndx, SignalType::StubSeriesTerminatedLogic_1v5);
            data_mask.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Slew(SlewType::Fast),
            })
        }
        let mut data_strobe_signal = Signal::default();
        for (ndx, name) in ["Y3", "R3"].iter().enumerate() {
            data_strobe_signal.add_location(ndx, name);
            data_strobe_signal
                .add_signal_type(ndx, SignalType::DifferentialStubSeriesTerminatedLogic_1v5);
            data_strobe_signal.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Slew(SlewType::Fast),
            })
        }
        let mut data_strobe_signal_neg = Signal::default();
        for (ndx, name) in ["AA3", "R2"].iter().enumerate() {
            data_strobe_signal_neg.add_location(ndx, name);
            data_strobe_signal_neg
                .add_signal_type(ndx, SignalType::DifferentialStubSeriesTerminatedLogic_1v5);
            data_strobe_signal_neg.add_constraint(PinConstraint {
                index: ndx,
                constraint: Constraint::Slew(SlewType::Fast),
            })
        }
        let mut dram_clock = Signal::default();
        dram_clock.add_location(0, "V9");
        dram_clock.add_signal_type(0, SignalType::DifferentialStubSeriesTerminatedLogic_1v5);
        dram_clock.add_constraint(PinConstraint {
            index: 0,
            constraint: Constraint::Slew(SlewType::Fast),
        });
        let mut dram_clock_neg = Signal::default();
        dram_clock_neg.add_location(0, "V8");
        dram_clock_neg.add_signal_type(0, SignalType::DifferentialStubSeriesTerminatedLogic_1v5);
        dram_clock_neg.add_constraint(PinConstraint {
            index: 0,
            constraint: Constraint::Slew(SlewType::Fast),
        });
        Self {
            data_bus,
            address,
            bank_select,
            row_address_strobe_not,
            column_address_strobe_not,
            write_enable_not,
            on_die_termination,
            clock_enable,
            data_mask,
            data_strobe_signal,
            data_strobe_signal_neg,
            dram_clock,
            dram_clock_neg,
            reset_not,
        }
    }
}

#[test]
fn test_dram7_if_xdc() {
    use rust_hdl_yosys_synth::TopWrap;
    let uut = TopWrap::new(MCBInterface4GDDR3::xem_7010_constrained());
    let xdc = rust_hdl_toolchain_vivado::xdc_gen::generate_xdc(&uut);
    println!("{}", xdc);
    assert!(xdc.contains("set_property PACKAGE_PIN AB1 [get_ports { uut$data_bus[0] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN Y4 [get_ports { uut$data_bus[1] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AB2 [get_ports { uut$data_bus[2] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN V4 [get_ports { uut$data_bus[3] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AB5 [get_ports { uut$data_bus[4] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AA5 [get_ports { uut$data_bus[5] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AB3 [get_ports { uut$data_bus[6] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AA4 [get_ports { uut$data_bus[7] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN U3 [get_ports { uut$data_bus[8] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN W2 [get_ports { uut$data_bus[9] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN U2 [get_ports { uut$data_bus[10] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN Y2 [get_ports { uut$data_bus[11] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN U1 [get_ports { uut$data_bus[12] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN Y1 [get_ports { uut$data_bus[13] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN T1 [get_ports { uut$data_bus[14] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN W1 [get_ports { uut$data_bus[15] }]"));
    for ndx in 0..16 {
        assert!(xdc.contains(&format!(
            "set_property IOSTANDARD SSTL15 [get_ports {{ uut$data_bus[{}] }}]",
            ndx
        )));
        assert!(xdc.contains(&format!(
            "set_property SLEW FAST [get_ports {{ uut$data_bus[{}] }}]",
            ndx
        )));
    }
    assert!(xdc.contains("set_property PACKAGE_PIN W6 [get_ports { uut$address[0] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN U7 [get_ports { uut$address[1] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN W7 [get_ports { uut$address[2] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN Y6 [get_ports { uut$address[3] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN U6 [get_ports { uut$address[4] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AB7 [get_ports { uut$address[5] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN Y8 [get_ports { uut$address[6] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AB8 [get_ports { uut$address[7] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN Y7 [get_ports { uut$address[8] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AA8 [get_ports { uut$address[9] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN T4 [get_ports { uut$address[10] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN V7 [get_ports { uut$address[11] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN T6 [get_ports { uut$address[12] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN Y9 [get_ports { uut$address[13] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN W9 [get_ports { uut$address[14] }]"));
    for ndx in 0..15 {
        assert!(xdc.contains(&format!(
            "set_property IOSTANDARD SSTL15 [get_ports {{ uut$address[{}] }}]",
            ndx
        )));
        assert!(xdc.contains(&format!(
            "set_property SLEW FAST [get_ports {{ uut$address[{}] }}]",
            ndx
        )));
    }
    assert!(xdc.contains("set_property PACKAGE_PIN AB6 [get_ports { uut$bank_select[0] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN R6 [get_ports { uut$bank_select[1] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN AA6 [get_ports { uut$bank_select[2] }]"));
    for ndx in 0..3 {
        assert!(xdc.contains(&format!(
            "set_property IOSTANDARD SSTL15 [get_ports {{ uut$bank_select[{}] }}]",
            ndx
        )));
        assert!(xdc.contains(&format!(
            "set_property SLEW FAST [get_ports {{ uut$bank_select[{}] }}]",
            ndx
        )));
    }
    assert!(
        xdc.contains("set_property PACKAGE_PIN U5 [get_ports { uut$column_address_strobe_not }]")
    );
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$column_address_strobe_not }]"));
    assert!(xdc
        .contains("set_property IOSTANDARD SSTL15 [get_ports { uut$column_address_strobe_not }]"));

    assert!(xdc.contains("set_property PACKAGE_PIN V5 [get_ports { uut$row_address_strobe_not }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$row_address_strobe_not }]"));
    assert!(
        xdc.contains("set_property IOSTANDARD SSTL15 [get_ports { uut$row_address_strobe_not }]")
    );

    assert!(xdc.contains("set_property PACKAGE_PIN T5 [get_ports { uut$write_enable_not }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$write_enable_not }]"));
    assert!(xdc.contains("set_property IOSTANDARD SSTL15 [get_ports { uut$write_enable_not }]"));

    assert!(xdc.contains("set_property PACKAGE_PIN T3 [get_ports { uut$reset_not }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$reset_not }]"));
    assert!(xdc.contains("set_property IOSTANDARD LVCMOS15 [get_ports { uut$reset_not }]"));

    assert!(xdc.contains("set_property PACKAGE_PIN R4 [get_ports { uut$clock_enable }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$clock_enable }]"));
    assert!(xdc.contains("set_property IOSTANDARD SSTL15 [get_ports { uut$clock_enable }]"));

    assert!(xdc.contains("set_property PACKAGE_PIN W5 [get_ports { uut$on_die_termination }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$on_die_termination }]"));
    assert!(xdc.contains("set_property IOSTANDARD SSTL15 [get_ports { uut$on_die_termination }]"));

    assert!(xdc.contains("set_property PACKAGE_PIN AA1 [get_ports { uut$data_mask[0] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN V2 [get_ports { uut$data_mask[1] }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$data_mask[0] }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$data_mask[1] }]"));
    assert!(xdc.contains("set_property IOSTANDARD SSTL15 [get_ports { uut$data_mask[0] }]"));
    assert!(xdc.contains("set_property IOSTANDARD SSTL15 [get_ports { uut$data_mask[1] }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN Y3 [get_ports { uut$data_strobe_signal[0] }]"));
    assert!(
        xdc.contains("set_property PACKAGE_PIN AA3 [get_ports { uut$data_strobe_signal_neg[0] }]")
    );
    assert!(xdc.contains("set_property PACKAGE_PIN R3 [get_ports { uut$data_strobe_signal[1] }]"));
    assert!(
        xdc.contains("set_property PACKAGE_PIN R2 [get_ports { uut$data_strobe_signal_neg[1] }]")
    );
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$data_strobe_signal[0] }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$data_strobe_signal[1] }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$data_strobe_signal_neg[0] }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$data_strobe_signal_neg[1] }]"));
    assert!(xdc
        .contains("set_property IOSTANDARD DIFF_SSTL15 [get_ports { uut$data_strobe_signal[0] }]"));
    assert!(xdc
        .contains("set_property IOSTANDARD DIFF_SSTL15 [get_ports { uut$data_strobe_signal[1] }]"));
    assert!(xdc.contains(
        "set_property IOSTANDARD DIFF_SSTL15 [get_ports { uut$data_strobe_signal_neg[0] }]"
    ));
    assert!(xdc.contains(
        "set_property IOSTANDARD DIFF_SSTL15 [get_ports { uut$data_strobe_signal_neg[1] }]"
    ));
    assert!(xdc.contains("set_property PACKAGE_PIN V9 [get_ports { uut$dram_clock }]"));
    assert!(xdc.contains("set_property PACKAGE_PIN V8 [get_ports { uut$dram_clock_neg }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$dram_clock }]"));
    assert!(xdc.contains("set_property SLEW FAST [get_ports { uut$dram_clock_neg }]"));
    assert!(xdc.contains("set_property IOSTANDARD DIFF_SSTL15 [get_ports { uut$dram_clock }]"));
    assert!(xdc.contains("set_property IOSTANDARD DIFF_SSTL15 [get_ports { uut$dram_clock_neg }]"));
}
