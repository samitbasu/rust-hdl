// Wrap the MIG (Memory Interface Generator) into a RustHDL object
use rust_hdl_core::direction::Direction;
use rust_hdl_core::prelude::*;

#[derive(Clone, LogicInterface, Default)]
pub struct MCBInterface {
    data_bus: Signal<InOut, Bits<16>>,
    address: Signal<Out, Bits<13>>,
    bank_select: Signal<Out, Bits<3>>,
    row_address_strobe_not: Signal<Out, Bit>,
    column_address_strobe_not: Signal<Out, Bit>,
    write_enable_not: Signal<Out, Bit>,
    on_die_termination: Signal<Out, Bit>,
    clock_enable: Signal<Out, Bit>,
    data_mask: Signal<Out, Bit>,
    upper_byte_data_strobe: Signal<Out, Bit>,
    upper_byte_data_strobe_neg: Signal<Out, Bit>,
    rzq: Signal<InOut, Bit>,
    zio: Signal<InOut, Bit>,
    upper_data_mask: Signal<Out, Bit>,
    data_strobe_signal: Signal<InOut, Bit>,
    data_strobe_signal_neg: Signal<InOut, Bit>,
    dram_clock: Signal<Out, Bit>,
    dram_clock_neg: Signal<Out, Bit>,
    chip_select_neg: Signal<Out, Bit>,
}

impl MCBInterface {
    pub fn xem_6010() -> MCBInterface {
        fn mk_pin<D: Direction>(loc: &str, kind: SignalType) -> Signal<D, Bit> {
            let mut p = Signal::default();
            p.add_location(0, loc);
            p.add_signal_type(0, kind);
            p
        }
        // Pin validation - locations should be unique
        // IO signals should not be terminated
        let mut data_bus = Signal::default();
        for (ndx, name) in [
            "N3", "N1", "M2", "M1", "J3", "J1", "K2", "K1", "P2", "P1", "R3", "R1", "U3", "U1",
            "V2", "V1",
        ]
        .iter()
        .enumerate()
        {
            data_bus.add_location(ndx, name);
            data_bus.add_signal_type(ndx, SignalType::StubSeriesTerminatedLogic_II_No_Termination);
        }
        let mut address = Signal::default();
        for (ndx, name) in [
            "H2", "H1", "H5", "K6", "F3", "K3", "J4", "H6", "E3", "E1", "G4", "C1", "D1",
        ]
        .iter()
        .enumerate()
        {
            address.add_location(ndx, name);
            address.add_signal_type(ndx, SignalType::StubSeriesTerminatedLogic_II);
        }
        let mut bank_select = Signal::default();
        for (ndx, name) in ["G3", "G1", "F1"].iter().enumerate() {
            bank_select.add_location(ndx, name);
            bank_select.add_signal_type(ndx, SignalType::StubSeriesTerminatedLogic_II);
        }
        let row_address_strobe_not = mk_pin("K5", SignalType::StubSeriesTerminatedLogic_II);
        let column_address_strobe_not = mk_pin("K4", SignalType::StubSeriesTerminatedLogic_II);
        let write_enable_not = mk_pin("F2", SignalType::StubSeriesTerminatedLogic_II);
        let on_die_termination = mk_pin("J6", SignalType::StubSeriesTerminatedLogic_II);
        let clock_enable = mk_pin("D2", SignalType::StubSeriesTerminatedLogic_II);
        let data_mask = mk_pin("L4", SignalType::StubSeriesTerminatedLogic_II);
        let upper_byte_data_strobe = mk_pin(
            "T2",
            SignalType::DifferentialStubSeriesTerminatedLogic_II_No_Termination,
        );
        let upper_byte_data_strobe_neg = mk_pin(
            "T1",
            SignalType::DifferentialStubSeriesTerminatedLogic_II_No_Termination,
        );
        let rzq = mk_pin("K7", SignalType::StubSeriesTerminatedLogic_II);
        let zio = mk_pin("Y2", SignalType::StubSeriesTerminatedLogic_II);
        let upper_data_mask = mk_pin("M3", SignalType::StubSeriesTerminatedLogic_II);
        let data_strobe_signal = mk_pin(
            "L3",
            SignalType::DifferentialStubSeriesTerminatedLogic_II_No_Termination,
        );
        let data_strobe_signal_neg = mk_pin(
            "L1",
            SignalType::DifferentialStubSeriesTerminatedLogic_II_No_Termination,
        );
        let dram_clock = mk_pin("H4", SignalType::DifferentialStubSeriesTerminatedLogic_II);
        let dram_clock_neg = mk_pin("H3", SignalType::DifferentialStubSeriesTerminatedLogic_II);
        let chip_select_neg = mk_pin("C3", SignalType::LowVoltageCMOS_1v8);
        MCBInterface {
            data_bus,
            address,
            bank_select,
            row_address_strobe_not,
            column_address_strobe_not,
            write_enable_not,
            on_die_termination,
            clock_enable,
            data_mask,
            upper_byte_data_strobe,
            upper_byte_data_strobe_neg,
            rzq,
            zio,
            upper_data_mask,
            data_strobe_signal,
            data_strobe_signal_neg,
            dram_clock,
            dram_clock_neg,
            chip_select_neg,
        }
    }
}

