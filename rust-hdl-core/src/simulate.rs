use crossbeam::channel::{bounded, Receiver, Sender};
use crossbeam::channel::{RecvError, SendError};

use crate::block::Block;
use crate::check_connected::check_connected;
use crate::vcd_probe::{write_vcd_change, write_vcd_dump, write_vcd_header};
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SimError {
    SimTerminated,
    MaxTimeReached,
    SimHalted,
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

struct Worker<T> {
    id: usize,
    channel_to_worker: Sender<Message<T>>,
    kind: TriggerType<T>,
}

pub struct Simulation<T> {
    workers: Vec<Worker<T>>,
    recv: Receiver<Message<T>>,
    channel_to_sim: Sender<Message<T>>,
    time: u64,
    testbenches: Vec<JoinHandle<Result<()>>>,
}

pub struct Sim<T> {
    time: u64,
    to_sim: Sender<Message<T>>,
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
        }
    }
    pub fn add_clock<F>(&mut self, interval: u64, clock_fn: F)
    where
        F: Fn(&mut Box<T>) -> () + Send + 'static,
    {
        self.add_testbench(move |mut ep: Sim<T>| {
            let mut x = ep.init()?;
            loop {
                x = ep.clock(interval, x)?;
                clock_fn(&mut x);
            }
        });
    }
    pub fn add_testbench<F>(&mut self, testbench: F)
    where
        F: Fn(Sim<T>) -> Result<()> + Send + 'static,
    {
        let ep = self.endpoint();
        self.testbenches
            .push(std::thread::spawn(move || testbench(ep)));
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
        let mut x = self.recv.recv()?;
        worker.kind = x.kind;
        // Update the circuit
        for _ in 0..10 {
            x.circuit.update_all();
            if !x.circuit.has_changed() {
                break;
            }
        }
        Ok(x.circuit)
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
        check_connected(x.as_mut());
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
    pub fn run_traced<W: Write>(&mut self, mut x: Box<T>, max_time: u64, trace: W) -> Result<()> {
        check_connected(x.as_mut());
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
        self.to_sim.send(Message {
            kind: TriggerType::Function(Box::new(check)),
            circuit: x,
        })?;
        let t = self.from_sim.recv()?;
        if let TriggerType::Time(t0) = t.kind {
            self.time = t0;
        }
        Ok(t.circuit)
    }
    pub fn clock(&mut self, delta: u64, x: Box<T>) -> Result<Box<T>> {
        self.to_sim.send(Message {
            kind: TriggerType::Clock(delta + self.time),
            circuit: x,
        })?;
        let t = self.from_sim.recv()?;
        if let TriggerType::Time(t0) = t.kind {
            self.time = t0;
        }
        Ok(t.circuit)
    }
    pub fn wait(&mut self, delta: u64, x: Box<T>) -> Result<Box<T>> {
        self.to_sim.send(Message {
            kind: TriggerType::Time(delta + self.time),
            circuit: x,
        })?;
        let t = self.from_sim.recv()?;
        if let TriggerType::Time(t0) = t.kind {
            self.time = t0;
        }
        Ok(t.circuit)
    }
    pub fn done(&self, x: Box<T>) -> Result<()> {
        self.to_sim.send(Message {
            kind: TriggerType::Never,
            circuit: x,
        })?;
        Ok(())
    }
    pub fn halt(&self, x: Box<T>) -> Result<()> {
        self.to_sim.send(Message {
            kind: TriggerType::Halt,
            circuit: x,
        })?;
        Err(SimError::SimHalted)
    }
    pub fn time(&self) -> u64 {
        self.time
    }
}

#[macro_export]
macro_rules! wait_clock_true {
    ($sim: ident, $clock: ident, $me: expr) => {
        $me = $sim.watch(|x| x.$clock.val().0, $me)?
    };
}

#[macro_export]
macro_rules! wait_clock_false {
    ($sim: ident, $clock: ident, $me: expr) => {
        $me = $sim.watch(|x| !x.$clock.val().0, $me)?
    };
}

#[macro_export]
macro_rules! wait_clock_cycle {
    ($sim: ident, $clock: ident, $me: expr) => {
        if $me.$clock.val().0 {
            wait_clock_false!($sim, $clock, $me);
            wait_clock_true!($sim, $clock, $me);
        } else {
            wait_clock_true!($sim, $clock, $me);
            wait_clock_false!($sim, $clock, $me);
        }
    };
    ($sim: ident, $clock: ident, $me: expr, $count: expr) => {
        for _i in 0..$count {
            wait_clock_cycle!($sim, $clock, $me);
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
