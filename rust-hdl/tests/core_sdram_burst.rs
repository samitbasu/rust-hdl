use rust_hdl::core::prelude::*;
use rust_hdl::sim::sdr_sdram::chip::SDRAMSimulator;
use rust_hdl::widgets::prelude::*;
use rust_hdl::widgets::sdram::buffer::SDRAMOnChipBuffer;

#[derive(LogicBlock)]
struct TestSDRAMDevice {
    dram: SDRAMSimulator<5, 5, 10, 16>,
    buffer: SDRAMOnChipBuffer<16>,
    cntrl: SDRAMBurstController<5, 5, 4, 16>,
    clock: Signal<In, Clock>,
}

impl Logic for TestSDRAMDevice {
    #[hdl_gen]
    fn update(&mut self) {
        SDRAMDriver::<16>::join(&mut self.cntrl.sdram, &mut self.buffer.buf_in);
        SDRAMDriver::<16>::join(&mut self.buffer.buf_out, &mut self.dram.sdram);
        clock!(self, clock, cntrl);
    }
}

#[cfg(test)]
fn make_test_device() -> TestSDRAMDevice {
    let timings = MemoryTimings::fast_boot_sim(100e6);
    // Because the buffer adds 1 cycle of read delay
    // we need to extend the SDRAM CAS by 1 clock.
    let mut uut = TestSDRAMDevice {
        dram: SDRAMSimulator::new(timings),
        buffer: Default::default(),
        cntrl: SDRAMBurstController::new(3, timings, OutputBuffer::DelayTwo),
        clock: Default::default(),
    };
    uut.cntrl.data_in.connect();
    uut.cntrl.cmd_strobe.connect();
    uut.cntrl.cmd_address.connect();
    uut.cntrl.write_not_read.connect();
    uut.connect_all();
    uut
}

#[cfg(test)]
fn make_test_controller() -> SDRAMBurstController<5, 8, 8, 16> {
    let timings = MemoryTimings::fast_boot_sim(100e6);
    let mut uut = SDRAMBurstController::new(3, timings, OutputBuffer::DelayOne);
    uut.connect_all();
    uut
}

#[test]
fn test_sdram_controller_is_synthesizable() {
    let uut = make_test_controller();
    let vlog = generate_verilog(&uut);
    yosys_validate("sdram_burst_controller", &vlog).unwrap();
}

#[test]
fn test_unit_is_synthesizable() {
    let uut = make_test_device();
    let vlog = generate_verilog(&uut);
    yosys_validate("sdram_burst_test_unit", &vlog).unwrap();
}

#[test]
fn test_unit_boots() {
    let uut = make_test_device();
    let mut sim = Simulation::new();
    sim.add_clock(5000, |x: &mut Box<TestSDRAMDevice>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<TestSDRAMDevice>| {
        let mut x = sim.init()?;
        x = sim.wait(10_000_000, x)?;
        sim_assert!(sim, !x.dram.test_error.val(), x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 12_000_000, "burst_sdram_boot.vcd")
        .unwrap()
}

#[macro_export]
macro_rules! sdram_basic_write {
    ($sim: ident, $uut: ident, $cntrl: ident, $addr: expr, $data: expr) => {
        $uut = $sim.watch(|x| !x.$cntrl.busy.val(), $uut)?;
        $uut.$cntrl.cmd_address.next = ($addr as u32).to_bits();
        $uut.$cntrl.write_not_read.next = true;
        $uut.$cntrl.cmd_strobe.next = true;
        wait_clock_cycle!($sim, clock, $uut);
        $uut.$cntrl.cmd_strobe.next = false;
        $uut.$cntrl.cmd_address.next = 0.into();
        $uut.$cntrl.write_not_read.next = false;
        $uut.$cntrl.data_in.next = 0.into();
        for datum in $data {
            $uut.$cntrl.data_in.next = (*datum as u32).to_bits();
            $uut = $sim.watch(|x| x.$cntrl.data_strobe.val(), $uut)?;
            wait_clock_cycle!($sim, clock, $uut);
        }
    };
}

#[macro_export]
macro_rules! sdram_basic_read {
    ($sim: ident, $uut: ident, $cntrl: ident, $addr: expr, $count: expr) => {{
        let mut ret = vec![];
        $uut = $sim.watch(|x| !x.$cntrl.busy.val(), $uut)?;
        $uut.$cntrl.cmd_address.next = ($addr as u32).to_bits();
        $uut.$cntrl.write_not_read.next = false;
        $uut.$cntrl.cmd_strobe.next = true;
        wait_clock_cycle!($sim, clock, $uut);
        $uut.$cntrl.cmd_strobe.next = false;
        $uut.$cntrl.cmd_address.next = 0.into();
        for _n in 0..$count {
            $uut = $sim.watch(|x| x.$cntrl.data_valid.val(), $uut)?;
            ret.push($uut.$cntrl.data_out.val().index() as u16);
            wait_clock_cycle!($sim, clock, $uut);
        }
        ret
    }};
}

#[test]
fn test_unit_writes() {
    use rand::Rng;
    let uut = make_test_device();
    let mut sim = Simulation::new();
    let test_data = (0..256)
        .map(|_| {
            (0..4)
                .map(|_| rand::thread_rng().gen::<u16>())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    sim.add_clock(5000, |x: &mut Box<TestSDRAMDevice>| {
        x.clock.next = !x.clock.val()
    });
    let send = test_data.clone();
    let recv = test_data.clone();
    sim.add_testbench(move |mut sim: Sim<TestSDRAMDevice>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        sdram_basic_write!(sim, x, cntrl, 0, &[0xDEAD, 0xBEEF, 0xCAFE, 0xBABE]);
        sdram_basic_write!(sim, x, cntrl, 4, &[0x1234, 0xABCD, 0x5678, 0xEFFE]);
        let read = sdram_basic_read!(sim, x, cntrl, 2, 4);
        wait_clock_cycles!(sim, clock, x, 10);
        sim_assert_eq!(sim, read, [0xCAFE, 0xBABE, 0x1234, 0xABCD], x);
        let read = sdram_basic_read!(sim, x, cntrl, 4, 4);
        sim_assert_eq!(sim, read, [0x1234, 0xABCD, 0x5678, 0xEFFE], x);
        for (ndx, val) in send.iter().enumerate() {
            sdram_basic_write!(sim, x, cntrl, ndx * 4 + 8, val);
        }
        for (ndx, val) in recv.iter().enumerate() {
            let read = sdram_basic_read!(sim, x, cntrl, ndx * 4 + 8, 4);
            sim_assert_eq!(sim, &read, val, x);
        }
        sim_assert!(sim, !x.dram.test_error.val(), x);
        sim.done(x)
    });
    sim.run_to_file(
        Box::new(uut),
        100_000_000,
        &vcd_path!("burst_sdram_writes.vcd"),
    )
    .unwrap()
}
