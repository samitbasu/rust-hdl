use crossbeam::channel::{bounded, Receiver, Sender};
use crossbeam::channel::{RecvError, SendError};

use crate::block::Block;

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

pub struct Simulation<T> {
    workers: Vec<Worker<T>>,
    recv: Receiver<Message<T>>,
    channel_to_sim: Sender<Message<T>>,
    time: u64,
}

pub struct Endpoint<T> {
    idx: usize,
    time: u64,
    to_sim: Sender<Message<T>>,
    from_sim: Receiver<Message<T>>,
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
        let (send_to_worker, recv_from_sim_to_worker) = bounded(0);
        let id = self.workers.len();
        let worker = Worker {
            id,
            channel_to_worker: send_to_worker,
            kind: TriggerType::Never,
        };
        self.workers.push(worker);
        Endpoint {
            idx: id,
            to_sim: self.channel_to_sim.clone(),
            from_sim: recv_from_sim_to_worker,
            time: 0,
        }
    }
    fn dispatch(&mut self, idx: usize, x: T) -> Result<T> {
        let worker = &mut self.workers[idx];
        println!("Sending circuit to worker {}", worker.id);
        worker.channel_to_worker.send(Message {
            id: worker.id,
            kind: TriggerType::Time(self.time),
            circuit: x,
        })?;
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
    where
        S: Fn(&T) -> bool + Send + 'static,
    {
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
