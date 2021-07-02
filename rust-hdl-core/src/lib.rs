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

    #[derive(Clone, Debug)]
    struct Strobe<const N: usize> {
        pub enable: Signal<In, Bit>,
        pub strobe: Signal<Out, Bit>,
        pub clock: Signal<In, Clock>,
        pub strobe_incr: Constant<Bits<N>>,
        counter: DFF<Bits<N>>,
    }

    impl<const N: usize> Default for Strobe<N> {
        fn default() -> Self {
            Self {
                enable: Signal::default(),
                strobe: Signal::<Out, Bit>::new_with_default(false),
                clock: Signal::default(),
                strobe_incr: Constant::new(1_usize.into()),
                counter: DFF::new(0_usize.into()),
            }
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
        fn connect_all(&mut self) {
            self.connect();
            self.enable.connect_all();
            self.strobe.connect_all();
            self.clock.connect_all();
            self.counter.connect_all();
        }

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
        let mut uut: Strobe<4> = Strobe::default();
        // Simulate 100 clock cycles
        uut.enable.next = true;
        println!("Starting");
        uut.clock.connect();
        uut.enable.connect();
        uut.connect_all();
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
    fn test_write_modules_nested_ports() {

        #[derive(Clone, Debug, Default)]
        struct MyBus {
            pub data: FIFORead<8>,
            pub cmd: FIFORead<3>,
        }

        impl Logic for MyBus {
            fn update(&mut self) {}
            fn connect(&mut self) {}
        }

        impl Block for MyBus {
            fn connect_all(&mut self) {
                self.connect();
                self.data.connect_all();
                self.cmd.connect_all();
            }

            fn update_all(&mut self) {
                self.update();
                self.data.update_all();
                self.cmd.update_all();
            }

            fn has_changed(&self) -> bool {
                self.data.has_changed() || self.cmd.has_changed()
            }

            fn accept(&self, name: &str, probe: &mut dyn Probe) {
                probe.visit_start_namespace(name, self);
                self.data.accept("data", probe);
                self.cmd.accept("cmd", probe);
                probe.visit_end_namespace(name, self);
            }
        }

        #[derive(Clone, Debug, Default)]
        struct FIFORead<const D: usize> {
            pub read: Signal<In, Bit>,
            pub output: Signal<Out, Bits<D>>,
            pub empty: Signal<Out, Bit>,
            pub almost_empty: Signal<Out, Bit>,
            pub underflow: Signal<Out, Bit>,
        }

        impl<const D: usize> Logic for FIFORead<D> {
            fn update(&mut self) {}

            fn connect(&mut self) {}
        }

        impl<const D: usize> Block for FIFORead<D> {
            fn connect_all(&mut self) {
                self.connect();
                self.read.connect_all();
                self.output.connect_all();
                self.empty.connect_all();
                self.almost_empty.connect_all();
                self.underflow.connect_all();
            }

            fn update_all(&mut self) {
                self.update();
                self.read.update_all();
                self.output.update_all();
                self.empty.update_all();
                self.almost_empty.update_all();
                self.underflow.update_all();
            }

            fn has_changed(&self) -> bool {
                self.read.has_changed() ||
                    self.output.has_changed() ||
                    self.empty.has_changed() ||
                    self.almost_empty.has_changed() ||
                    self.underflow.has_changed()
            }

            fn accept(&self, name: &str, probe: &mut dyn Probe) {
                probe.visit_start_namespace(name, self);
                self.read.accept("read", probe);
                self.output.accept("output", probe);
                self.almost_empty.accept("almost_empty", probe);
                self.underflow.accept("underflow", probe);
                probe.visit_end_namespace(name, self);
            }
        }

        #[derive(Clone, Debug, Default)]
        struct Widget {
            pub clock: Signal<In, Clock>,
            pub bus: MyBus,
        }

        impl Logic for Widget {
            fn update(&mut self) {}

            fn connect(&mut self) {
                self.bus.data.almost_empty.connect();
                self.bus.data.empty.connect();
                self.bus.data.underflow.connect();
                self.bus.data.output.connect();
                self.bus.cmd.almost_empty.connect();
                self.bus.cmd.empty.connect();
                self.bus.cmd.underflow.connect();
                self.bus.cmd.output.connect();
            }
        }

        impl Block for Widget {
            fn connect_all(&mut self) {
                self.connect();
                self.clock.connect_all();
                self.bus.connect_all();
            }

            fn update_all(&mut self) {
                self.update();
                self.clock.update_all();
                self.bus.update_all();
            }

            fn has_changed(&self) -> bool {
                self.clock.has_changed() || self.bus.has_changed()
            }

            fn accept(&self, name: &str, probe: &mut dyn Probe) {
                probe.visit_start_scope(name, self);
                self.clock.accept("clock", probe);
                self.bus.accept("bus", probe);
                probe.visit_end_scope(name, self);
            }
        }

        #[derive(Clone, Debug, Default)]
        struct UUT {
            pub bus: MyBus,
            widget_a: Widget,
            widget_b: Widget,
            pub clock: Signal<In, Clock>,
            pub select: Signal<In, Bit>,
        }

        impl Logic for UUT {
            fn update(&mut self) {
                self.widget_a.clock.next = self.clock.val;
                self.widget_b.clock.next = self.clock.val;

                if self.select.val {
                    self.bus.cmd.underflow.next = self.widget_a.bus.cmd.underflow.val;
                    self.bus.cmd.almost_empty.next = self.widget_a.bus.cmd.almost_empty.val;
                    self.bus.cmd.empty.next = self.widget_a.bus.cmd.empty.val;
                    self.bus.cmd.output.next = self.widget_a.bus.cmd.output.val;
                    self.widget_a.bus.cmd.read.next = self.bus.cmd.read.val;

                    self.bus.data.underflow.next = self.widget_a.bus.data.underflow.val;
                    self.bus.data.almost_empty.next = self.widget_a.bus.data.almost_empty.val;
                    self.bus.data.empty.next = self.widget_a.bus.data.empty.val;
                    self.bus.data.output.next = self.widget_a.bus.data.output.val;
                    self.widget_a.bus.data.read.next = self.bus.data.read.val;

                } else {
                    self.bus.cmd.underflow.next = self.widget_b.bus.cmd.underflow.val;
                    self.bus.cmd.almost_empty.next = self.widget_b.bus.cmd.almost_empty.val;
                    self.bus.cmd.empty.next = self.widget_b.bus.cmd.empty.val;
                    self.bus.cmd.output.next = self.widget_b.bus.cmd.output.val;
                    self.widget_b.bus.cmd.read.next = self.bus.cmd.read.val;

                    self.bus.data.underflow.next = self.widget_b.bus.data.underflow.val;
                    self.bus.data.almost_empty.next = self.widget_b.bus.data.almost_empty.val;
                    self.bus.data.empty.next = self.widget_b.bus.data.empty.val;
                    self.bus.data.output.next = self.widget_b.bus.data.output.val;
                    self.widget_b.bus.data.read.next = self.bus.data.read.val;
                }
            }

            fn connect(&mut self) {
                self.bus.cmd.underflow.connect();
                self.bus.cmd.almost_empty.connect();
                self.bus.cmd.empty.connect();
                self.bus.cmd.output.connect();
                self.widget_a.bus.cmd.read.connect();
                self.widget_a.bus.data.read.connect();
                self.widget_b.bus.cmd.read.connect();
                self.widget_b.bus.data.read.connect();
                self.widget_a.clock.connect();
                self.widget_b.clock.connect();
            }
        }

        impl Block for UUT {
            fn connect_all(&mut self) {
                self.connect();
                self.select.connect_all();
                self.widget_a.connect_all();
                self.widget_b.connect_all();
                self.bus.connect_all();
                self.clock.connect_all();
            }

            fn update_all(&mut self) {
                self.update();
                self.select.update_all();
                self.widget_a.update_all();
                self.widget_b.update_all();
                self.bus.update_all();
                self.clock.update_all();
            }

            fn has_changed(&self) -> bool {
                self.select.has_changed() ||
                    self.widget_a.has_changed() ||
                    self.widget_b.has_changed() ||
                    self.bus.has_changed() ||
                    self.clock.has_changed()
            }

            fn accept(&self, name: &str, probe: &mut dyn Probe) {
                probe.visit_start_scope(name, self);
                self.select.accept("select", probe);
                self.widget_a.accept("widget_a", probe);
                self.widget_b.accept("widget_b", probe);
                self.bus.accept("bus", probe);
                self.clock.accept("clock", probe);
                probe.visit_end_scope(name, self);
            }
        }

        let mut uut = UUT::default();
        uut.clock.connect();
        uut.bus.cmd.read.connect();
        uut.bus.data.read.connect();
        uut.select.connect();
        uut.connect_all();
//        check_connected(&uut);
        let mut defines = ModuleDefines::default();
        uut.accept("uut", &mut defines);
        defines.defines();

    }


    #[test]
    fn test_write_modules() {
        #[derive(Clone, Default, Debug)]
        struct StrobePair {
            pub a_strobe: Strobe<4>,
            pub b_strobe: Strobe<6>,
            pub clock: Signal<In, Clock>,
            pub enable: Signal<In, Bit>,
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
            fn connect_all(&mut self) {
                self.connect();
                self.a_strobe.connect_all();
                self.b_strobe.connect_all();
                self.clock.connect_all();
                self.enable.connect_all();
            }

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

        let mut uut = StrobePair::default();
        // Simulate 100 clock cycles
        //uut.enable.next = true;
        println!("Starting");
        uut.clock.connect();
        uut.enable.connect();
        uut.connect_all();
        check_connected(&uut);
        let mut defines = ModuleDefines::default();
        uut.accept("uut", &mut defines);
        defines.defines();
    }
}
