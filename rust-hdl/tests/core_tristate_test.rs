use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock, Default)]
struct BusPoint {
    pub bus_wire: Signal<InOut, Bits<8>>,
    pub buffer: TristateBuffer<Bits<8>>,
}

impl Logic for BusPoint {
    fn update(&mut self) {
        self.bus_wire.link(&mut self.buffer.bus);
    }

    fn connect(&mut self) {
        self.bus_wire.connect();
    }
}

#[derive(LogicBlock, Default)]
struct BusTest {
    pub left: BusPoint,
    pub right: BusPoint,
}

impl Logic for BusTest {
    fn update(&mut self) {
        self.left.bus_wire.simulate_connected_tristate(&mut self.right.bus_wire);
    }
}

#[test]
fn test_tristate_buffer_works() {
    let mut uut = BusTest::default();
    uut.left.buffer.write_data.connect();
    uut.left.buffer.write_enable.connect();
    uut.right.buffer.write_data.connect();
    uut.right.buffer.write_enable.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    println!("Signal IDs");
    println!("Left buffer bus_wire {}", uut.left.bus_wire.id());
    println!("Right buffer bus_wire {}", uut.right.bus_wire.id());
    yosys_validate("tristate", &vlog).unwrap();
    let mut sim = Simulation::new();
    sim.add_testbench(move |mut sim: Sim<BusTest>| {
        let mut x = sim.init()?;
        // Drive the bus
        x.left.buffer.write_enable.next = true;
        for val in [42_u8, 32, 16, 0, 7] {
            x.left.buffer.write_data.next = val.into();
            x = sim.wait(10, x)?;
        }
        // Drop the bus drive
        x.left.buffer.write_enable.next = false;
        x = sim.wait(40, x)?;
        for ndx in [43_u8, 33, 17, 1, 8] {
            x = sim.wait(10, x)?;
            sim_assert!(sim, x.left.buffer.read_data.val() == ndx, x);
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<BusTest>| {
        let mut x = sim.init()?;
        x.right.buffer.write_enable.next = false;
        for ndx in [42_u8, 32, 16, 0, 7] {
            sim_assert!(sim, x.right.buffer.read_data.val() == ndx, x);
            x = sim.wait(10, x)?;
        }
        // Wait for the turn around
        x = sim.wait(40, x)?;
        x.right.buffer.write_enable.next = true;
        for ndx in [43_u8, 33, 17, 1, 8] {
            x.right.buffer.write_data.next = ndx.into();
            x = sim.wait(10, x)?;
        }
        x.right.buffer.write_enable.next = false;
        // Wait for the turn around
        x = sim.wait(40, x)?;
        sim.done(x)
    });
    sim.run_traced(Box::new(uut), 200,
                   std::fs::File::create(vcd_path!("tristate.vcd")).unwrap()).unwrap()
}