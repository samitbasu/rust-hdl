use std::collections::BTreeMap;
use std::f64::consts::PI;
use rand::Rng;
use crate::core::prelude::*;
use crate::hls::bridge::Bridge;
use crate::hls::bus::{FIFOReadController, FIFOReadResponder, FIFOWriteController, FIFOWriteResponder, SoCBusController, SoCPortController};
use crate::hls::controller::BaseController;
use crate::hls::fifo::AsyncFIFO;
use crate::hls::miso_port::MISOPort;
use crate::hls::mosi_port::MOSIPort;
use crate::widgets::prelude::*;

pub fn snore<const P: usize>(x: u32) -> Bits<P> {
    let amp = (f64::exp(f64::sin(((x as f64) - 128.0 / 2.) * PI / 128.0)) - 0.36787944) * 108.0;
    let amp = (amp.max(0.0).min(255.0).floor() / 255.0 * (1 << P) as f64) as u8;
    amp.to_bits()
}

#[derive(LogicBlock)]
pub struct FaderWithSyncROM {
    pub clock: Signal<In, Clock>,
    pub active: Signal<Out, Bit>,
    pub enable: Signal<In, Bit>,
    strobe: Strobe<32>,
    pwm: PulseWidthModulator<6>,
    rom: SyncROM<Bits<6>, 8>,
    counter: DFF<Bits<8>>,
}

impl FaderWithSyncROM {
    pub fn new(clock_frequency: u64, phase: u32) -> Self {
        let rom = (0..256)
            .map(|x| (x.to_bits(), snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            clock: Signal::default(),
            active: Signal::new_with_default(false),
            enable: Signal::default(),
            strobe: Strobe::new(clock_frequency, 120.0),
            pwm: PulseWidthModulator::default(),
            rom: SyncROM::new(rom),
            counter: Default::default(),
        }
    }
}

impl Logic for FaderWithSyncROM {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, strobe, pwm, counter);
        self.rom.clock.next = self.clock.val();
        self.rom.address.next = self.counter.q.val();
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
        self.strobe.enable.next = self.enable.val();
        self.pwm.enable.next = self.enable.val();
        self.active.next = self.pwm.active.val();
        self.pwm.threshold.next = self.rom.data.val();
    }
}

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
            data_len: Constant::new(data.len().to_bits()),
        }
    }
}

impl<T: Synth, const N: usize> Logic for LazyFIFOFeeder<T, N> {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, state, sleep_counter, index);
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
                    if self.index.q.val() == (self.data_len.val() - 1) {
                        self.state.d.next = FIFOFeederState::Done;
                    } else if self.sleep_rom.data.val().any() {
                        self.state.d.next = FIFOFeederState::Sleeping;
                        self.sleep_counter.d.next = self.sleep_rom.data.val();
                    } else {
                        self.index.d.next = self.index.q.val() + 1;
                    }
                }
            }
            FIFOFeederState::Sleeping => {
                if self.sleep_counter.q.val() == 0 {
                    self.state.d.next = FIFOFeederState::Running;
                    self.index.d.next = self.index.q.val() + 1;
                } else {
                    self.sleep_counter.d.next = self.sleep_counter.q.val() - 1;
                }
            }
            FIFOFeederState::Done => {
                self.done.next = true;
            }
            _ => {
                self.state.d.next = FIFOFeederState::Idle;
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
            data_len: Constant::new(data.len().to_bits()),
        }
    }
}

impl<T: Synth, const N: usize> Logic for LazyFIFOReader<T, N> {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, mismatch, state, sleep_counter, index);
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
                    if self.index.q.val() == (self.data_len.val() - 1) {
                        self.state.d.next = FIFOFeederState::Done;
                    } else if self.sleep_rom.data.val().any() {
                        self.state.d.next = FIFOFeederState::Sleeping;
                        self.sleep_counter.d.next = self.sleep_rom.data.val();
                    } else {
                        self.index.d.next = self.index.q.val() + 1;
                    }
                }
            }
            FIFOFeederState::Sleeping => {
                if self.sleep_counter.q.val() == 0 {
                    self.state.d.next = FIFOFeederState::Running;
                    self.index.d.next = self.index.q.val() + 1;
                } else {
                    self.sleep_counter.d.next = self.sleep_counter.q.val() - 1;
                }
            }
            FIFOFeederState::Done => {
                self.done.next = true;
            }
            _ => {
                self.state.d.next = FIFOFeederState::Idle;
            }
        }
    }
}

