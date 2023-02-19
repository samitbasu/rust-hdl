use crossbeam::channel::{bounded, Receiver, Sender};
use crossbeam::channel::{RecvError, SendError};

use crate::block::Block;
use crate::check_error::{check_all, CheckError};
use crate::vcd_probe::{write_vcd_change, write_vcd_dump, write_vcd_header};
use std::io::Write;
use std::thread::JoinHandle;

/// Update changes to a circuit until it stabilizes
///
/// # Arguments
///
/// * `uut` - reference to the circuit - must implement the [Block] trait
/// * `max_iters` - the maximum number of iterations to try and stabilize the circuit
///
/// Returns `true` if the circuit stabilizes, and `false` if not.  Generally you won't
/// need this function directly.
pub fn simulate<B: Block>(uut: &mut B, max_iters: usize) -> bool {
    for _ in 0..max_iters {
        uut.update_all();
        if !uut.has_changed() {
            return true;
        }
    }
    false
}

#[derive(Clone, Debug, PartialEq)]
/// The error type returned by a simulation
pub enum SimError {
    /// The simulation terminated prematurely (i.e., something went wrong)
    SimTerminated,
    /// The simulation reached the maximum allowed time for the simulation
    MaxTimeReached,
    /// The simulation halted - usually this means an assertion failed
    SimHalted,
    /// The circuit failed to converge.  This means the logic has some issue (like an oscillation).
    FailedToConverge,
    /// Something went wrong with the circuit check (either a missing connection or other issue, like a latching write).
    Check(CheckError),
    /// The simulation panicked.  This usually means `.unwrap` was called on a result in the testbench.
    SimPanic,
}

impl From<CheckError> for SimError {
    fn from(x: CheckError) -> Self {
        SimError::Check(x)
    }
}

impl From<RecvError> for SimError {
    fn from(_x: RecvError) -> Self {
        SimError::SimTerminated
    }
}

impl<T> From<SendError<T>> for SimError {
    fn from(_x: SendError<T>) -> Self {
        SimError::SimTerminated
    }
}

/// Result type used by the simulation routines.
pub type Result<T> = std::result::Result<T, SimError>;

enum TriggerType<T> {
    Never,
    Time(u64),
    Function(Box<dyn Fn(&T) -> bool + Send>),
    Clock(u64),
    Halt,
}

struct Message<T> {
    kind: TriggerType<T>,
    circuit: Box<T>,
}

enum MessageOrPanic<T> {
    Message(Message<T>),
    Panic,
}

struct Worker<T> {
    id: usize,
    channel_to_worker: Sender<Message<T>>,
    kind: TriggerType<T>,
}

/// The [CustomLogicFn] is a boxed function that can be used to implement
/// things (like tri-state buffers or open collector shared busses) that
/// are otherwise difficult or impossible to model.
pub type CustomLogicFn<T> = Box<dyn Fn(&mut T) -> ()>;

/// This type represents a simulation over a circuit `T`.   To simulate
/// a circuit, you will need to construct one of these structs.
pub struct Simulation<T> {
    workers: Vec<Worker<T>>,
    recv: Receiver<MessageOrPanic<T>>,
    channel_to_sim: Sender<MessageOrPanic<T>>,
    time: u64,
    testbenches: Vec<JoinHandle<Result<()>>>,
    custom_logic: Vec<CustomLogicFn<T>>,
}

/// The `Sim` struct is used to communicate with a simulation.  Every testbench
/// will be provided with a copy of this struct, and will use it to communicate
/// with the core simulation.
pub struct Sim<T> {
    time: u64,
    to_sim: Sender<MessageOrPanic<T>>,
    from_sim: Receiver<Message<T>>,
}

struct NextTime {
    time: u64,
    idx: usize,
    clocks_only: bool,
    halted: bool,
}

