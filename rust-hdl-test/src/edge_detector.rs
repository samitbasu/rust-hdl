use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::edge_detector::EdgeDetector;

#[test]
fn test_rising_edge_detector_works() {
    let mut uut = EdgeDetector::new(true);
    uut.clock.connect();
    uut.input_signal.connect();
    uut.connect_all();
    yosys_validate("edge_2", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<EdgeDetector>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<EdgeDetector>| {
        let mut x = sim.init()?;
        x.input_signal.next = true;
        wait_clock_true!(sim, clock, x);
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.edge_signal.val(), x);
        x.input_signal.next = false;
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.edge_signal.val(), x);
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.edge_signal.val(), x);
        x.input_signal.next = true;
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.edge_signal.val(), x);
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.edge_signal.val(), x);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create("edge_det2.vcd").unwrap(),
    )
    .unwrap();
}

#[test]
fn test_falling_edge_detector_works() {
    let mut uut = EdgeDetector::new(false);
    uut.clock.connect();
    uut.input_signal.connect();
    uut.connect_all();
    yosys_validate("edge_1", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<EdgeDetector>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<EdgeDetector>| {
        let mut x = sim.init()?;
        x.input_signal.next = false;
        wait_clock_true!(sim, clock, x);
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.edge_signal.val(), x);
        x.input_signal.next = true;
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.edge_signal.val(), x);
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.edge_signal.val(), x);
        x.input_signal.next = false;
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.edge_signal.val(), x);
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.edge_signal.val(), x);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create("edge_det.vcd").unwrap(),
    )
    .unwrap();
}
