use crate::bus::{FIFOReadController, FIFOWriteController};
use rust_hdl_private_core::prelude::*;
use rust_hdl_private_widgets::prelude::*;

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "BidiBusD"]
pub struct BidiBusM<T: Synth> {
    pub sig_inout: Signal<InOut, T>,
    pub sig_empty: Signal<In, Bit>,
    pub sig_full: Signal<In, Bit>,
    pub sig_not_read: Signal<Out, Bit>,
    pub sig_not_write: Signal<Out, Bit>,
    pub sig_master: Signal<Out, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "BidiBusM"]
pub struct BidiBusD<T: Synth> {
    pub sig_inout: Signal<InOut, T>,
    pub sig_empty: Signal<Out, Bit>,
    pub sig_full: Signal<Out, Bit>,
    pub sig_not_read: Signal<In, Bit>,
    pub sig_not_write: Signal<In, Bit>,
    pub sig_master: Signal<In, Bit>,
}

#[derive(LogicBlock, Default)]
pub struct BidiMaster<T: Synth> {
    pub bus: BidiBusM<T>,
    pub clock: Signal<In, Clock>,
    bus_buffer: TristateBuffer<T>,
    pub data_to_bus: FIFOReadController<T>,
    pub data_from_bus: FIFOWriteController<T>,
    state: DFF<BidiState>,
    can_send_to_bus: Signal<Local, Bit>,
    can_read_from_bus: Signal<Local, Bit>,
}

impl<T: Synth> Logic for BidiMaster<T> {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, state);
        self.bus_buffer.write_enable.next = false;
        self.bus.sig_not_read.next = true;
        self.bus.sig_not_write.next = true;
        // Wire up the tristate buffer
        Signal::<InOut, T>::link(&mut self.bus.sig_inout, &mut self.bus_buffer.bus);
        self.bus_buffer.write_data.next = self.data_to_bus.data.val();
        self.data_from_bus.data.next = self.bus_buffer.read_data.val();
        // Now we do the FSM logic
        self.bus.sig_master.next = true;
        self.can_send_to_bus.next = !self.data_to_bus.empty.val() & !self.bus.sig_full.val();
        self.can_read_from_bus.next = !self.data_from_bus.full.val() & !self.bus.sig_empty.val();
        // Default values for the FIFO controllers
        self.data_to_bus.read.next = false;
        self.data_from_bus.write.next = false;
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
                self.data_to_bus.read.next = self.can_send_to_bus.val();
                self.bus.sig_not_write.next = !self.can_send_to_bus.val();
                if !self.can_send_to_bus.val() {
                    self.state.d.next = BidiState::Idle;
                }
            }
            BidiState::Receiving => {
                self.bus.sig_master.next = false;
                self.data_from_bus.write.next = self.can_read_from_bus.val();
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
            _ => {
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
    let mut uut = BidiMaster::<Bits<8>>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("bidi_master2", &vlog).unwrap();
}

#[derive(LogicBlock, Default)]
pub struct BidiSimulatedDevice<T: Synth> {
    pub bus: BidiBusD<T>,
    pub clock: Signal<In, Clock>,
    bus_buffer: TristateBuffer<T>,
    pub data_to_bus: FIFOReadController<T>,
    pub data_from_bus: FIFOWriteController<T>,
}

impl<T: Synth> Logic for BidiSimulatedDevice<T> {
    #[hdl_gen]
    fn update(&mut self) {
        // Wire up the tristate buffer
        Signal::<InOut, T>::link(&mut self.bus.sig_inout, &mut self.bus_buffer.bus);
        // The bus interfaces
        self.data_to_bus.read.next = !self.bus.sig_not_read.val();
        self.bus.sig_empty.next = self.data_to_bus.empty.val();
        self.bus_buffer.write_data.next = self.data_to_bus.data.val();
        // The bus interfaces
        self.data_from_bus.data.next = self.bus_buffer.read_data.val();
        self.data_from_bus.write.next = !self.bus.sig_not_write.val();
        self.bus.sig_full.next = self.data_from_bus.full.val();
        // Connect the write enable line
        self.bus_buffer.write_enable.next = !self.bus.sig_master.val();
    }
}

#[test]
fn test_bidi_device_synthesizable() {
    let mut uut = BidiSimulatedDevice::<Bits<8>>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("bidi_device", &vlog).unwrap();
}
