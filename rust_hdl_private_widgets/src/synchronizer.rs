use rust_hdl__core::prelude::*;

/// A [BitSynchronizer] is used to move signals that are asynchronous to a clock into that
/// clock domain using a pair of back-to-back flip-flops.  While the first flip flop may
/// become metastable, the second one is likely to be stable.
#[derive(LogicBlock, Default)]
pub struct BitSynchronizer {
    /// The input signal, which is asynchronous to the clock
    pub sig_in: Signal<In, Bit>,
    /// The output signal, synchronized to the clock
    pub sig_out: Signal<Out, Bit>,
    /// The clock signal to synchronize the output to
    pub clock: Signal<In, Clock>,
    dff0: DFF<Bit>,
    dff1: DFF<Bit>,
}

impl Logic for BitSynchronizer {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, dff0, dff1);
        self.dff0.d.next = self.sig_in.val();
        self.dff1.d.next = self.dff0.q.val();
        self.sig_out.next = self.dff1.q.val();
    }
}

#[test]
fn sync_is_synthesizable() {
    let mut dev: BitSynchronizer = Default::default();
    dev.connect_all();
    yosys_validate("sync", &generate_verilog(&dev)).unwrap();
}

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
enum SyncSenderState {
    Idle,
    WaitAck,
    WaitDone,
}

/// When you need to send many bits between two clock domains, it is risky to use a vector
/// of [BitSynchronizer] structs.  That is because, you cannot guarantee at any given moment
/// that all of the bits of your multi-bit signal will cross into the new clock domain at once.
/// So to synchronize a multi-bit signal, use a [SyncSender] and [SyncReceiver] pair.  These
/// widgets will use a set of handshake signals to move a value from one clock domain to another
/// safely.  Note that while the state machine is executing, the synchronizer will indicate it
/// is busy.  Crossing clock domains with greater ease is best done with an [AsynchronousFIFO].
#[derive(LogicBlock, Default)]
pub struct SyncSender<T: Synth> {
    /// The input signal to synchronize across clock domains
    pub sig_in: Signal<In, T>,
    /// The input signals are assumed to be synchronous to this clock
    pub clock: Signal<In, Clock>,
    /// These are the wires used to send signals to the [SyncReceiver].
    pub sig_cross: Signal<Out, T>,
    /// A protocol flag signal indicating that data is ready to be transferred to the second clock doamin.
    pub flag_out: Signal<Out, Bit>,
    /// A protocol flag signal indicating that the data has been transferred to the second clock domain.
    pub ack_in: Signal<In, Bit>,
    /// A signal indicating that the [SyncSender] is busy transferring data to the second clock domain.
    pub busy: Signal<Out, Bit>,
    /// A protocol signal - raise this high for one cycle to latch [sig_in].
    pub send: Signal<In, Bit>,
    hold: DFF<T>,
    state: DFF<SyncSenderState>,
    sync: BitSynchronizer,
}

impl<T: Synth> Logic for SyncSender<T> {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, hold, state);
        clock!(self, clock, sync);
        // By default, the hold DFF does not change
        self.sig_cross.next = self.hold.q.val();
        self.flag_out.next = false.into();
        self.sync.sig_in.next = self.ack_in.val();
        // State machine
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
            _ => {
                self.state.d.next = SyncSenderState::Idle;
            }
        }
    }
}

#[test]
fn sync_sender_is_synthesizable() {
    let mut dev: SyncSender<Bits<8>> = Default::default();
    dev.connect_all();
    yosys_validate("sync_send", &generate_verilog(&dev)).unwrap();
}

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
enum SyncReceiverState {
    WaitSteady,
    WaitDone,
}

/// A [SyncReceiver] works together with a [SyncSender] to transmit data from one clock domain
/// to another (in one direction).  To use a [SyncReceiver] wire up the [sig_cross], [flag_in]
/// and [ack_out] signals between the two.
#[derive(LogicBlock, Default)]
pub struct SyncReceiver<T: Synth> {
    /// The data output synchronized to the receiver's clock
    pub sig_out: Signal<Out, T>,
    /// The receivers clock signal.  Data is synchronized to this clock.
    pub clock: Signal<In, Clock>,
    /// The wires used to send data from the [SyncSender] to the [SyncReceiver].
    pub sig_cross: Signal<In, T>,
    /// This is wired up to the [SyncSender::flag_out], and carries the new-data flag.
    pub flag_in: Signal<In, Bit>,
    /// This is wired up to the [SyncSender::ack_in], and carries the acknowledge flag.
    pub ack_out: Signal<Out, Bit>,
    /// This signal will strobe high for one clock when the output is valid and synchronized.
    pub update: Signal<Out, Bit>,
    hold: DFF<T>,
    update_delay: DFF<Bit>,
    state: DFF<SyncReceiverState>,
    sync: BitSynchronizer,
}

impl<T: Synth> Logic for SyncReceiver<T> {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, hold, update_delay, state);
        clock!(self, clock, sync);
        self.sig_out.next = self.hold.q.val();
        self.ack_out.next = false.into();
        self.sync.sig_in.next = self.flag_in.val();
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
            _ => {
                self.state.d.next = SyncReceiverState::WaitSteady;
            }
        }
    }
}

#[test]
fn sync_receiver_is_synthesizable() {
    let mut dev: SyncReceiver<Bits<8>> = Default::default();
    dev.connect_all();
    yosys_validate("sync_recv", &generate_verilog(&dev)).unwrap();
}

/// A [VectorSynchronizer] uses a [SyncSender] and [SyncReceiver] in a matched pair to
/// transmit a vector of bits (or any [Synth] type from one clock domain to a second
/// clock domain without metastability or data corruption.  You can think of a [VectorSynchronizer]
/// as a single-element asynchronous FIFO, and indeed [AsynchronousFIFO] uses the [VectorSynchronizer]
/// internally.
///
/// Note that the [VectorSynchronizer] can be used to reflect a value/register into a
/// second clock domain by tying `self.send.next = !self.busy.val()`.  In that case, the output
/// signal will be always attempting to follow the [sig_in] input as quickly as possible.
#[derive(LogicBlock, Default)]
pub struct VectorSynchronizer<T: Synth> {
    /// The input clock interface.  Input data is clocked in using this clock.
    pub clock_in: Signal<In, Clock>,
    /// The input data interface.  Any synthesizable type can be used here.  This is the data to send.
    pub sig_in: Signal<In, T>,
    /// The busy signal is asserted as long as the synchronizer is, well, synchronizing.  You must
    /// wait until this flag goes low before attempting to send more data.  The [send] signal is
    /// only valid when [busy] is low.
    pub busy: Signal<Out, Bit>,
    /// Raise the [send] signal for a single clock cycle to indicate that the current data on
    /// [sig_in] should be sent across the synchronizer.
    pub send: Signal<In, Bit>,
    /// The clock to use on the output side of the [VectorSynchronizer].  This is the output clock.
    pub clock_out: Signal<In, Clock>,
    /// Data synchronized to the output clock [clock_out].
    pub sig_out: Signal<Out, T>,
    /// The update flag is strobed whenever a new valid output is available on [sig_out].
    pub update: Signal<Out, Bit>,
    sender: SyncSender<T>,
    recv: SyncReceiver<T>,
}

impl<T: Synth> Logic for VectorSynchronizer<T> {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock_in, sender);
        clock!(self, clock_out, recv);
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
    let mut dev: VectorSynchronizer<Bits<8>> = Default::default();
    dev.connect_all();
    yosys_validate("vsync", &generate_verilog(&dev)).unwrap();
}
