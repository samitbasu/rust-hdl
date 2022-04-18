use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock, Default)]
struct RegFIFOTest {
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, ResetN>,
    pub in_fifo: SynchronousFIFO<Bits<16>, 4, 5, 1>,
    pub reg_fifo: RegisterFIFO<Bits<16>>,
    pub out_fifo: SynchronousFIFO<Bits<16>, 4, 5, 1>,
}

impl Logic for RegFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        clock_reset!(self, clock, reset, reg_fifo, in_fifo, out_fifo);
        self.reg_fifo.data_in.next = self.in_fifo.data_out.val();
        self.in_fifo.read.next = !self.reg_fifo.full.val() & !self.in_fifo.empty.val();
        self.reg_fifo.write.next = !self.reg_fifo.full.val() & !self.in_fifo.empty.val();
        self.out_fifo.data_in.next = self.reg_fifo.data_out.val();
        self.out_fifo.write.next = !self.out_fifo.full.val() & !self.reg_fifo.empty.val();
        self.reg_fifo.read.next = !self.out_fifo.full.val() & !self.reg_fifo.empty.val();
    }
}

#[test]
fn test_register_fifo_works() {
    let mut uut = RegFIFOTest::default();
    uut.clock.connect();
    uut.reset.connect();
    uut.out_fifo.read.connect();
    uut.in_fifo.write.connect();
    uut.in_fifo.data_in.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    let rdata = (0..256)
        .map(|_| Bits::<16>::from(rand::random::<u16>()))
        .collect::<Vec<_>>();
    let rdata_read = rdata.clone();
    sim.add_clock(5, |x: &mut Box<RegFIFOTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<RegFIFOTest>| {
        let mut x = sim.init()?;
        reset_sim!(sim, clock, reset, x);
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.in_fifo.full.val(), x)?;
            x.in_fifo.data_in.next = (*sample).into();
            x.in_fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.in_fifo.write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<RegFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata_read {
            x = sim.watch(|x| !x.out_fifo.empty.val(), x)?;
            sim_assert!(sim, x.out_fifo.data_out.val().eq(sample), x);
            x.out_fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.out_fifo.read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        1_000_000,
        std::fs::File::create(vcd_path!("reg_fifo.vcd")).unwrap(),
    )
    .unwrap();
}