#[derive(LogicBlock)]
struct DRAMIFTest5 {
    pub mcb_dram: MCBInterface,
}

impl Logic for DRAMIFTest5 {
    fn update(&mut self) {}
}

#[test]
fn test_dram_if_ucf() {
    let uut = DRAMIFTest5 {
        mcb_dram: MCBInterface::xem_6010(),
    };
    let ucf = crate::ucf_gen::generate_ucf(&uut);
    println!("{}", ucf);
    assert!(ucf.contains("mcb_dram_zio LOC=Y2"));
    assert!(ucf.contains("mcb_dram_rzq LOC=K7"));
    assert!(ucf.contains("mcb_dram_address<0> LOC=H2;"));
    assert!(ucf.contains("mcb_dram_address<1> LOC=H1;"));
    assert!(ucf.contains("mcb_dram_address<2> LOC=H5;"));
    assert!(ucf.contains("mcb_dram_address<3> LOC=K6;"));
    assert!(ucf.contains("mcb_dram_address<4> LOC=F3;"));
    assert!(ucf.contains("mcb_dram_address<5> LOC=K3;"));
    assert!(ucf.contains("mcb_dram_address<6> LOC=J4;"));
    assert!(ucf.contains("mcb_dram_address<7> LOC=H6;"));
    assert!(ucf.contains("mcb_dram_address<8> LOC=E3;"));
    assert!(ucf.contains("mcb_dram_address<9> LOC=E1;"));
    assert!(ucf.contains("mcb_dram_address<10> LOC=G4;"));
    assert!(ucf.contains("mcb_dram_address<11> LOC=C1;"));
    assert!(ucf.contains("mcb_dram_address<12> LOC=D1;"));
    assert!(ucf.contains("mcb_dram_bank_select<0> LOC=G3;"));
    assert!(ucf.contains("mcb_dram_bank_select<1> LOC=G1;"));
    assert!(ucf.contains("mcb_dram_bank_select<2> LOC=F1;"));
    assert!(ucf.contains("mcb_dram_column_address_strobe_not LOC=K4;"));
    assert!(ucf.contains("mcb_dram_dram_clock LOC=H4;"));
    assert!(ucf.contains("mcb_dram_dram_clock_neg LOC=H3;"));
    assert!(ucf.contains("mcb_dram_clock_enable LOC=D2;"));
    assert!(ucf.contains("mcb_dram_data_mask LOC=L4;"));
    assert!(ucf.contains("mcb_dram_data_bus<0> LOC=N3;"));
    assert!(ucf.contains("mcb_dram_data_bus<10> LOC=R3;"));
    assert!(ucf.contains("mcb_dram_data_bus<11> LOC=R1;"));
    assert!(ucf.contains("mcb_dram_data_bus<12> LOC=U3;"));
    assert!(ucf.contains("mcb_dram_data_bus<13> LOC=U1;"));
    assert!(ucf.contains("mcb_dram_data_bus<14> LOC=V2;"));
    assert!(ucf.contains("mcb_dram_data_bus<15> LOC=V1;"));
    assert!(ucf.contains("mcb_dram_data_bus<1> LOC=N1;"));
    assert!(ucf.contains("mcb_dram_data_bus<2> LOC=M2;"));
    assert!(ucf.contains("mcb_dram_data_bus<3> LOC=M1;"));
    assert!(ucf.contains("mcb_dram_data_bus<4> LOC=J3;"));
    assert!(ucf.contains("mcb_dram_data_bus<5> LOC=J1;"));
    assert!(ucf.contains("mcb_dram_data_bus<6> LOC=K2;"));
    assert!(ucf.contains("mcb_dram_data_bus<7> LOC=K1;"));
    assert!(ucf.contains("mcb_dram_data_bus<8> LOC=P2;"));
    assert!(ucf.contains("mcb_dram_data_bus<9> LOC=P1;"));
    assert!(ucf.contains("mcb_dram_data_strobe_signal LOC=L3;"));
    assert!(ucf.contains("mcb_dram_data_strobe_signal_neg LOC=L1;"));
    assert!(ucf.contains("mcb_dram_on_die_termination LOC=J6;"));
    assert!(ucf.contains("mcb_dram_row_address_strobe_not LOC=K5;"));
    assert!(ucf.contains("mcb_dram_upper_data_mask LOC=M3;"));
    assert!(ucf.contains("mcb_dram_upper_byte_data_strobe LOC=T2;"));
    assert!(ucf.contains("mcb_dram_upper_byte_data_strobe_neg LOC=T1;"));
    assert!(ucf.contains("mcb_dram_write_enable_not LOC=F2;"));
    assert!(ucf.contains("mcb_dram_chip_select_neg LOC=C3;"));
    assert!(ucf.contains("mcb_dram_zio IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_rzq IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<0> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<1> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<2> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<3> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<4> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<5> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<6> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<7> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<8> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<9> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<10> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<11> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_address<12> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_bank_select<0> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_bank_select<1> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_bank_select<2> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_column_address_strobe_not IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_dram_clock IOSTANDARD=DIFF_SSTL18_II"));
    assert!(ucf.contains("mcb_dram_dram_clock_neg IOSTANDARD=DIFF_SSTL18_II"));
    assert!(ucf.contains("mcb_dram_clock_enable IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_mask IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<0> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<10> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<11> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<12> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<13> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<14> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<15> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<1> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<2> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<3> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<4> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<5> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<6> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<7> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<8> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_bus<9> IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_strobe_signal IOSTANDARD=DIFF_SSTL18_II"));
    assert!(ucf.contains("mcb_dram_data_strobe_signal_neg IOSTANDARD=DIFF_SSTL18_II"));
    assert!(ucf.contains("mcb_dram_on_die_termination IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_row_address_strobe_not IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_upper_data_mask IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_upper_byte_data_strobe IOSTANDARD=DIFF_SSTL18_II"));
    assert!(ucf.contains("mcb_dram_upper_byte_data_strobe_neg IOSTANDARD=DIFF_SSTL18_II"));
    assert!(ucf.contains("mcb_dram_write_enable_not IOSTANDARD=SSTL18_II"));
    assert!(ucf.contains("mcb_dram_chip_select_neg IOSTANDARD=LVCMOS18"));
    fn is_unterm(ucf: &str, net: &str) -> bool {
        for k in ucf.split("\n") {
            if k.starts_with(&format!("NET {} IOSTANDARD", net)) {
                return k.ends_with("IN_TERM=NONE;");
            }
        }
        false
    }
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<0>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<1>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<2>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<3>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<4>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<5>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<6>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<7>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<8>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<9>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<10>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<11>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_address<12>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_bank_select<0>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_bank_select<1>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_bank_select<2>"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_column_address_strobe_not"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_dram_clock"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_dram_clock_neg"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_clock_enable"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_mask"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<0>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<10>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<11>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<12>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<13>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<14>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<15>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<1>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<2>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<3>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<4>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<5>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<6>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<7>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<8>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_bus<9>"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_strobe_signal"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_data_strobe_signal_neg"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_on_die_termination"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_row_address_strobe_not"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_upper_data_mask"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_upper_byte_data_strobe"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_upper_byte_data_strobe_neg"), true);
    assert_eq!(is_unterm(&ucf, "mcb_dram_write_enable_not"), false);
    assert_eq!(is_unterm(&ucf, "mcb_dram_chip_select_neg"), false);
}