impl<T: Send + 'static + Block> Default for Simulation<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + 'static + Block> Simulation<T> {
    /// Construct a simulation struct
    pub fn new() -> Simulation<T> {
        let (send, recv) = bounded(0);
        Self {
            workers: vec![],
            recv,
            channel_to_sim: send,
            time: 0,
            testbenches: vec![],
            custom_logic: vec![],
        }
    }
    /// Add a clock function to the simulation
    ///
    /// # Arguments
    ///
    /// * `interval` - the number of picoseconds between calls to the clock closure
    /// * `clock_fn` - a closure to change the clock state of the circuit
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rust_hdl_private_core::prelude::*;
    ///
    /// #[derive(LogicBlock)]
    /// struct Foo {
    ///    pub clock: Signal<In, Clock>
    /// }
    ///
    /// impl Logic for Foo {
    ///   #[hdl_gen]
    ///   fn update(&mut self) {
    ///   }
    /// }
    ///
    /// let mut sim : Simulation<Foo> = Default::default();
    /// sim.add_clock(5, |x| x.clock.next = !x.clock.val()); // Toggles the clock every 5 picoseconds.
    /// ```
    ///
    pub fn add_clock<F>(&mut self, interval: u64, clock_fn: F)
    where
        F: Fn(&mut Box<T>) -> () + Send + 'static + std::panic::RefUnwindSafe,
    {
        self.add_testbench(move |mut ep: Sim<T>| {
            let mut x = ep.init()?;
            loop {
                x = ep.clock(interval, x)?;
                clock_fn(&mut x);
            }
        });
    }
    /// Add a phased clock to the simulation
    ///
    /// Sometimes you will need to control the phasing of a clock so that it starts at some
    /// non-zero time. This method allows you to add a clock to a simulation and control the
    /// initial delay.
    ///
    /// # Arguments
    ///
    /// * `interval` - the delay in picoseconds between the clock function being called
    /// * `phase_delay` - the number of picoseconds to wait before the clock starts being toggled
    /// * `clock_fn` - the function that toggles the actual clock.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rust_hdl_private_core::prelude::*;
    ///
    /// #[derive(LogicBlock)]
    /// struct Foo {
    ///    pub clock: Signal<In, Clock>
    /// }
    ///
    /// impl Logic for Foo {
    ///   #[hdl_gen]
    ///   fn update(&mut self) {
    ///   }
    /// }
    ///
    /// let mut sim : Simulation<Foo> = Default::default();
    /// // Toggles every 5 picoseconds, starting after 15 picoseconds
    /// sim.add_phased_clock(5, 15, |x| x.clock.next = !x.clock.val());
    /// ```
    ///
    pub fn add_phased_clock<F>(&mut self, interval: u64, phase_delay: u64, clock_fn: F)
    where
        F: Fn(&mut Box<T>) -> () + Send + 'static + std::panic::RefUnwindSafe,
    {
        self.add_testbench(move |mut ep: Sim<T>| {
            let mut x = ep.init()?;
            x = ep.wait(phase_delay, x)?;
            loop {
                x = ep.clock(interval, x)?;
                clock_fn(&mut x);
            }
        });
    }
    /// Add a testbench to the simulation
    ///
    /// # Arguments
    ///
    /// * `testbench` - a testbench function that will be executed through the
    /// simulation.  Needs to return a simulation [Result], be [Send] and
    /// [RefUnwindSafe] (no FFI).
    ///
    /// # Example
    ///
    pub fn add_testbench<F>(&mut self, testbench: F)
    where
        F: Fn(Sim<T>) -> Result<()> + Send + 'static + std::panic::RefUnwindSafe,
    {
        let ep = self.endpoint();
        self.testbenches.push(std::thread::spawn(move || {
            let ep_panic = ep.to_sim.clone();
            let result = std::panic::catch_unwind(|| testbench(ep));
            match result {
                Ok(x) => x,
                Err(_e) => {
                    ep_panic.send(MessageOrPanic::Panic).unwrap();
                    Err(SimError::SimPanic)
                }
            }
        }));
    }
    pub fn add_custom_logic<F>(&mut self, logic: F)
    where
        F: Fn(&mut T) -> () + 'static,
    {
        self.custom_logic.push(Box::new(logic));
    }
    pub fn endpoint(&mut self) -> Sim<T> {
        let (send_to_worker, recv_from_sim_to_worker) = bounded(0);
        let id = self.workers.len();
        let worker = Worker {
            id,
            channel_to_worker: send_to_worker,
            kind: TriggerType::Never,
        };
        self.workers.push(worker);
        Sim {
            to_sim: self.channel_to_sim.clone(),
            from_sim: recv_from_sim_to_worker,
            time: 0,
        }
    }
    fn dispatch(&mut self, idx: usize, x: Box<T>) -> Result<Box<T>> {
        let worker = &mut self.workers[idx];
        worker.channel_to_worker.send(Message {
            kind: TriggerType::Time(self.time),
            circuit: x,
        })?;
        let x = self.recv.recv()?;
        let mut x = match x {
            MessageOrPanic::Message(x) => x,
            MessageOrPanic::Panic => {
                return Err(SimError::SimPanic);
            }
        };
        worker.kind = x.kind;
        // Update the circuit
        let mut converged = false;
        for _ in 0..100 {
            for l in &self.custom_logic {
                l(&mut x.circuit);
            }
            x.circuit.update_all();
            if !x.circuit.has_changed() {
                converged = true;
                break;
            }
        }
        if !converged {
            Err(SimError::FailedToConverge)
        } else {
            Ok(x.circuit)
        }
    }
    fn scan_workers(&self, x: &T) -> NextTime {
        let mut min_time = !0_u64;
        let mut min_idx = 0;
        let mut only_clock_waiters = true;
        for worker in self.workers.iter() {
            match &worker.kind {
                TriggerType::Halt => {
                    return NextTime {
                        halted: true,
                        time: !0,
                        idx: !0,
                        clocks_only: false,
                    }
                }
                TriggerType::Never => {}
                TriggerType::Time(t) => {
                    only_clock_waiters = false;
                    if *t < min_time {
                        min_time = *t;
                        min_idx = worker.id;
                    }
                }
                TriggerType::Function(watch) => {
                    only_clock_waiters = false;
                    if watch(&x) {
                        min_idx = worker.id;
                        min_time = self.time;
                        break;
                    }
                }
                TriggerType::Clock(t) => {
                    if *t < min_time {
                        min_time = *t;
                        min_idx = worker.id;
                    }
                }
            }
        }
        NextTime {
            time: min_time,
            idx: min_idx,
            clocks_only: only_clock_waiters,
            halted: false,
        }
    }
    fn terminate(&mut self) {
        self.workers.clear();
        for handle in std::mem::take(&mut self.testbenches) {
            let _ = handle.join().unwrap();
        }
    }
    pub fn run(&mut self, mut x: Box<T>, max_time: u64) -> Result<()> {
        x.as_mut().connect_all();
        check_all(x.as_mut())?;
        // First initialize the workers.
        for id in 0..self.workers.len() {
            x = self.dispatch(id, x)?;
        }
        // Next run until we have no one else waiting
        let mut halted = false;
        while self.time < max_time {
            let next = self.scan_workers(&x);
            if next.time == !0 || next.clocks_only || next.halted {
                halted = next.halted;
                break;
            }
            self.time = next.time;
            x = self.dispatch(next.idx, x)?;
        }
        self.terminate();
        if self.time >= max_time {
            return Err(SimError::MaxTimeReached);
        }
        if halted {
            return Err(SimError::SimHalted);
        }
        Ok(())
    }
    pub fn run_to_file(&mut self, x: Box<T>, max_time: u64, name: &str) -> Result<()> {
        let mut vcd = vec![];
        let result = self.run_traced(x, max_time, &mut vcd);
        std::fs::write(name, vcd).unwrap();
        result
    }
    pub fn run_traced<W: Write>(&mut self, mut x: Box<T>, max_time: u64, trace: W) -> Result<()> {
        x.as_mut().connect_all();
        check_all(x.as_mut())?;
        let mut vcd = write_vcd_header(trace, x.as_ref());
        // First initialize the workers.
        for id in 0..self.workers.len() {
            x = self.dispatch(id, x)?;
        }
        vcd = write_vcd_dump(vcd, x.as_ref());
        let mut halted = false;
        // Next run until we have no one else waiting
        while self.time < max_time {
            let next = self.scan_workers(x.as_ref());
            if next.time == !0 || next.clocks_only || next.halted {
                halted = next.halted;
                break;
            }
            self.time = next.time;
            x = self.dispatch(next.idx, x)?;
            vcd.timestamp(next.time).unwrap();
            vcd = write_vcd_change(vcd, x.as_ref());
        }
        self.terminate();
        if self.time >= max_time {
            return Err(SimError::MaxTimeReached);
        }
        if halted {
            return Err(SimError::SimHalted);
        }
        Ok(())
    }
}

