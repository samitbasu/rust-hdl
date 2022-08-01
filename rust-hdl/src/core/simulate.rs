use crossbeam::channel::{bounded, Receiver, Sender};
use crossbeam::channel::{RecvError, SendError};

use crate::core::block::Block;
use crate::core::check_connected::check_connected;
use crate::core::check_error::CheckError;
use crate::core::check_logic_loops::check_logic_loops;
use crate::core::vcd_probe::{write_vcd_change, write_vcd_dump, write_vcd_header};
use std::io::Write;
use std::thread::JoinHandle;

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
pub enum SimError {
    SimTerminated,
    MaxTimeReached,
    SimHalted,
    FailedToConverge,
    Check(CheckError),
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

pub type CustomLogicFn<T> = Box<dyn Fn(&mut T) -> ()>;

pub struct Simulation<T> {
    workers: Vec<Worker<T>>,
    recv: Receiver<MessageOrPanic<T>>,
    channel_to_sim: Sender<MessageOrPanic<T>>,
    time: u64,
    testbenches: Vec<JoinHandle<Result<()>>>,
    custom_logic: Vec<CustomLogicFn<T>>,
}

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

impl<T: Send + 'static + Block> Simulation<T> {
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
                Err(e) => {
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
        check_connected(x.as_mut())?;
        check_logic_loops(x.as_mut())?;
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
        check_connected(x.as_mut())?;
        check_logic_loops(x.as_mut())?;
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
