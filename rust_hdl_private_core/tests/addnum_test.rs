use rust_hdl_private_core::prelude::*;

#[test]
fn test_addnum() {
    #[derive(LogicBlock)]
    struct AddNum {
        pub i1: Signal<In, Bits<8>>,
        pub o1: Signal<Out, Bits<8>>,
        c1: Constant<Bits<8>>,
    }

    impl Default for AddNum {
        fn default() -> Self {
            Self {
                i1: Default::default(),
                o1: Default::default(),
                c1: Constant::new(42.into()),
            }
        }
    }

    impl Logic for AddNum {
        #[hdl_gen]
        fn update(&mut self) {
            // Note that `self.c1.next` does not exist...
            self.o1.next = self.i1.val() + self.c1.val();
        }
    }

    let mut sim: Simulation<AddNum> = Simulation::default();
    sim.add_testbench(|mut ep: Sim<AddNum>| {
        let mut x = ep.init()?;
        x.i1.next = 13.into();
        x = ep.wait(1, x)?;
        sim_assert_eq!(ep, x.o1.val(), 55, x);
        ep.done(x)
    });

    let mut uut = AddNum::default();
    uut.connect_all();
    sim.run(Box::new(uut), 100).unwrap();
}
