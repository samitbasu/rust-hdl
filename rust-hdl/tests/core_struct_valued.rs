use rand::Rng;
use rust_hdl::core::prelude::*;

// Create a test struxt
#[derive(Clone, Copy, Debug, PartialEq, LogicState)]
enum MIGCommand {
    Noop,
    Read,
    Write,
    ReadPrecharge,
    WritePrechage,
    Refresh,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, LogicStruct)]
struct MIGStruct {
    instruction: MIGCommand,
    burst_length: Bits<6>,
    byte_address: Bits<30>,
}

#[derive(LogicBlock, Default)]
struct MIGTester {
    pub cmd_in: Signal<In, MIGStruct>,
    pub cmd_out: Signal<Out, MIGStruct>,
    pub cmd_local: Signal<Local, MIGStruct>,
}

impl Logic for MIGTester {
    #[hdl_gen]
    fn update(&mut self) {
        self.cmd_local.next = self.cmd_in.val();
        if self.cmd_local.val().instruction == MIGCommand::Read {
            self.cmd_local.next.byte_address = 1.into();
            self.cmd_local.next.instruction = MIGCommand::Write;
        }
        self.cmd_out.next.instruction = self.cmd_local.val().instruction;
        self.cmd_out.next.byte_address = self.cmd_local.val().byte_address;
        self.cmd_out.next.burst_length = self.cmd_local.val().burst_length;
    }
}

#[test]
fn test_mig_tester_vcd() {
    let mut uut = MIGTester::default();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_testbench(move |mut sim: Sim<MIGTester>| {
        let mut x = sim.init()?;
        x.cmd_in.next.instruction = MIGCommand::Read;
        x.cmd_in.next.burst_length = 53.into();
        x.cmd_in.next.byte_address = 15342.into();
        x = sim.wait(10, x)?;
        x.cmd_in.next.instruction = MIGCommand::Write;
        x.cmd_in.next.burst_length = 42.into();
        x.cmd_in.next.byte_address = 142.into();
        x = sim.wait(10, x)?;
        x.cmd_in.next.instruction = MIGCommand::WritePrechage;
        x.cmd_in.next.burst_length = 13.into();
        x.cmd_in.next.byte_address = 14.into();
        x = sim.wait(10, x)?;
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MIGTester>| {
        let mut x = sim.init()?;
        x = sim.wait(5, x)?;
        sim_assert!(sim, x.cmd_out.val().instruction == MIGCommand::Write, x);
        sim_assert!(sim, x.cmd_out.val().byte_address == 1, x);
        sim_assert!(sim, x.cmd_out.val().burst_length == 53, x);
        x = sim.wait(10, x)?;
        sim_assert!(sim, x.cmd_out.val().instruction == MIGCommand::Write, x);
        sim_assert!(sim, x.cmd_out.val().byte_address == 142, x);
        sim_assert!(sim, x.cmd_out.val().burst_length == 42, x);
        x = sim.wait(10, x)?;
        sim_assert!(
            sim,
            x.cmd_out.val().instruction == MIGCommand::WritePrechage,
            x
        );
        sim_assert!(sim, x.cmd_out.val().byte_address == 14, x);
        sim_assert!(sim, x.cmd_out.val().burst_length == 13, x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 1000, "mig_test.vcd")
        .unwrap();
}

#[test]
fn test_mig_tester_synthesizes() {
    let mut uut = MIGTester::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("mig_tester_struct", &generate_verilog(&uut)).unwrap();
}

#[test]
fn test_struct_pack() {
    for _ in 0..10_000 {
        let opcode = rand::thread_rng().gen::<u8>() % 6;
        let instruction = match opcode {
            0 => MIGCommand::Noop,
            1 => MIGCommand::Read,
            2 => MIGCommand::Write,
            3 => MIGCommand::ReadPrecharge,
            4 => MIGCommand::WritePrechage,
            5 => MIGCommand::Refresh,
            _ => panic!("Unexpected random enum value"),
        };
        let burst_length = rand::thread_rng().gen::<u8>().to_bits();
        let byte_address = rand::thread_rng().gen::<u32>().to_bits();
        let x = MIGStruct {
            instruction,
            burst_length,
            byte_address,
        };
        let y: Bits<{ MIGStruct::BITS }> = x.into();
        let recon_byte_address = y.get_bits::<30>(x.get_my_offset_byte_address());
        let recon_burst_length = y.get_bits::<6>(x.get_my_offset_burst_length());
        let recon_opcode = y.get_bits::<{ MIGCommand::BITS }>(x.get_my_offset_instruction());
        assert_eq!(recon_burst_length, burst_length);
        assert_eq!(recon_opcode, opcode.to_bits());
        assert_eq!(recon_byte_address, byte_address);
    }
}
