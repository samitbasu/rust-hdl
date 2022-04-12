use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[test]
fn test_sync_reset() {
    let mut uut = AutoReset::default();
    uut.clock.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5000, |x: &mut Box<AutoReset>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<AutoReset>| {
        let mut x = sim.init()?;
        x = sim.wait(15_000, x)?;
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 20_000, &vcd_path!("sr_test.vcd"))
        .unwrap();
}
