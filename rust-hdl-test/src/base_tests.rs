#[cfg(test)]
mod tests {
    use rust_hdl_core::prelude::*;
    use rust_hdl_macros::hdl_gen;
    use rust_hdl_macros::LogicBlock;
    use rust_hdl_widgets::dff::DFF;
    use rust_hdl_widgets::strobe::Strobe;
    use rust_hdl_alchitry_cu::pins::Mhz100;
    use std::marker::PhantomData;

    #[derive(Copy, Clone, Debug, PartialEq)]
    enum MyState {
        Init,
        Start,
        Running,
        Paused,
        Stopped,
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
                _ => "",
            }
        }
        fn vcd(self) -> VCDValue {
            match self {
                MyState::Init => VCDValue::String("Init".into()),
                MyState::Start => VCDValue::String("Start".into()),
                MyState::Running => VCDValue::String("Running".into()),
                MyState::Paused => VCDValue::String("Paused".into()),
                MyState::Stopped => VCDValue::String("Stopped".into()),
            }
        }
        fn verilog(self) -> VerilogLiteral {
            match self {
                MyState::Init => 0_u32.into(),
                MyState::Start => 1_u32.into(),
                MyState::Running => 2_u32.into(),
                MyState::Paused => 3_u32.into(),
                MyState::Stopped => 4_u32.into(),
            }
        }
    }

    #[test]
    fn test_visit_version() {
        make_domain!(F, 100);
        let mut uut: Strobe<F, 32> = Strobe::new(10.0);
        // Simulate 100 clock cycles
        uut.enable.next = true.into();
        println!("Starting");
        uut.clock.connect();
        uut.enable.connect();
        uut.connect_all();
        check_connected(&uut);
        let mut strobe_count = 0;
        for clock in 0..10_000_000 {
            uut.clock.next = Clock(clock % 2 == 0).into();
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            if uut.strobe.val().raw() {
                strobe_count += 1;
            }
        }
        assert_eq!(strobe_count, 1_000_000);
    }

    #[test]
    fn test_enum_state() {
        #[derive(Copy, Clone, Debug, PartialEq)]
        enum MyState {
            Init,
            Start,
            Running,
            Paused,
            Stopped,
        }

        impl<D: Domain> Into<Tagged<MyState, D>> for MyState {
            fn into(self) -> Tagged<MyState, D> {
                Tagged(self, PhantomData)
            }
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
                    _ => "",
                }
            }
            fn vcd(self) -> VCDValue {
                match self {
                    MyState::Init => VCDValue::String("Init".into()),
                    MyState::Start => VCDValue::String("Start".into()),
                    MyState::Running => VCDValue::String("Running".into()),
                    MyState::Paused => VCDValue::String("Paused".into()),
                    MyState::Stopped => VCDValue::String("Stopped".into()),
                }
            }
            fn verilog(self) -> VerilogLiteral {
                match self {
                    MyState::Init => 0_u32.into(),
                    MyState::Start => 1_u32.into(),
                    MyState::Running => 2_u32.into(),
                    MyState::Paused => 3_u32.into(),
                    MyState::Stopped => 4_u32.into(),
                }
            }
        }

        #[derive(Clone, Debug, LogicBlock)]
        struct StateMachine {
            pub clock: Signal<In, Clock, Async>,
            pub advance: Signal<In, Bit, Async>,
            state: DFF<MyState, Async>,
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
            #[hdl_gen]
            fn update(&mut self) {
                self.state.clk.next = self.clock.val();

                if self.advance.val().raw() {
                    match self.state.q.val().raw() {
                        MyState::Init => self.state.d.next = MyState::Start.into(),
                        MyState::Start => self.state.d.next = MyState::Running.into(),
                        MyState::Running => self.state.d.next = MyState::Paused.into(),
                        MyState::Paused => self.state.d.next = MyState::Stopped.into(),
                        MyState::Stopped => self.state.d.next = MyState::Init.into(),
                    }
                }
            }
        }

        let mut uut = StateMachine::new();
        println!("Starting");
        uut.clock.connect();
        uut.advance.connect();
        uut.connect_all();
        check_connected(&uut);
        for clock in 0..10 {
            uut.clock.next = Clock(clock % 2 == 0).into();
            uut.advance.next = true.into();
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            println!("State {:?}", uut.state.q.val());
        }
        println!("{}", generate_verilog(&uut));
    }

    #[test]
    fn test_write_modules() {
        #[derive(Clone, Debug, LogicBlock)]
        struct StrobePair {
            pub clock: Signal<In, Clock, Mhz100>,
            pub enable: Signal<In, Bit, Mhz100>,
            a_strobe: Strobe<Mhz100, 32>,
            b_strobe: Strobe<Mhz100, 32>,
            increment: Constant<Bits<6>>,
            local: Signal<Local, Bit, Mhz100>,
        }

        impl StrobePair {
            pub fn new() -> StrobePair {
                Self {
                    a_strobe: Strobe::new(10.0),
                    b_strobe: Strobe::new(10.0),
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
        println!("{}", generate_verilog(&uut));
    }

    #[test]
    fn test_async() {
        #[derive(LogicBlock, Clone, Default)]
        struct Semaphore {
            pub push: Signal<In, Bit, Mhz100>,
            pub pop: Signal<In, Bit, Mhz100>,
            pub clk: Signal<In, Clock, Mhz100>,
            pub empty: Signal<Out, Bit, Mhz100>,
            pub full: Signal<Out, Bit, Mhz100>,
            will_read: Signal<Local, Bit, Mhz100>,
            will_write: Signal<Local, Bit, Mhz100>,
            count: DFF<Bits<4>, Mhz100>,
        }

        impl Logic for Semaphore {
            #[hdl_gen]
            fn update(&mut self) {
                self.count.clk.next = self.clk.val();
                self.count.d.next = self.count.q.val();

                self.will_read.next = !self.empty.val() & self.pop.val();
                self.will_write.next = !self.full.val() & self.push.val();

                if (self.will_read.val() & !self.will_write.val()).raw() {
                    self.count.d.next = self.count.q.val() - 1_u32;
                } else if (self.will_write.val() & !self.will_read.val()).raw() {
                    self.count.d.next = self.count.q.val() + 1_u32;
                }

                self.full.next = (self.count.q.val() == 15_u32).into();
                self.empty.next = (self.count.q.val() == 0_u32).into();
            }
        }

        let mut uut = Semaphore::default();
        // Simulate 100 clock cycles
        uut.clk.connect();
        uut.pop.connect();
        uut.push.connect();
        uut.connect_all();
        check_connected(&uut);
    }

    /*
    pub async fn fifo_vector_feeder<T: Synthesizable>(
        clock: Clock,
        mut writer: FIFOWriterClient<T>,
        data: Vec<T>,
        prob_pause: f64,
        pause_len: u64,
    ) -> Result<(), HDLError> {
        writer.write.set(false);
        clock.next_negedge().await;
        for datum in data {
            while writer.full.get() {
                writer.write.set(false);
                clock.next_negedge().await;
            }
            if rand::thread_rng().gen::<f64>() < prob_pause {
                writer.write.set(false);
                clock.delay_negedge(pause_len).await;
            }
            writer.input.set(datum);
            writer.write.set(true);
            clock.next_negedge().await;
        }
        writer.write.set(false);
        if writer.overflow.get() {
            Err(HDLError::TestBenchFailed("FIFO overflowed".to_string()))
        } else {
            Ok(())
        }
    }
     */

    #[derive(LogicBlock)]
    struct Circuit {
        x: Signal<In, Bits<32>, Mhz100>,
        pub strobe: Strobe<Mhz100, 32>,
    }

    impl Logic for Circuit {
        #[hdl_gen]
        fn update(&mut self) {}
    }

    fn sample_func(mut ep: Sim<Circuit>) -> rust_hdl_core::simulate::Result<()> {
        // Need an initialization stage...
        // Get the initial circuit - this must be serviced first.
        println!("Initialize TB 1");
        let x = ep.init()?;
        println!("Hello from TB 1");
        let mut x = ep.wait(0, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        x.x.next = 42_u32.into();
        let mut x = ep.wait(100, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        x.x.next = 100_u32.into();
        let x = ep.watch(|m| m.x.val() == 89, x)?;
        println!("Hello from TB1 where x value is {:?}", x.x.next);
        let x = ep.wait(250, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        // This is called last
        ep.done(x)?;
        println!("TB 1 done");
        Ok(())
    }

    fn sample_func2(mut ep: Sim<Circuit>) -> rust_hdl_core::simulate::Result<()> {
        let x = ep.init()?;
        println!("Hello from TB 2");
        let mut x = ep.wait(125, x)?;
        println!("Hello from TB 2 at time {}", ep.time());
        x.x.next = 89_u32.into();
        ep.done(x)?;
        println!("TB 2 done");
        Ok(())
    }

    #[test]
    fn test_tb() {
        let mut sim = Simulation::new();
        sim.add_clock(5, |x: &mut Circuit| {
            x.strobe.clock.next = !x.strobe.clock.val()
        });
        sim.add_testbench(sample_func);
        sim.add_testbench(sample_func2);
        let mut x = Circuit {
            x: Signal::default(),
            strobe: Strobe::new(10.0),
        };
        x.x.connect();
        x.strobe.clock.connect();
        x.strobe.enable.connect();
        x.connect_all();
        sim.run(x, 400).unwrap();
    }
}
