use rust_hdl::core::check_error::{CheckError, LogicLoop};
use rust_hdl::core::prelude::*;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
enum FooState {
    Init,
    Start,
    Running,
    Paused,
    Stopped,
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use rust_hdl::core::prelude::*;
    use rust_hdl::simple_sim;
    use rust_hdl::widgets::prelude::*;

    #[derive(Copy, Clone, Debug, PartialEq, LogicState)]
    enum MyState {
        Init,
        Start,
        Running,
        Paused,
        Stopped,
    }

    #[test]
    fn test_visit_version() {
        let mut uut: Strobe<32> = Strobe::new(100, 10.0);
        // Simulate 100 clock cycles
        uut.enable.next = true;
        println!("Starting");
        uut.connect_all();
        check_all(&uut).unwrap();
        let mut strobe_count = 0;
        for clock in 0..10_000_000 {
            uut.clock.next = (clock % 2 == 0).into();
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            if uut.strobe.val() {
                strobe_count += 1;
            }
        }
        assert_eq!(strobe_count, 1_000_000);
    }

    #[test]
    fn test_enum_state() {
        #[derive(Copy, Clone, Debug, PartialEq, LogicState)]
        enum MyState {
            Init,
            Start,
            Running,
            Paused,
            Stopped,
        }

        #[derive(Clone, Debug, LogicBlock)]
        struct StateMachine {
            pub clock: Signal<In, Clock>,
            pub advance: Signal<In, Bit>,
            state: DFF<MyState>,
        }

        impl StateMachine {
            pub fn new() -> StateMachine {
                StateMachine {
                    clock: Signal::default(),
                    advance: Signal::default(),
                    state: Default::default(),
                }
            }
        }

        impl Logic for StateMachine {
            #[hdl_gen]
            fn update(&mut self) {
                dff_setup!(self, clock, state);

                if self.advance.val() {
                    match self.state.q.val() {
                        MyState::Init => self.state.d.next = MyState::Start,
                        MyState::Start => self.state.d.next = MyState::Running,
                        MyState::Running => self.state.d.next = MyState::Paused,
                        MyState::Paused => self.state.d.next = MyState::Stopped,
                        MyState::Stopped => self.state.d.next = MyState::Init,
                        _ => self.state.d.next = MyState::Init,
                    }
                }
            }
        }

        let mut uut = StateMachine::new();
        println!("Starting");
        uut.connect_all();
        check_all(&uut).unwrap();
        for clock in 0..10 {
            uut.clock.next = Clock::from(clock % 2 == 0).into();
            uut.advance.next = true;
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            println!("State {:?}", uut.state.q.val());
        }
        println!("{}", generate_verilog(&uut));
    }

