use crate::core::prelude::*;
use crate::widgets::async_fifo::AsynchronousFIFO;
use crate::widgets::dff::DFF;
use crate::widgets::prelude::{TristateBuffer, SynchronousFIFO};

#[derive(Clone, Debug, Default, LogicInterface)]
pub struct FifoBus<T: Synth> {
    pub to_bus: Signal<In, T>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub from_bus: Signal<Out, T>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
pub struct BidiBusM<T: Synth> {
    pub sig_inout: Signal<InOut, T>,
    pub sig_empty: Signal<In, Bit>,
    pub sig_full: Signal<In, Bit>,
    pub sig_not_read: Signal<Out, Bit>,
    pub sig_not_write: Signal<Out, Bit>,
    pub sig_master: Signal<Out, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
pub struct BidiBusD<T: Synth> {
    pub sig_inout: Signal<InOut, T>,
    pub sig_empty: Signal<Out, Bit>,
    pub sig_full: Signal<Out, Bit>,
    pub sig_not_read: Signal<In, Bit>,
    pub sig_not_write: Signal<In, Bit>,
    pub sig_master: Signal<In, Bit>,
}


#[derive(LogicBlock, Default)]
pub struct BidiMaster<T: Synth, const N: usize, const NP1: usize> {
    pub bus: BidiBusM<T>,
    pub bus_clock: Signal<In, Clock>,
    bus_buffer: TristateBuffer<T>,
    fifo_to_bus: AsynchronousFIFO<T, N, NP1, 1>,
    fifo_from_bus: AsynchronousFIFO<T, N, NP1, 1>,
    pub data: FifoBus<T>,
    pub data_clock: Signal<In, Clock>,
    state: DFF<BidiState>,
    can_send_to_bus: Signal<Local, Bit>,
    can_read_from_bus: Signal<Local, Bit>,
}

impl<T: Synth, const N: usize, const NP1: usize> Logic for BidiMaster<T, N, NP1> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the logic
        self.fifo_to_bus.read_clock.next = self.bus_clock.val();
        self.fifo_from_bus.write_clock.next = self.bus_clock.val();
        self.fifo_to_bus.write_clock.next = self.data_clock.val();
        self.fifo_from_bus.read_clock.next = self.data_clock.val();
        self.state.clk.next = self.bus_clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.bus_buffer.write_enable.next = false;
        self.bus.sig_not_read.next = true;
        self.bus.sig_not_write.next = true;
        self.fifo_to_bus.read.next = false;
        self.fifo_from_bus.write.next = false;
        // Wire up the tristate buffer
        self.bus.sig_inout.link(&mut self.bus_buffer.bus);
        self.bus_buffer.write_data.next = self.fifo_to_bus.data_out.val();
        self.fifo_from_bus.data_in.next = self.bus_buffer.read_data.val();
        // Wire up the fabric side signals for the from-bus interface
        self.data.from_bus.next = self.fifo_from_bus.data_out.val();
        self.fifo_from_bus.read.next = self.data.read.val();
        self.data.empty.next = self.fifo_from_bus.empty.val();
        // Wire up the fabric side signals for the to-bus interface
        self.fifo_to_bus.data_in.next = self.data.to_bus.val();
        self.fifo_to_bus.write.next = self.data.write.val();
        self.data.full.next = self.fifo_to_bus.full.val();
        // Now we do the FSM logic
        self.bus.sig_master.next = true;
        self.can_send_to_bus.next = !self.fifo_to_bus.empty.val() & !self.bus.sig_full.val();
        self.can_read_from_bus.next = !self.fifo_from_bus.full.val() & !self.bus.sig_empty.val();
        match self.state.q.val() {
            BidiState::Idle => {
                if self.can_send_to_bus.val() {
                    self.state.d.next = BidiState::Claiming;
                }
                if self.can_read_from_bus.val() {
                    self.state.d.next = BidiState::Turnaround;
                    self.bus.sig_master.next = false;
                }
            }
            BidiState::Claiming => {
                self.bus_buffer.write_enable.next = true;
                self.state.d.next = BidiState::Sending;
            }
            BidiState::Sending => {
                self.bus_buffer.write_enable.next = true;
                self.fifo_to_bus.read.next = self.can_send_to_bus.val();
                self.bus.sig_not_write.next = !self.can_send_to_bus.val();
                if !self.can_send_to_bus.val() {
                    self.state.d.next = BidiState::Idle;
                }
            }
            BidiState::Receiving => {
                self.bus.sig_master.next = false;
                self.fifo_from_bus.write.next = self.can_read_from_bus.val();
                self.bus.sig_not_read.next = !self.can_read_from_bus.val();
                if !self.can_read_from_bus.val() {
                    self.state.d.next = BidiState::Release;
                }
            }
            BidiState::Turnaround => {
                self.bus.sig_master.next = false;
                self.state.d.next = BidiState::Receiving;
            }
            BidiState::Release => {
                self.state.d.next = BidiState::Idle;
            }
        }
    }
}


