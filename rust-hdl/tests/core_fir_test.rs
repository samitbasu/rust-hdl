use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[test]
fn test_fir_is_synthesizable() {
    let coeffs = [1_i16, 2, 3, 2, 1];
    let mut uut = MultiplyAccumulateSymmetricFiniteImpulseResponseFilter::<3>::new(&coeffs);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("fir_synth", &vlog).unwrap();
}

#[test]
fn test_fir_impulse_response_is_expected() {
    type MACFIRTest = MultiplyAccumulateSymmetricFiniteImpulseResponseFilter<3>;
    let coeffs = [1_i16, 3, 4, 5, 4, 3, 1];
    let mut uut = MACFIRTest::new(&coeffs);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("fir_sim", &vlog).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MACFIRTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MACFIRTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for i in 0..15 {
            x = sim.watch(|x| x.strobe_out.val(), x)?;
            println!("Output value: {} -> {:x}", i, x.data_out.val());
            x = sim.watch(|x| !x.strobe_out.val(), x)?;
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MACFIRTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for val in [0_i32, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0] {
            x.data_in.next = val.into();
            x.strobe_in.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.strobe_in.next = false;
            wait_clock_cycles!(sim, clock, x, 15);
        }
        sim.done(x)?;
        Ok(())
    });
    let mut x = MACFIRTest::new(&coeffs);
    x.connect_all();
    sim.run_traced(
        Box::new(x),
        10000,
        std::fs::File::create(vcd_path!("fir.vcd")).unwrap(),
    )
    .unwrap()
}