    #[test]
    fn test_write_modules() {
        const MHZ100: u64 = 100_000_000;

        #[derive(Clone, Debug, LogicBlock)]
        struct StrobePair {
            pub clock: Signal<In, Clock>,
            pub enable: Signal<In, Bit>,
            a_strobe: Strobe<32>,
            b_strobe: Strobe<32>,
            increment: Constant<Bits<6>>,
            local: Signal<Local, Bit>,
        }

        impl StrobePair {
            pub fn new() -> StrobePair {
                Self {
                    a_strobe: Strobe::new(10_000_000, 10.0),
                    b_strobe: Strobe::new(10_000_000, 10.0),
                    clock: Signal::default(),
                    enable: Signal::default(),
                    increment: Constant::new(32.into()),
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
        uut.connect_all();
    }

    const MHZ100: u64 = 100_000_000;

    #[derive(LogicBlock)]
    struct Circuit {
        x: Signal<In, Bits<32>>,
        pub strobe: Strobe<32>,
    }

    impl Logic for Circuit {
        #[hdl_gen]
        fn update(&mut self) {}
    }

    fn sample_func(mut ep: Sim<Circuit>) -> rust_hdl::core::simulate::Result<()> {
        // Need an initialization stage...
        // Get the initial circuit - this must be serviced first.
        println!("Initialize TB 1");
        let x = ep.init()?;
        println!("Hello from TB 1");
        let mut x = ep.wait(0, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        x.x.next = 42.into();
        let mut x = ep.wait(100, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        x.x.next = 100.into();
        let x = ep.watch(|m| m.x.val() == 89, x)?;
        println!("Hello from TB1 where x value is {:?}", x.x.next);
        let x = ep.wait(250, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        // This is called last
        ep.done(x)?;
        println!("TB 1 done");
        Ok(())
    }

    fn sample_func2(mut ep: Sim<Circuit>) -> rust_hdl::core::simulate::Result<()> {
        let x = ep.init()?;
        println!("Hello from TB 2");
        let mut x = ep.wait(125, x)?;
        println!("Hello from TB 2 at time {}", ep.time());
        x.x.next = 89.into();
        ep.done(x)?;
        println!("TB 2 done");
        Ok(())
    }

    #[test]
    fn test_macro_tb() {
        let mut x = Circuit {
            x: Signal::default(),
            strobe: Strobe::new(1000, 10.0),
        };
        let mut sim = simple_sim!(Circuit, strobe.clock, 5, ep, {
            let x = ep.init()?;
            println!("Hello from macro driven simple simulation");
            let mut x = ep.wait(125, x)?;
            println!(
                "Hello from macro driven simple simluation at time {}",
                ep.time()
            );
            x.x.next = 89.into();
            ep.done(x)
        });
        x.strobe.clock.connect();
        x.strobe.enable.connect();
        x.connect_all();
        sim.run(Box::new(x), 400).unwrap();
    }

    #[test]
    fn test_tb() {
        let mut sim = Simulation::new();
        sim.add_clock(5, |x: &mut Box<Circuit>| {
            x.strobe.clock.next = !x.strobe.clock.val()
        });
        sim.add_testbench(sample_func);
        sim.add_testbench(sample_func2);
        let mut x = Circuit {
            x: Signal::default(),
            strobe: Strobe::new(1000, 10.0),
        };
        x.x.connect();
        x.strobe.clock.connect();
        x.strobe.enable.connect();
        x.connect_all();
        sim.run(Box::new(x), 400).unwrap();
    }

    #[test]
    fn test_array_assignments() {
        #[derive(LogicBlock, Default)]
        struct TestObj {
            channel: [Signal<Local, Bit>; 8],
            sum: Signal<Local, Bit>,
        }

        impl Logic for TestObj {
            #[hdl_gen]
            fn update(&mut self) {
                self.sum.next = self.channel[0].val() | self.channel[1].val();
            }
        }

        let mut x = TestObj::default();
        for i in 0..8 {
            x.channel[i].connect();
        }
        x.connect_all();
        let vlog = generate_verilog(&x);
        yosys_validate("test_obj1", &vlog).unwrap();
    }

    #[test]
    fn test_array_assignments_with_bitcast() {
        #[derive(LogicBlock, Default)]
        struct TestObj {
            channel: [Signal<Local, Bit>; 8],
            sum: Signal<Local, Bits<4>>,
        }

        impl Logic for TestObj {
            #[hdl_gen]
            fn update(&mut self) {
                self.sum.next = bit_cast::<4, 1>(self.channel[0].val().into())
                    | (bit_cast::<4, 1>(self.channel[1].val().into()) << 1);
            }
        }

        let mut x = TestObj::default();
        for i in 0..8 {
            x.channel[i].connect();
        }
        x.connect_all();
        let vlog = generate_verilog(&x);
        yosys_validate("test_obj", &vlog).unwrap();
    }
}

#[test]
fn test_local_logic() {
    #[derive(LogicBlock, Default)]
    struct EvenOddTest {
        pub signal: Signal<In, Bit>,
        pub is_odd: Signal<Out, Bit>,
        pub is_even: Signal<Out, Bit>,
        pub local: Signal<Local, Bits<3>>,
    }

    impl Logic for EvenOddTest {
        #[hdl_gen]
        fn update(&mut self) {
            self.local.next = bit_cast::<3, 1>(self.signal.val().into());
            self.local.next = self.local.val() + 1;
            self.is_odd.next = self.local.val().get_bit(0);
            self.local.next = self.local.val() + 1;
            self.is_even.next = self.local.val().get_bit(1);
        }
    }

    let mut uut = EvenOddTest::default();
    uut.signal.connect();
    uut.connect_all();
    yosys_validate("even_odd_test", &generate_verilog(&uut)).unwrap();

    let mut sim = Simulation::new();
    sim.add_testbench(move |mut sim: Sim<EvenOddTest>| {
        let mut x = sim.init()?;
        x.signal.next = true;
        x = sim.wait(10, x)?;
        sim_assert!(sim, x.local.val() == 3, x);
        x.signal.next = false;
        x = sim.wait(10, x)?;
        sim_assert!(sim, x.local.val() == 2, x);
        x.signal.next = true;
        x = sim.wait(10, x)?;
        sim_assert!(sim, x.local.val() == 3, x);
        x.signal.next = false;
        x = sim.wait(10, x)?;
        sim_assert!(sim, x.local.val() == 2, x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 100, &vcd_path!("even_odd_test.vcd"))
        .unwrap()
}

#[test]
fn test_local_logic_loop_detection() {
    #[derive(LogicBlock, Default)]
    struct LoopTest {
        pub signal: Signal<In, Bit>,
        pub is_odd: Signal<Out, Bit>,
        pub is_even: Signal<Out, Bit>,
        pub foo: Signal<Local, Bits<3>>,
    }

    impl Logic for LoopTest {
        #[hdl_gen]
        fn update(&mut self) {
            self.foo.next = self.foo.val() + self.signal.val();
            self.is_odd.next = self.foo.val().get_bit(0);
            self.foo.next = self.foo.val() + 1;
            self.is_even.next = self.foo.val().get_bit(1);
        }
    }

    let mut uut = LoopTest::default();
    uut.signal.connect();
    uut.connect_all();
    let e = check_all(&uut).expect_err("Loop should have been found");
    if let CheckError::LogicLoops(m) = e {
        assert!(m.contains(&LogicLoop {
            path: "uut".to_string(),
            name: "foo".to_string()
        }))
    } else {
        panic!("Error mismatch on loop detector")
    }
}