pub fn bursty_rand() -> Bits<32> {
    if rand::thread_rng().gen::<f64>() < 0.9 {
        Bits::from(0)
    } else {
        ((rand::thread_rng().gen::<f64>() * 40.0) as u32).to_bits()
    }
}

pub fn bursty_vec(len: usize) -> Vec<Bits<32>> {
    (0..len).map(|_| bursty_rand()).collect()
}

#[derive(LogicBlock)]
pub struct SoCTestChip {
    pub clock: Signal<In, Clock>,
    pub sys_clock: Signal<In, Clock>,
    pub from_cpu: FIFOWriteResponder<Bits<16>>,
    pub to_cpu: FIFOReadResponder<Bits<16>>,
    from_cpu_fifo: AsyncFIFO<Bits<16>, 8, 9, 1>,
    to_cpu_fifo: AsyncFIFO<Bits<16>, 8, 9, 1>,
    soc_host: BaseController<8>,
    bridge: Bridge<16, 8, 2>,
    mosi_port: MOSIPort<16>, // At address
    miso_port: MISOPort<16>,
    data_fifo: SynchronousFIFO<Bits<16>, 8, 9, 1>,
}

impl Default for SoCTestChip {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            sys_clock: Default::default(),
            from_cpu: Default::default(),
            to_cpu: Default::default(),
            from_cpu_fifo: Default::default(),
            to_cpu_fifo: Default::default(),
            soc_host: Default::default(),
            bridge: Bridge::new(["mosi", "miso"]),
            mosi_port: Default::default(),
            miso_port: Default::default(),
            data_fifo: Default::default(),
        }
    }
}

impl Logic for SoCTestChip {
    #[hdl_gen]
    fn update(&mut self) {
        self.from_cpu_fifo.write_clock.next = self.clock.val();
        self.to_cpu_fifo.read_clock.next = self.clock.val();
        self.from_cpu_fifo.read_clock.next = self.sys_clock.val();
        self.to_cpu_fifo.write_clock.next = self.sys_clock.val();
        self.soc_host.clock.next = self.sys_clock.val();
        // Connect the controller to the bridge
        SoCBusController::<16, 8>::join(&mut self.soc_host.bus, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.mosi_port.bus);
        SoCPortController::<16>::join(&mut self.bridge.nodes[1], &mut self.miso_port.bus);
        clock!(self, sys_clock, data_fifo);
        // Wire the MOSI port to the input of the data_fifo
        self.data_fifo.data_in.next = self.mosi_port.port_out.val() << 1;
        self.data_fifo.write.next = self.mosi_port.strobe_out.val();
        self.mosi_port.ready.next = !self.data_fifo.full.val();
        // Wire the MISO port to the output of the data fifo
        self.miso_port.port_in.next = self.data_fifo.data_out.val();
        self.data_fifo.read.next = self.miso_port.strobe_out.val();
        self.miso_port.ready_in.next = !self.data_fifo.empty.val();
        // Wire the cpu fifos to the host
        FIFOWriteResponder::<Bits<16>>::link(&mut self.from_cpu, &mut self.from_cpu_fifo.bus_write);
        FIFOReadResponder::<Bits<16>>::link(&mut self.to_cpu, &mut self.to_cpu_fifo.bus_read);
        FIFOReadResponder::<Bits<16>>::join(
            &mut self.from_cpu_fifo.bus_read,
            &mut self.soc_host.from_cpu,
        );
        FIFOWriteResponder::<Bits<16>>::join(
            &mut self.to_cpu_fifo.bus_write,
            &mut self.soc_host.to_cpu,
        );
    }
}
