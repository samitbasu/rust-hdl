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
    use rust_hdl_macros::LogicBlock;
    use rust_hdl_macros::LogicInterface;

    #[derive(Clone, Debug, LogicBlock)]
    struct Strobe<const N: usize> {
        pub enable: Signal<In, Bit>,
        pub strobe: Signal<Out, Bit>,
        pub clock: Signal<In, Clock>,
//        pub strobe_incr: Constant<Bits<N>>,
        counter: DFF<Bits<N>>,
    }

    impl<const N: usize> Default for Strobe<N> {
        fn default() -> Self {
            Self {
                enable: Signal::default(),
                strobe: Signal::<Out, Bit>::new_with_default(false),
                clock: Signal::default(),
//                strobe_incr: Constant::new(1_usize.into()),
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

        #[derive(Clone, Debug, Default, LogicInterface)]
        struct MyBus {
            pub data: FIFORead<8>,
            pub cmd: FIFORead<3>,
        }

        #[derive(Clone, Debug, Default, LogicInterface)]
        struct FIFORead<const D: usize> {
            pub read: Signal<In, Bit>,
            pub output: Signal<Out, Bits<D>>,
            pub empty: Signal<Out, Bit>,
            pub almost_empty: Signal<Out, Bit>,
            pub underflow: Signal<Out, Bit>,
        }

        #[derive(Clone, Debug, Default, LogicBlock)]
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

        #[derive(Clone, Debug, Default, LogicBlock)]
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
        #[derive(Clone, Default, Debug, LogicBlock)]
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
