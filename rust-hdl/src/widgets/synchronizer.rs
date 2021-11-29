use crate::widgets::dff::DFF;
use crate::core::prelude::*;

#[derive(LogicBlock, Default)]
pub struct BitSynchronizer {
    pub sig_in: Signal<In, Bit>,
    pub sig_out: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    dff0: DFF<Bit>,
    dff1: DFF<Bit>,
}

impl Logic for BitSynchronizer {
    #[hdl_gen]
    fn update(&mut self) {
        self.dff0.clk.next = self.clock.val();
        self.dff1.clk.next = self.clock.val();

        self.dff0.d.next = self.sig_in.val();
        self.dff1.d.next = self.dff0.q.val();
        self.sig_out.next = self.dff1.q.val();
    }
}

#[test]
fn sync_is_synthesizable() {
    rust_hdl_yosys_synth::top_wrap!(BitSynchronizer, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.sig_in.connect();
    dev.connect_all();
    rust_hdl_yosys_synth::yosys_validate("sync", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
pub enum SyncSenderState {
    Idle,
    WaitAck,
    WaitDone,
}

#[derive(LogicBlock, Default)]
pub struct SyncSender<T: Synth> {
    pub sig_in: Signal<In, T>,
    pub clock: Signal<In, Clock>,
    pub sig_cross: Signal<Out, T>,
    pub flag_out: Signal<Out, Bit>,
    pub ack_in: Signal<In, Bit>,
    pub busy: Signal<Out, Bit>,
    pub send: Signal<In, Bit>,
    hold: DFF<T>,
    state: DFF<SyncSenderState>,
    sync: BitSynchronizer,
}

impl<T: Synth> Logic for SyncSender<T> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clocks
        self.state.clk.next = self.clock.val();
        self.sync.clock.next = self.clock.val();
        self.hold.clk.next = self.clock.val();

        // By default, the hold DFF does not change
        self.hold.d.next = self.hold.q.val();
        self.sig_cross.next = self.hold.q.val();
        self.flag_out.next = false.into();
        self.sync.sig_in.next = self.ack_in.val();

        // State machine
        self.state.d.next = self.state.q.val();
        self.busy.next = true;
        match self.state.q.val() {
            SyncSenderState::Idle => {
                self.busy.next = false.into();
                if self.send.val() {
                    // Sample the input signal
                    self.hold.d.next = self.sig_in.val();
                    self.state.d.next = SyncSenderState::WaitAck.into();
                    // Indicate that the output is valid
                    self.flag_out.next = true;
                }
            }
            SyncSenderState::WaitAck => {
                self.flag_out.next = true;
                if self.sync.sig_out.val() {
                    self.state.d.next = SyncSenderState::WaitDone.into();
                    self.flag_out.next = false.into();
                }
            }
            SyncSenderState::WaitDone => {
                if !self.sync.sig_out.val() {
                    self.hold.d.next = self.sig_in.val();
                    // Indicate that the output is valid
                    self.flag_out.next = true;
                    self.state.d.next = SyncSenderState::Idle.into();
                }
            }
        }
    }
}

#[test]
fn sync_sender_is_synthesizable() {
    rust_hdl_yosys_synth::top_wrap!(SyncSender<Bits<8>>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.sig_in.connect();
    dev.uut.ack_in.connect();
    dev.uut.send.connect();
    dev.connect_all();
    println!("{}", generate_verilog(&dev));
    rust_hdl_yosys_synth::yosys_validate("sync_send", &generate_verilog(&dev)).unwrap();
}

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
pub enum SyncReceiverState {
    WaitSteady,
    WaitDone,
}

#[derive(LogicBlock, Default)]
pub struct SyncReceiver<T: Synth> {
    pub sig_out: Signal<Out, T>,
    pub clock: Signal<In, Clock>,
    pub sig_cross: Signal<In, T>,
    pub flag_in: Signal<In, Bit>,
    pub ack_out: Signal<Out, Bit>,
    pub update: Signal<Out, Bit>,
    hold: DFF<T>,
    update_delay: DFF<Bit>,
    state: DFF<SyncReceiverState>,
    sync: BitSynchronizer,
}

impl<T: Synth> Logic for SyncReceiver<T> {
    #[hdl_gen]
    fn update(&mut self) {
        self.state.clk.next = self.clock.val();
        self.sync.clock.next = self.clock.val();
        self.hold.clk.next = self.clock.val();
        self.update_delay.clk.next = self.clock.val();

        self.hold.d.next = self.hold.q.val();
        self.sig_out.next = self.hold.q.val();
        self.ack_out.next = false.into();
        self.sync.sig_in.next = self.flag_in.val();

        self.state.d.next = self.state.q.val();
        self.update.next = self.update_delay.q.val();
        self.update_delay.d.next = false.into();
        match self.state.q.val() {
            SyncReceiverState::WaitSteady => {
                if self.sync.sig_out.val() {
                    self.ack_out.next = true;
                    self.state.d.next = SyncReceiverState::WaitDone.into();
                }
            }
            SyncReceiverState::WaitDone => {
                if !self.sync.sig_out.val() {
                    self.ack_out.next = false.into();
                    self.update_delay.d.next = true;
                    self.hold.d.next = self.sig_cross.val().into();
                    self.state.d.next = SyncReceiverState::WaitSteady.into();
                } else {
                    self.ack_out.next = true;
                }
            }
        }
    }
}

#[test]
fn sync_receiver_is_synthesizable() {
    rust_hdl_yosys_synth::top_wrap!(SyncReceiver<Bits<8>>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.sig_cross.connect();
    dev.uut.flag_in.connect();
    dev.connect_all();
    println!("{}", generate_verilog(&dev));
    rust_hdl_yosys_synth::yosys_validate("sync_recv", &generate_verilog(&dev)).unwrap();
}

#[derive(LogicBlock, Default)]
pub struct VectorSynchronizer<T: Synth> {
    // The input interface...
    pub clock_in: Signal<In, Clock>,
    pub sig_in: Signal<In, T>,
    pub busy: Signal<Out, Bit>,
    pub send: Signal<In, Bit>,
    // The output interface...
    pub clock_out: Signal<In, Clock>,
    pub sig_out: Signal<Out, T>,
    pub update: Signal<Out, Bit>,
    // The two pieces of the synchronizer
    sender: SyncSender<T>,
    recv: SyncReceiver<T>,
}

impl<T: Synth> Logic for VectorSynchronizer<T> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clocks...
        self.sender.clock.next = self.clock_in.val();
        self.recv.clock.next = self.clock_out.val();
        // Wire the inputs..
        self.sender.sig_in.next = self.sig_in.val();
        self.busy.next = self.sender.busy.val();
        self.sender.send.next = self.send.val();
        // Wire the outputs..
        self.sig_out.next = self.recv.sig_out.val();
        self.update.next = self.recv.update.val();
        // Cross wire the two parts
        self.recv.sig_cross.next = self.sender.sig_cross.val();
        self.recv.flag_in.next = self.sender.flag_out.val();
        self.sender.ack_in.next = self.recv.ack_out.val();
    }
}

#[test]
fn test_vec_sync_synthesizable() {
    rust_hdl_yosys_synth::top_wrap!(VectorSynchronizer<Bits<8>>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock_in.connect();
    dev.uut.sig_in.connect();
    dev.uut.send.connect();
    dev.uut.clock_out.connect();
    dev.connect_all();
    rust_hdl_yosys_synth::yosys_validate("vsync", &generate_verilog(&dev)).unwrap();
}
