use crate::dff::DFF;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;

#[derive(LogicBlock, Default)]
pub struct BitSynchronizer<F: Domain, T: Domain> {
    pub sig_in: Signal<In, Bit, F>,
    pub sig_out: Signal<Out, Bit, T>,
    pub clock: Signal<In, Clock, T>,
    dff0: DFF<Bit, T>,
    dff1: DFF<Bit, T>,
}

impl<F: Domain, T: Domain> Logic for BitSynchronizer<F, T> {
    #[hdl_gen]
    fn update(&mut self) {
        self.dff0.clk.next = self.clock.val();
        self.dff1.clk.next = self.clock.val();

        // Note!  The raw() call here is needed because we
        // _are_ crossing clock domains.  This should be one
        // of the few places you can call it safely!
        self.dff0.d.next = self.sig_in.val().raw().into();
        self.dff1.d.next = self.dff0.q.val();
        self.sig_out.next = self.dff1.q.val();
    }
}

make_domain!(MHz1, 1_000_000);

#[test]
fn sync_is_synthesizable() {
    rust_hdl_synth::top_wrap!(BitSynchronizer<MHz1, MHz1>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.sig_in.connect();
    dev.connect_all();
    yosys_validate("sync", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
pub enum SyncSenderState {
    Sample,
    WaitAck,
    WaitDone,
}

#[derive(LogicBlock, Default)]
pub struct SyncSender<F: Domain, X: Domain, T: Synth> {
    pub sig_in: Signal<In, T, F>,
    pub clock: Signal<In, Clock, F>,
    pub sig_cross: Signal<Out, T, F>,
    pub flag_out: Signal<Out, Bit, F>,
    pub ack_in: Signal<In, Bit, X>,
    hold: DFF<T, F>,
    state: DFF<SyncSenderState, F>,
    sync: BitSynchronizer<X, F>,
}

impl<F: Domain, X: Domain, T: Synth> Logic for SyncSender<F, X, T> {
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
        println!("Current state {:?}", self.state.q.val().raw());
        match self.state.q.val().raw() {
            SyncSenderState::Sample => {
                // Sample the input signal
                self.hold.d.next = self.sig_in.val();
                // Indicate that the output is valid
                self.flag_out.next = true.into();
                self.state.d.next = SyncSenderState::WaitAck.into();
            }
            SyncSenderState::WaitAck => {
                self.flag_out.next = true.into();
                if self.sync.sig_out.val().any() {
                    self.state.d.next = SyncSenderState::WaitDone.into();
                    self.flag_out.next = false.into();
                }
            }
            SyncSenderState::WaitDone => {
                if !self.sync.sig_out.val().any() {
                    self.state.d.next = SyncSenderState::Sample.into();
                }
            }
        }
    }
}

#[test]
fn sync_sender_is_synthesizable() {
    rust_hdl_synth::top_wrap!(SyncSender<MHz1, MHz1, Bits<8>>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.sig_in.connect();
    dev.uut.ack_in.connect();
    dev.connect_all();
    println!("{}", generate_verilog(&dev));
    yosys_validate("sync_send", &generate_verilog(&dev)).unwrap();
}

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
pub enum SyncReceiverState {
    WaitSteady,
    WaitDone,
}

#[derive(LogicBlock, Default)]
pub struct SyncReceiver<F: Domain, X: Domain, T: Synth> {
    pub sig_out: Signal<Out, T, X>,
    pub clock: Signal<In, Clock, X>,
    pub sig_cross: Signal<In, T, F>,
    pub flag_in: Signal<In, Bit, F>,
    pub ack_out: Signal<Out, Bit, X>,
    hold: DFF<T, X>,
    state: DFF<SyncReceiverState, X>,
    sync: BitSynchronizer<F, X>,
}

impl<F: Domain, X: Domain, T: Synth> Logic for SyncReceiver<F, X, T> {
    #[hdl_gen]
    fn update(&mut self) {
        self.state.clk.next = self.clock.val();
        self.sync.clock.next = self.clock.val();
        self.hold.clk.next = self.clock.val();

        self.hold.d.next = self.hold.q.val();
        self.sig_out.next = self.hold.q.val();
        self.ack_out.next = false.into();
        self.sync.sig_in.next = self.flag_in.val();

        self.state.d.next = self.state.q.val();
        match self.state.q.val().raw() {
            SyncReceiverState::WaitSteady => {
                if self.sync.sig_out.val().any() {
                    self.ack_out.next = true.into();
                    self.state.d.next = SyncReceiverState::WaitDone.into();
                }
            }
            SyncReceiverState::WaitDone => {
                if !self.sync.sig_out.val().any() {
                    self.hold.d.next = self.sig_cross.val().raw().into();
                    self.ack_out.next = false.into();
                    self.state.d.next = SyncReceiverState::WaitSteady.into();
                } else {
                    self.ack_out.next = true.into();
                }
            }
        }
    }
}

#[test]
fn sync_receiver_is_synthesizable() {
    rust_hdl_synth::top_wrap!(SyncReceiver<MHz1, MHz1, Bits<8>>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.sig_cross.connect();
    dev.uut.flag_in.connect();
    dev.connect_all();
    println!("{}", generate_verilog(&dev));
    yosys_validate("sync_recv", &generate_verilog(&dev)).unwrap();
}