pub mod sim_time {
    pub const ONE_PICOSECOND: u64 = 1;
    pub const ONE_NANOSECOND: u64 = 1000 * ONE_PICOSECOND;
    pub const ONE_MICROSECOND: u64 = 1000 * ONE_NANOSECOND;
    pub const ONE_MILLISECOND: u64 = 1000 * ONE_MICROSECOND;
    pub const ONE_SEC: u64 = 1000 * ONE_MILLISECOND;
}

impl<T> Sim<T> {
    pub fn init(&self) -> Result<Box<T>> {
        Ok(self.from_sim.recv()?.circuit)
    }
    pub fn watch<S>(&mut self, check: S, x: Box<T>) -> Result<Box<T>>
    where
        S: Fn(&T) -> bool + Send + 'static,
    {
        self.to_sim.send(MessageOrPanic::Message(Message {
            kind: TriggerType::Function(Box::new(check)),
            circuit: x,
        }))?;
        let t = self.from_sim.recv()?;
        if let TriggerType::Time(t0) = t.kind {
            self.time = t0;
        }
        Ok(t.circuit)
    }
    pub fn clock(&mut self, delta: u64, x: Box<T>) -> Result<Box<T>> {
        self.to_sim.send(MessageOrPanic::Message(Message {
            kind: TriggerType::Clock(delta + self.time),
            circuit: x,
        }))?;
        let t = self.from_sim.recv()?;
        if let TriggerType::Time(t0) = t.kind {
            self.time = t0;
        }
        Ok(t.circuit)
    }
    pub fn wait(&mut self, delta: u64, x: Box<T>) -> Result<Box<T>> {
        self.to_sim.send(MessageOrPanic::Message(Message {
            kind: TriggerType::Time(delta + self.time),
            circuit: x,
        }))?;
        let t = self.from_sim.recv()?;
        if let TriggerType::Time(t0) = t.kind {
            self.time = t0;
        }
        Ok(t.circuit)
    }
    pub fn done(&self, x: Box<T>) -> Result<()> {
        self.to_sim.send(MessageOrPanic::Message(Message {
            kind: TriggerType::Never,
            circuit: x,
        }))?;
        Ok(())
    }
    pub fn halt(&self, x: Box<T>) -> Result<()> {
        self.to_sim.send(MessageOrPanic::Message(Message {
            kind: TriggerType::Halt,
            circuit: x,
        }))?;
        Err(SimError::SimHalted)
    }
    pub fn time(&self) -> u64 {
        self.time
    }
}

