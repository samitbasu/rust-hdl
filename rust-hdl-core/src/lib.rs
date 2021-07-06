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


    #[test]
    fn test_async() {
        #[derive(LogicBlock, Clone, Default)]
        struct Semaphore {
            pub push: Signal<In, Bit>,
            pub pop: Signal<In, Bit>,
            pub clk: Signal<In, Clock>,
            pub empty: Signal<Out, Bit>,
            pub full: Signal<Out, Bit>,
            will_read: Signal<Local, Bit>,
            will_write: Signal<Local ,Bit>,
            count: DFF<Bits<4>>
        }

        impl Logic for Semaphore {
            fn update(&mut self) {
                self.count.clk.next = self.clk.val;
                self.count.d.next = self.count.q.val;

                self.will_read.next = !self.empty.val && self.pop.val;
                self.will_write.next = !self.full.val && self.push.val;

                if self.will_read.val && !self.will_write.val {
                    self.count.d.next = self.count.q.val - 1;
                } else if self.will_write.val && !self.will_read.val {
                    self.count.d.next = self.count.q.val + 1;
                }

                self.full.next = self.count.q.val == 15_usize.into();
                self.empty.next = self.count.q.val == 0_usize.into();
            }

            fn connect(&mut self) {
                self.count.clk.connect();
                self.count.d.connect();
                self.will_write.connect();
                self.will_read.connect();
                self.full.connect();
                self.empty.connect();
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

    use crossbeam::channel::{Sender, Receiver, bounded, RecvError, SendError};

    #[derive(Copy, Clone, Debug, PartialEq)]
    enum SimError {
        SimTerminated
    }

    impl From<RecvError> for SimError {
        fn from(x: RecvError) -> Self {
            SimError::SimTerminated
        }
    }

    impl<T> From<SendError<T>> for SimError {
        fn from(x: SendError<T>) -> Self {
            SimError:: SimTerminated
        }
    }

    type Result<T> = std::result::Result<T, SimError>;

    struct Circuit {
        x: i32,
        strobe: Strobe<4>,
    }

    enum TriggerType<T> {
        Never,
        Time(u64),
        Function(Box<dyn Fn(&T) -> bool + Send>)
    }

    struct Message<T> {
        id: usize,
        kind: TriggerType<T>,
        circuit: T,
    }

    struct Worker<T> {
        id: usize,
        channel_to_worker: Sender<Message<T>>,
        kind: TriggerType<T>,
    }

    struct Simulation<T> {
        workers: Vec<Worker<T>>,
        recv: Receiver<Message<T>>,
        channel_to_sim: Sender<Message<T>>,
        time: u64,
    }

    struct Endpoint<T> {
        idx: usize,
        time: u64,
        to_sim: Sender<Message<T>>,
        from_sim: Receiver<Message<T>>
    }

    impl<T> Simulation<T> {
        pub fn new() -> Simulation<T> {
            let (send, recv) = bounded(0);
            Self {
                workers: vec![],
                recv,
                channel_to_sim: send,
                time: 0,
            }
        }
        pub fn endpoint(&mut self) -> Endpoint<T> {
            let (send_to_worker,
                recv_from_sim_to_worker) = bounded(0);
            let id = self.workers.len();
            let worker = Worker {
                id,
                channel_to_worker: send_to_worker,
                kind: TriggerType::Never
            };
            self.workers.push(worker);
            Endpoint {
                idx: id,
                to_sim: self.channel_to_sim.clone(),
                from_sim: recv_from_sim_to_worker,
                time: 0
            }
        }
        fn dispatch(&mut self, idx: usize, x: T) -> Result<T> {
            let worker = &mut self.workers[idx];
            println!("Sending circuit to worker {}", worker.id);
            worker.channel_to_worker.send(Message {
                id: worker.id,
                kind: TriggerType::Time(self.time),
                circuit: x
            });
            println!("Waiting for circuit to return");
            let x = self.recv.recv()?;
            println!("Received circuit from worker {}", x.id);
            match &x.kind {
                TriggerType::Never => {
                    println!("Worker does not want to be re-awoken")
                }
                TriggerType::Time(t) => {
                    println!("Worker would like to be notified at time {}", t);
                }
                TriggerType::Function(_) => {
                    println!("Worker would like to be notified when function returns true");
                }
            }
            worker.kind = x.kind;
            Ok(x.circuit)
        }
        pub fn run(&mut self, mut x: T, max_time: u64) -> Result<()> {
            // First initialize the workers.
            for id in 0..self.workers.len() {
                x = self.dispatch(id, x)?;
            }
            // Next run until we have no one else waiting
            while self.time < max_time {
                let mut min_time = !0_u64;
                let mut min_idx = 0;
                for worker in self.workers.iter() {
                    match &worker.kind {
                        TriggerType::Never => {}
                        TriggerType::Time(t) => {
                            if *t < min_time {
                                min_time = *t;
                                min_idx = worker.id;
                            }
                        }
                        TriggerType::Function(watch) => {
                            if watch(&x) {
                                min_idx = worker.id;
                                min_time = self.time;
                                break;
                            }
                        }
                    }
                }
                if min_time == !0 {
                    break;
                }
                println!("Updating time to {}", min_time);
                self.time = min_time;
                x = self.dispatch(min_idx, x)?;
            }
            println!("No more work to do... ending simulation");
            self.workers.clear();
            Ok(())
        }
    }

    impl<T> Endpoint<T> {
        pub fn init(&self) -> Result<T> {
            Ok(self.from_sim.recv()?.circuit)
        }
        pub fn watch<S>(&mut self, check: S, x: T) -> Result<T>
            where S: Fn(&T) -> bool + Send + 'static {
            self.to_sim.send(Message {
                id: self.idx,
                kind: TriggerType::Function(Box::new(check)),
                circuit: x,
            })?;
            let t = self.from_sim.recv()?;
            Ok(t.circuit)
        }
        pub fn wait(&mut self, delta: u64, x: T) -> Result<T> {
            self.to_sim.send(Message {
                id: self.idx,
                kind: TriggerType::Time(delta + self.time),
                circuit: x,
            })?;
            let t = self.from_sim.recv()?;
            if let TriggerType::Time(t0) = t.kind {
                self.time = t0;
            }
            Ok(t.circuit)
        }
        pub fn done(&self, x: T) -> Result<()> {
            self.to_sim.send(Message {
                id: self.idx,
                kind: TriggerType::Never,
                circuit: x,
            })?;
            Ok(())
        }
        pub fn time(&self) -> u64 {
            self.time
        }
    }

    fn sample_func(mut ep: Endpoint<Circuit>) -> Result<()> {
        // Need an initialization stage...
        // Get the initial circuit - this must be serviced first.
        println!("Initialize TB 1");
        let x = ep.init()?;
        println!("Hello from TB 1");
        let mut x = ep.wait(0, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        x.x = 42;
        let mut x = ep.wait(100, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        x.x = 100;
        let mut x = ep.watch(|m| m.x == 89, x)?;
        println!("Hello from TB1 where x value is {}", x.x);
        let mut x = ep.wait(250, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        // This is called last
        ep.done(x)?;
        println!("TB 1 done");
        Ok(())
    }

    fn sample_func2(mut ep: Endpoint<Circuit>) -> Result<()> {
        let x = ep.init()?;
        println!("Hello from TB 2");
        let mut x = ep.wait(125, x)?;
        println!("Hello from TB 2 at time {}", ep.time());
        x.x = 88;
        ep.done(x)?;
        println!("TB 2 done");
        Ok(())
    }

    #[test]
    fn test_tb() {
        let mut sim = Simulation::new();
        let ep1 = sim.endpoint();
        let sf1 = std::thread::spawn(move || sample_func(ep1));
        let ep2 = sim.endpoint();
        let sf2 = std::thread::spawn(move || sample_func2(ep2));
        let x = Circuit{ x: 0 , strobe: Strobe::default()};
        sim.run(x, 1000);
        sf1.join();
        sf2.join();
    }

}