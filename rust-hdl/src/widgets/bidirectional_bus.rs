use crate::core::prelude::*;
use crate::widgets::sync_fifo::SynchronousFIFO;
use crate::widgets::dff::DFF;
use crate::widgets::prelude::TristateBuffer;

#[derive(LogicState, Debug, Copy, Clone, PartialEq)]
enum BidiState {
    Idle,
    Claiming,
    Sending,
    Receiving,
    Turnaround,
    Release,
}

#[derive(LogicBlock, Default)]
pub struct BidirectionalBusMaster<T: Synth, const N: usize, const NP1: usize> {
    pub bus: Signal<InOut, T>,
    pub bus_empty: Signal<In, Bit>,
    pub bus_full: Signal<In, Bit>,
    pub bus_read: Signal<Out, Bit>,
    pub bus_write: Signal<Out, Bit>,
    pub slave_is_reading: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    bus_buffer: TristateBuffer<T>,
    fifo_to_bus: SynchronousFIFO<T, N, NP1, 1>,
    fifo_from_bus: SynchronousFIFO<T, N, NP1, 1>,
    pub data_out: Signal<Out, T>,
    pub data_read: Signal<In, Bit>,
    pub data_empty: Signal<Out, Bit>,
    pub data_in: Signal<In, T>,
    pub data_write: Signal<In, Bit>,
    pub data_full: Signal<Out, Bit>,
    state: DFF<BidiState>,
    can_send_to_bus: Signal<Local, Bit>,
    can_read_from_bus: Signal<Local, Bit>,
}

impl<T: Synth, const N: usize, const NP1: usize> Logic for BidirectionalBusMaster<T, N, NP1> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the logic
        self.fifo_to_bus.clock.next = self.clock.val();
        self.fifo_from_bus.clock.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.bus_buffer.write_enable.next = false;
        self.bus_read.next = false;
        self.bus_write.next = false;
        self.fifo_to_bus.read.next = false;
        self.fifo_from_bus.write.next = false;
        // Wire up the tristate buffer
        self.bus.link(&mut self.bus_buffer.bus);
        self.bus_buffer.write_data.next = self.fifo_to_bus.data_out.val();
        self.fifo_from_bus.data_in.next = self.bus_buffer.read_data.val();
        // Wire up the fabric side signals for the from-bus interface
        self.data_out.next = self.fifo_from_bus.data_out.val();
        self.fifo_from_bus.read.next = self.data_read.val();
        self.data_empty.next = self.fifo_from_bus.empty.val();
        // Wire up the fabric side signals for the to-bus interface
        self.fifo_to_bus.data_in.next = self.data_in.val();
        self.fifo_to_bus.write.next = self.data_write.val();
        self.data_full.next = self.fifo_to_bus.full.val();
        // Now we do the FSM logic
        self.slave_is_reading.next = true;
        self.can_send_to_bus.next = !self.fifo_to_bus.empty.val() & !self.bus_full.val();
        self.can_read_from_bus.next = !self.fifo_from_bus.full.val() & !self.bus_empty.val();
        match self.state.q.val() {
            BidiState::Idle => {
                if self.can_send_to_bus.val() {
                    self.state.d.next = BidiState::Claiming;
                }
                if self.can_read_from_bus.val() {
                    self.state.d.next = BidiState::Turnaround;
                    self.slave_is_reading.next = false;
                }
            }
            BidiState::Claiming => {
                self.bus_buffer.write_enable.next = true;
                self.state.d.next = BidiState::Sending;
            }
            BidiState::Sending => {
                self.bus_buffer.write_enable.next = true;
                self.fifo_to_bus.read.next = self.can_send_to_bus.val();
                self.bus_write.next = self.can_send_to_bus.val();
                if !self.can_send_to_bus.val() {
                    self.state.d.next = BidiState::Idle;
                }
            }
            BidiState::Receiving => {
                self.slave_is_reading.next = false;
                self.fifo_from_bus.write.next = self.can_read_from_bus.val();
                self.bus_read.next = self.can_read_from_bus.val();
                if !self.can_read_from_bus.val() {
                    self.state.d.next = BidiState::Release;
                }
            }
            BidiState::Turnaround => {
                self.slave_is_reading.next = false;
                self.state.d.next = BidiState::Receiving;
            }
            BidiState::Release => {
                self.state.d.next = BidiState::Idle;
            }
        }
    }
}

#[test]
fn test_bidi_master_synthesizable() {
    let mut uut = BidirectionalBusMaster::<Bits<8>, 8, 9>::default();
    uut.bus.connect();
    uut.bus_empty.connect();
    uut.bus_full.connect();
    uut.clock.connect();
    uut.data_write.connect();
    uut.data_read.connect();
    uut.data_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("bidi_master", &vlog).unwrap();
}


#[derive(LogicBlock, Default)]
pub struct BidirectionalBusSlave<T: Synth, const N: usize, const NP1: usize> {
    pub bus: Signal<InOut, T>,
    pub bus_empty: Signal<Out, Bit>,
    pub bus_full: Signal<Out, Bit>,
    pub bus_read: Signal<In, Bit>,
    pub bus_write: Signal<In, Bit>,
    pub slave_is_reading: Signal<In, Bit>,
    pub clock: Signal<In, Clock>,
    bus_buffer: TristateBuffer<T>,
    fifo_to_bus: SynchronousFIFO<T, N, NP1, 1>,
    fifo_from_bus: SynchronousFIFO<T, N, NP1, 1>,
    pub data_out: Signal<Out, T>,
    pub data_read: Signal<In, Bit>,
    pub data_empty: Signal<Out, Bit>,
    pub data_in: Signal<In, T>,
    pub data_write: Signal<In, Bit>,
    pub data_full: Signal<Out, Bit>,
}

impl<T: Synth, const N: usize, const NP1: usize> Logic for BidirectionalBusSlave<T, N, NP1> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the logic
        self.fifo_to_bus.clock.next = self.clock.val();
        self.fifo_from_bus.clock.next = self.clock.val();
        // Wire up the tristate buffer
        self.bus.link(&mut self.bus_buffer.bus);
        // Connect the FIFO that leads to the bus
        // The data interfaces
        self.fifo_to_bus.data_in.next = self.data_in.val();
        self.fifo_to_bus.write.next = self.data_write.val();
        self.data_full.next = self.fifo_to_bus.full.val();
        // The bus interfaces
        self.fifo_to_bus.read.next = self.bus_read.val();
        self.bus_empty.next = self.fifo_to_bus.empty.val();
        self.bus_buffer.write_data.next = self.fifo_to_bus.data_out.val();
        // Connect the FIFO that leads from the bus
        // The data interfaces
        self.data_out.next = self.fifo_from_bus.data_out.val();
        self.data_empty.next = self.fifo_from_bus.empty.val();
        self.fifo_from_bus.read.next = self.data_read.val();
        // The bus interfaces
        self.fifo_from_bus.data_in.next = self.bus_buffer.read_data.val();
        self.fifo_from_bus.write.next = self.bus_write.val();
        self.bus_full.next = self.fifo_from_bus.full.val();
        // Connect the write enable line
        self.bus_buffer.write_enable.next = !self.slave_is_reading.val();
    }
}

#[test]
fn test_bidi_slave_synthesizable() {
    let mut uut = BidirectionalBusSlave::<Bits<8>, 8, 9>::default();
    uut.bus.connect();
    uut.data_read.connect();
    uut.data_write.connect();
    uut.clock.connect();
    uut.data_in.connect();
    uut.bus_read.connect();
    uut.bus_write.connect();
    uut.slave_is_reading.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("bidi_slave", &vlog).unwrap();
}
