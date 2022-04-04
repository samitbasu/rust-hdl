use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicState, Debug, Copy, Clone, PartialEq)]
enum FIFOFeederState {
    Idle,
    Running,
    Sleeping,
    Done,
}

#[derive(LogicBlock)]
pub struct LazyFIFOFeeder<T: Synth, const N: usize> {
    pub clock: Signal<In, Clock>,
    pub bus: FIFOWriteController<T>,
    pub done: Signal<Out, Bit>,
    pub start: Signal<In, Bit>,
    state: DFF<FIFOFeederState>,
    sleep_counter: DFF<Bits<32>>,
    index: DFF<Bits<N>>,
    data_rom: ROM<T, N>,
    sleep_rom: ROM<Bits<32>, N>,
    data_len: Constant<Bits<N>>,
}

impl<T: Synth, const N: usize> LazyFIFOFeeder<T, N> {
    pub fn new(data: &[T], sleeps: &[Bits<32>]) -> LazyFIFOFeeder<T, N> {
        assert!(clog2(data.len()) <= N);
        assert_eq!(data.len(), sleeps.len());
        Self {
            clock: Default::default(),
            bus: Default::default(),
            done: Default::default(),
            start: Default::default(),
            state: Default::default(),
            sleep_counter: Default::default(),
            index: Default::default(),
            data_rom: data.to_vec().into_iter().into(),
            sleep_rom: sleeps.to_vec().into_iter().into(),
            data_len: Constant::new(data.len().into()),
        }
    }
}

impl<T: Synth, const N: usize> Logic for LazyFIFOFeeder<T, N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the flops
        self.state.clk.next = self.clock.val();
        self.sleep_counter.clk.next = self.clock.val();
        self.index.clk.next = self.clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.sleep_counter.d.next = self.sleep_counter.q.val();
        self.index.d.next = self.index.q.val();
        // Wire the FIFO bus to our data array
        self.bus.data.next = self.data_rom.data.val();
        self.bus.write.next = false;
        // Connect the ROMS
        self.sleep_rom.address.next = self.index.q.val();
        self.data_rom.address.next = self.index.q.val();
        self.done.next = false;
        match self.state.q.val() {
            FIFOFeederState::Idle => {
                if self.start.val() {
                    self.state.d.next = FIFOFeederState::Running;
                }
            }
            FIFOFeederState::Running => {
                if !self.bus.full.val() {
                    self.bus.write.next = true;
                    if self.index.q.val() == (self.data_len.val() - 1_usize) {
                        self.state.d.next = FIFOFeederState::Done;
                    } else if self.sleep_rom.data.val().any() {
                        self.state.d.next = FIFOFeederState::Sleeping;
                        self.sleep_counter.d.next = self.sleep_rom.data.val();
                    } else {
                        self.index.d.next = self.index.q.val() + 1_u32;
                    }
                }
            }
            FIFOFeederState::Sleeping => {
                if self.sleep_counter.q.val() == 0_u32 {
                    self.state.d.next = FIFOFeederState::Running;
                    self.index.d.next = self.index.q.val() + 1_u32;
                } else {
                    self.sleep_counter.d.next = self.sleep_counter.q.val() - 1_u32;
                }
            }
            FIFOFeederState::Done => {
                self.done.next = true;
            }
        }
    }
}

#[derive(LogicBlock)]
pub struct LazyFIFOReader<T: Synth, const N: usize> {
    pub clock: Signal<In, Clock>,
    pub bus: FIFOReadController<T>,
    pub done: Signal<Out, Bit>,
    pub start: Signal<In, Bit>,
    pub error: Signal<Out, Bit>,
    mismatch: DFF<Bit>,
    state: DFF<FIFOFeederState>,
    sleep_counter: DFF<Bits<32>>,
    index: DFF<Bits<N>>,
    data_rom: ROM<T, N>,
    sleep_rom: ROM<Bits<32>, N>,
    data_len: Constant<Bits<N>>,
}

impl<T: Synth, const N: usize> LazyFIFOReader<T, N> {
    pub fn new(data: &[T], sleeps: &[Bits<32>]) -> LazyFIFOReader<T, N> {
        assert!(clog2(data.len()) <= N);
        assert_eq!(data.len(), sleeps.len());
        Self {
            clock: Default::default(),
            bus: Default::default(),
            done: Default::default(),
            start: Default::default(),
            error: Default::default(),
            mismatch: Default::default(),
            state: Default::default(),
            sleep_counter: Default::default(),
            index: Default::default(),
            data_rom: data.to_vec().into_iter().into(),
            sleep_rom: sleeps.to_vec().into_iter().into(),
            data_len: Constant::new(data.len().into()),
        }
    }
}

impl<T: Synth, const N: usize> Logic for LazyFIFOReader<T, N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the logic
        self.state.clk.next = self.clock.val();
        self.mismatch.clk.next = self.clock.val();
        self.sleep_counter.clk.next = self.clock.val();
        self.index.clk.next = self.clock.val();
        // Latch prevention
        self.mismatch.d.next = self.mismatch.q.val();
        self.state.d.next = self.state.q.val();
        self.sleep_counter.d.next = self.sleep_counter.q.val();
        self.index.d.next = self.index.q.val();
        self.bus.read.next = false;
        self.done.next = false;
        // Connect the ROMS
        self.sleep_rom.address.next = self.index.q.val();
        self.data_rom.address.next = self.index.q.val();
        self.error.next = self.mismatch.q.val();
        match self.state.q.val() {
            FIFOFeederState::Idle => {
                if self.start.val() {
                    self.state.d.next = FIFOFeederState::Running;
                }
            }
            FIFOFeederState::Running => {
                if !self.bus.empty.val() {
                    if self.bus.data.val() != self.data_rom.data.val() {
                        self.mismatch.d.next = true;
                    }
                    self.bus.read.next = true;
                    if self.index.q.val() == (self.data_len.val() - 1_usize) {
                        self.state.d.next = FIFOFeederState::Done;
                    } else if self.sleep_rom.data.val().any() {
                        self.state.d.next = FIFOFeederState::Sleeping;
                        self.sleep_counter.d.next = self.sleep_rom.data.val();
                    } else {
                        self.index.d.next = self.index.q.val() + 1_u32;
                    }
                }
            }
            FIFOFeederState::Sleeping => {
                if self.sleep_counter.q.val() == 0_u32 {
                    self.state.d.next = FIFOFeederState::Running;
                    self.index.d.next = self.index.q.val() + 1_u32;
                } else {
                    self.sleep_counter.d.next = self.sleep_counter.q.val() - 1_u32;
                }
            }
            FIFOFeederState::Done => {
                self.done.next = true;
            }
        }
    }
}

pub fn bursty_rand() -> Bits<32> {
    if rand::thread_rng().gen::<f64>() < 0.9 {
        Bits::from(0_u32)
    } else {
        Bits::from((rand::thread_rng().gen::<f64>() * 40.0) as u32)
    }
}

pub fn bursty_vec(len: usize) -> Vec<Bits<32>> {
    (0..len).map(|_| bursty_rand()).collect()
}
