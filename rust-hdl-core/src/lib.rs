mod atom;
mod bits;
mod bitvec;
mod block;
mod check_connected;
mod clock;
mod constant;
mod dff;
mod direction;
mod logic;
mod module_defines;
mod named_path;
mod probe;
mod shortbitvec;
mod signal;
mod simulate;
mod struct_valued;
mod synth;

#[cfg(test)]
mod tests {
    use crate::bits::{Bit, Bits, clog2};
    use crate::block::Block;
    use crate::check_connected::check_connected;
    use crate::clock::Clock;
    use crate::constant::Constant;
    use crate::dff::DFF;
    use crate::direction::{In, Out, Local};
    use crate::logic::Logic;
    use crate::module_defines::ModuleDefines;
    use crate::probe::Probe;
    use crate::signal::Signal;
    use crate::simulate::simulate;
    use rust_hdl_macros::LogicBlock;
    use rust_hdl_macros::LogicInterface;
    use crate::synth::Synth;

    #[derive(Clone, Debug, LogicBlock)]
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
                self.counter.d.next = self.counter.q.val + self.strobe_incr.val;
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
    fn test_enum_state() {
        #[derive(Copy, Clone, Debug, PartialEq)]
        enum MyState {
            Init,
            Start,
            Running,
            Paused,
            Stopped
        }

        impl Default for MyState {
            fn default() -> Self {
                Self::Init
            }
        }

        impl Synth for MyState {
            const BITS: usize = clog2(5);
            const ENUM_TYPE: bool = true;
            const TYPE_NAME: &'static str = "MyState";
            fn name(ndx: usize) -> &'static str {
                match ndx {
                    0 => "Init",
                    1 => "Start",
                    2 => "Running",
                    3 => "Paused",
                    4 => "Stopped",
                    _ => ""
                }
            }
        }

        #[derive(Clone, Debug, LogicBlock)]
        struct StateMachine {
            pub clock: Signal<In, Clock>,
            pub advance: Signal<In, Bit>,
            state: DFF<MyState>
        }

        impl StateMachine {
            pub fn new() -> StateMachine {
                StateMachine {
                    clock: Signal::default(),
                    advance: Signal::default(),
                    state: DFF::new(MyState::Init),
                }
            }
        }

        impl Logic for StateMachine {
            fn update(&mut self) {
                self.state.clk.next = self.clock.val;

                if self.advance.val {
                    match self.state.q.val {
                        MyState::Init => {
                            self.state.d.next = MyState::Start
                        }
                        MyState::Start => {
                            self.state.d.next = MyState::Running
                        }
                        MyState::Running => {
                            self.state.d.next = MyState::Paused
                        }
                        MyState::Paused => {
                            self.state.d.next = MyState::Stopped
                        }
                        MyState::Stopped => {
                            self.state.d.next = MyState::Init
                        }
                    }
                }
            }

            fn connect(&mut self) {
                self.state.d.connect();
                self.state.clk.connect();
            }
        }

        let mut uut = StateMachine::new();
        println!("Starting");
        uut.clock.connect();
        uut.advance.connect();
        uut.connect_all();
        check_connected(&uut);
        for clock in 0..10 {
            uut.clock.next = Clock(clock % 2 == 0);
            uut.advance.next = true;
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            println!("State {:?}", uut.state.q.val);
        }
        let mut defines = ModuleDefines::default();
        uut.accept("uut", &mut defines);
        defines.defines();
    }

    #[test]
    fn test_write_modules() {
        #[derive(Clone, Debug, LogicBlock)]
        struct StrobePair {
            pub clock: Signal<In, Clock>,
            pub enable: Signal<In, Bit>,
            a_strobe: Strobe<4>,
            b_strobe: Strobe<6>,
            increment: Constant<Bits<6>>,
            local: Signal<Local, Bit>,
        }

        impl StrobePair {
            pub fn new() -> StrobePair {
                Self {
                    a_strobe: Strobe::default(),
                    b_strobe: Strobe::default(),
                    clock: Signal::default(),
                    enable: Signal::default(),
                    increment: Constant::new(32_usize.into()),
                    local: Signal::default(),
                }
            }
        }

        impl Logic for StrobePair {
            fn update(&mut self) {}
            fn connect(&mut self) {
                self.a_strobe.enable.connect();
                self.b_strobe.enable.connect();
                self.a_strobe.clock.connect();
                self.b_strobe.clock.connect();
                self.local.connect();
            }
        }

        let mut uut = StrobePair::new();
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
