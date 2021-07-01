mod direction;
mod synth;
mod clock;
mod constant;
mod atom;
mod signal;
mod logic;
mod block;
mod probe;
mod dff;
mod simulate;
mod check_connected;
mod shortbitvec;
mod bits;
mod bitvec;
mod struct_valued;
mod module_defines;
mod named_path;

#[cfg(test)]
mod tests {
    use crate::signal::Signal;
    use crate::direction::{In, Out};
    use crate::bits::{Bit, Bits};
    use crate::clock::Clock;
    use crate::constant::Constant;
    use crate::logic::Logic;
    use crate::block::Block;
    use crate::probe::Probe;
    use crate::simulate::simulate;
    use crate::dff::DFF;
    use crate::check_connected::check_connected;
    use crate::module_defines::ModuleDefines;

    struct Strobe<const N: usize> {
        pub enable: Signal<In, Bit>,
        pub strobe: Signal<Out, Bit>,
        pub clock: Signal<In, Clock>,
        pub strobe_incr: Constant<Bits<N>>,
        counter: DFF<Bits<N>>,
    }

    impl<const N: usize> Strobe<N> {
        pub fn new() -> Self {
            let mut x = Self {
                enable: Signal::<In, Bit>::new(),
                strobe: Signal::<Out, Bit>::new_with_default(false),
                clock: Signal::<In, Clock>::new(),
                strobe_incr: Constant::new(1_usize.into()),
                counter: DFF::new(0_usize.into()),
            };
            x.connect();
            x
        }
    }

    impl<const N: usize> Logic for Strobe<N> {
        fn update(&mut self) {
            self.counter.clk.next = self.clock.val;
            if self.enable.val {
                self.counter.d.next = self.counter.q.val + 1; //self.strobe_incr.val;
            }
            self.strobe.next = self.enable.val & !self.counter.q.val.any();
        }
        fn connect(&mut self) {
            self.strobe.connect();
            self.counter.clk.connect();
            self.counter.d.connect();
        }
    }

    impl<const N: usize> Block for Strobe<N> {
        fn update_all(&mut self) {
            self.update();
            self.enable.update_all();
            self.strobe.update_all();
            self.clock.update_all();
            self.counter.update_all();
        }

        fn has_changed(&self) -> bool {
            self.enable.changed ||
                self.strobe.changed ||
                self.clock.changed ||
                self.counter.has_changed()
        }

        fn accept(&self, name: &str, probe: &mut dyn Probe) {
            probe.visit_start_scope(name, self);
            self.enable.accept("enable", probe);
            self.strobe.accept("strobe", probe);
            self.clock.accept("clock", probe);
            self.counter.accept("counter", probe);
            probe.visit_end_scope(name, self);
        }
    }


    #[test]
    fn test_visit_version() {
        let mut uut: Strobe<4> = Strobe::new();
        // Simulate 100 clock cycles
        uut.enable.next = true;
        println!("Starting");
        uut.clock.connect();
        uut.enable.connect();
        check_connected(&uut);
        let mut strobe_count = 0;
        for clock in 0..100_000_000 {
            uut.clock.next = Clock(clock % 2 == 0);
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            if uut.strobe.val {
                strobe_count += 1;
            }
        }
        assert_eq!(strobe_count, 6_250_000);
    }

    #[test]
    fn test_write_modules() {

        struct StrobePair {
            pub a_strobe: Strobe<4>,
            pub b_strobe: Strobe<6>,
            pub clock: Signal<In, Clock>,
            pub enable: Signal<In, Bit>,
        }

        impl StrobePair {
            fn new() -> StrobePair {
                let mut x = Self {
                    a_strobe: Strobe::new(),
                    b_strobe: Strobe::new(),
                    clock: Signal::new(),
                    enable: Signal::new(),
                };
                x.connect();
                x
            }
        }

        impl Logic for StrobePair {
            fn update(&mut self) {}
            fn connect(&mut self) {
                self.a_strobe.enable.connect();
                self.b_strobe.enable.connect();
                self.a_strobe.clock.connect();
                self.b_strobe.clock.connect();
            }
        }

        impl Block for StrobePair {
            fn update_all(&mut self) {
                self.a_strobe.update_all();
                self.b_strobe.update_all();
                self.clock.update_all();
                self.enable.update_all();
            }

            fn has_changed(&self) -> bool {
                self.a_strobe.has_changed() ||
                    self.b_strobe.has_changed() ||
                    self.clock.has_changed() ||
                    self.enable.has_changed()
            }

            fn accept(&self, name: &str, probe: &mut dyn Probe) {
                probe.visit_start_scope(name, self);
                self.a_strobe.accept("a_strobe", probe);
                self.b_strobe.accept("b_strobe", probe);
                self.clock.accept("clock", probe);
                self.enable.accept("enable", probe);
                probe.visit_end_scope(name, self);
            }
        }

        let mut uut = StrobePair::new();
        // Simulate 100 clock cycles
        //uut.enable.next = true;
        println!("Starting");
        uut.clock.connect();
        uut.enable.connect();
        check_connected(&uut);
        let mut defines = ModuleDefines::default();
        uut.accept("uut", &mut defines);
        defines.defines();
    }
}