#[derive(LogicState, Debug, Copy, Clone, PartialEq)]
enum BidiState {
    Idle,
    Claiming,
    Sending,
    Receiving,
    Turnaround,
    Release,
}

#[test]
fn test_bidi_master2_synthesizable() {
    let mut uut = BidiMaster::<Bits<8>, 8, 9>::default();
    uut.bus.link_connect_dest();
    uut.bus_clock.connect();
    uut.data.write.connect();
    uut.data.read.connect();
    uut.data.to_bus.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("bidi_master2", &vlog).unwrap();
}

#[derive(LogicBlock, Default)]
pub struct BidiDevice<T: Synth, const N: usize, const NP1: usize> {
    pub bus: BidiBusD<T>,
    pub clock: Signal<In, Clock>,
    bus_buffer: TristateBuffer<T>,
    fifo_to_bus: SynchronousFIFO<T, N, NP1, 1>,
    fifo_from_bus: SynchronousFIFO<T, N, NP1, 1>,
    pub data: FifoBus<T>,
}

impl<T: Synth, const N: usize, const NP1: usize> Logic for BidiDevice<T, N, NP1> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the logic
        self.fifo_to_bus.clock.next = self.clock.val();
        self.fifo_from_bus.clock.next = self.clock.val();
        // Wire up the tristate buffer
        self.bus.sig_inout.link(&mut self.bus_buffer.bus);
        // Connect the FIFO that leads to the bus
        // The data interfaces
        self.fifo_to_bus.data_in.next = self.data.to_bus.val();
        self.fifo_to_bus.write.next = self.data.write.val();
        self.data.full.next = self.fifo_to_bus.full.val();
        // The bus interfaces
        self.fifo_to_bus.read.next = !self.bus.sig_not_read.val();
        self.bus.sig_empty.next = self.fifo_to_bus.empty.val();
        self.bus_buffer.write_data.next = self.fifo_to_bus.data_out.val();
        // Connect the FIFO that leads from the bus
        // The data interfaces
        self.data.from_bus.next = self.fifo_from_bus.data_out.val();
        self.data.empty.next = self.fifo_from_bus.empty.val();
        self.fifo_from_bus.read.next = self.data.read.val();
        // The bus interfaces
        self.fifo_from_bus.data_in.next = self.bus_buffer.read_data.val();
        self.fifo_from_bus.write.next = !self.bus.sig_not_write.val();
        self.bus.sig_full.next = self.fifo_from_bus.full.val();
        // Connect the write enable line
        self.bus_buffer.write_enable.next = !self.bus.sig_master.val();
    }
}

#[test]
fn test_bidi_device_synthesizable() {
    let mut uut = BidiDevice::<Bits<8>, 8, 9>::default();
    uut.bus.connect();
    uut.data.read.connect();
    uut.data.write.connect();
    uut.clock.connect();
    uut.data.to_bus.connect();
    uut.bus.sig_not_read.connect();
    uut.bus.sig_not_write.connect();
    uut.bus.sig_master.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("bidi_device", &vlog).unwrap();
}