#[macro_export]
macro_rules! wait_clock_true {
    ($sim: ident, $($clock: ident).+, $me: expr) => {
        $me = $sim.watch(|x| x.$($clock).+.val().clk, $me)?
    };
}

#[macro_export]
macro_rules! wait_clock_false {
    ($sim: ident, $($clock: ident).+, $me: expr) => {
        $me = $sim.watch(|x| !x.$($clock).+.val().clk, $me)?
    };
}

#[macro_export]
macro_rules! wait_clock_cycle {
    ($sim: ident, $($clock: ident).+, $me: expr) => {
        if $me.$($clock).+.val().clk {
            wait_clock_false!($sim, $($clock).+, $me);
            wait_clock_true!($sim, $($clock).+, $me);
        } else {
            wait_clock_true!($sim, $($clock).+, $me);
            wait_clock_false!($sim, $($clock).+, $me);
        }
    };
    ($sim: ident, $clock: ident, $me: expr, $count: expr) => {
        for _i in 0..$count {
            wait_clock_cycle!($sim, $clock, $me);
        }
    };
}

#[macro_export]
macro_rules! wait_clock_cycles {
    ($sim: ident, $($clock: ident).+, $me: expr, $count: expr) => {
        for _i in 0..$count {
            wait_clock_cycle!($sim, $($clock).+, $me);
        }
    };
}

#[macro_export]
macro_rules! sim_assert {
    ($sim: ident, $test: expr, $circuit: ident) => {
        if !($test) {
            println!("HALT {}", stringify!($test));
            return $sim.halt($circuit);
        }
    };
}

#[macro_export]
macro_rules! sim_assert_eq {
    ($sim: ident, $lhs: expr, $rhs: expr, $circuit: ident) => {
        if !($lhs == $rhs) {
            println!(
                "HALT {} != {},  {:?} != {:?}",
                stringify!($lhs),
                stringify!($rhs),
                $lhs,
                $rhs
            );
            return $sim.halt($circuit);
        }
    };
}

#[macro_export]
macro_rules! simple_sim {
    ($kind: ty, $($clock: ident).+, $clock_speed_hz: expr, $fixture: ident, $testbench: expr) => {
        {
            let mut sim = Simulation::new();
            let half_period = 1_000_000_000_000 / (2 * $clock_speed_hz);
            sim.add_clock(half_period, |x: &mut Box<$kind>| x.$($clock).+.next = !x.$($clock).+.val());
            sim.add_testbench(move |mut $fixture: Sim<$kind>| {
                $testbench
            });
            sim
        }
    }
}

pub const SIMULATION_TIME_ONE_SECOND: u64 = 1_000_000_000_000;
