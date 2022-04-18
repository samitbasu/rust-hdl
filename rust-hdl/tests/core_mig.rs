use rust_hdl::bsp::ok_xem6010::mig::{MIGInstruction, MemoryInterfaceGenerator};
use rust_hdl::core::prelude::*;

#[test]
fn test_mig() {
    let mut uut = MemoryInterfaceGenerator::default();
    uut.raw_sys_clk.connect();
    uut.p0_cmd.cmd.connect();
    uut.p0_cmd.enable.connect();
    uut.p0_cmd.clock.connect();
    uut.p0_wr.clock.connect();
    uut.p0_wr.enable.connect();
    uut.p0_wr.data.connect();
    uut.p0_rd.enable.connect();
    uut.p0_rd.clock.connect();
    uut.reset.connect();
    uut.connect_all();
    yosys_validate("mig_test", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(4, |x: &mut Box<MemoryInterfaceGenerator>| {
        x.raw_sys_clk.next = !x.raw_sys_clk.val()
    });
    sim.add_clock(5, |x: &mut Box<MemoryInterfaceGenerator>| {
        x.p0_cmd.clock.next = !x.p0_cmd.clock.val()
    });
    sim.add_clock(5, |x: &mut Box<MemoryInterfaceGenerator>| {
        x.p0_wr.clock.next = !x.p0_wr.clock.val()
    });
    sim.add_clock(5, |x: &mut Box<MemoryInterfaceGenerator>| {
        x.p0_rd.clock.next = !x.p0_rd.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MemoryInterfaceGenerator>| {
        let mut x = sim.init()?;
        x.reset.next = false.into();
        x = sim.wait(20, x)?;
        x.reset.next = true.into();
        x = sim.watch(|x| x.calib_done.val(), x)?;
        wait_clock_true!(sim, p0_cmd.clock, x);
        // Feed in a set of values
        let data_vec = [
            0x1423_5686_u32,
            0xa423_5123,
            0x9851_5312,
            0xcafe_babe,
            0xdead_beef,
        ];
        for val in &data_vec {
            x.p0_wr.data.next.data = (*val).into();
            x.p0_wr.enable.next = true;
            wait_clock_cycle!(sim, p0_wr.clock, x);
        }
        x.p0_wr.enable.next = false;
        x.p0_cmd.cmd.next.byte_address = 0xD0_usize.into();
        x.p0_cmd.cmd.next.burst_len = 4_usize.into();
        x.p0_cmd.cmd.next.instruction = MIGInstruction::Write;
        x.p0_cmd.enable.next = true;
        wait_clock_cycle!(sim, p0_cmd.clock, x);
        x.p0_cmd.enable.next = false;
        wait_clock_cycle!(sim, p0_cmd.clock, x);
        x = sim.watch(|x| x.p0_cmd.empty.val() & x.p0_wr.empty.val(), x)?;
        wait_clock_cycle!(sim, p0_cmd.clock, x);
        x.p0_cmd.cmd.next.byte_address = 0xD0_usize.into();
        x.p0_cmd.cmd.next.burst_len = 4_usize.into();
        x.p0_cmd.cmd.next.instruction = MIGInstruction::Read;
        x.p0_cmd.enable.next = true;
        wait_clock_cycle!(sim, p0_cmd.clock, x);
        x.p0_cmd.enable.next = false;
        wait_clock_cycle!(sim, p0_cmd.clock, x);
        for ndx in &data_vec {
            x = sim.watch(|x| !x.p0_rd.empty.val(), x)?;
            sim_assert_eq!(sim, x.p0_rd.data.val(), *ndx, x);
            x.p0_rd.enable.next = true;
            wait_clock_cycle!(sim, p0_rd.clock, x);
            x.p0_rd.enable.next = false;
        }
        x.p0_rd.enable.next = false;
        sim_assert!(sim, x.p0_rd.empty.val(), x);
        x.p0_cmd.cmd.next.byte_address = 0xD8_usize.into(); // Want to advance by 2 words = 8 bytes
        x.p0_cmd.cmd.next.burst_len = 2_usize.into();
        x.p0_cmd.cmd.next.instruction = MIGInstruction::Read;
        x.p0_cmd.enable.next = true;
        wait_clock_cycle!(sim, p0_cmd.clock, x);
        x.p0_cmd.enable.next = false;
        wait_clock_cycle!(sim, p0_cmd.clock, x);
        for ndx in &data_vec[2..] {
            x = sim.watch(|x| !x.p0_rd.empty.val(), x)?;
            sim_assert_eq!(sim, x.p0_rd.data.val(), *ndx, x);
            x.p0_rd.enable.next = true;
            wait_clock_cycle!(sim, p0_rd.clock, x);
            x.p0_rd.enable.next = false;
        }
        x.p0_rd.enable.next = false;
        x = sim.wait(100, x)?;
        sim_assert!(sim, !x.p0_rd.error.val(), x);
        sim_assert!(sim, !x.p0_wr.error.val(), x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 10000, &vcd_path!("mig_basic.vcd"))
        .unwrap();
}
