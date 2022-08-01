use crate::core::prelude::*;
use crate::core::signed::ToSignedBits;
use crate::dff_setup;
use crate::widgets::dff::DFF;
use crate::widgets::ramrom::ram::RAM;
use crate::widgets::ramrom::sync_rom::SyncROM;

#[derive(Clone, Debug, LogicState, Copy, PartialEq)]
enum MACFIRState {
    Idle,
    Dwell,
    Compute,
    CenterTap,
    Write,
}

#[derive(LogicBlock)]
pub struct MultiplyAccumulateSymmetricFiniteImpulseResponseFilter<const ADDR_BITS: usize> {
    pub data_in: Signal<In, Signed<16>>,
    pub strobe_in: Signal<In, Bit>,
    pub data_out: Signal<Out, Signed<48>>,
    pub strobe_out: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    pub busy: Signal<Out, Bit>,
    coeff_memory: SyncROM<Signed<16>, ADDR_BITS>,
    left_bank: RAM<Signed<16>, ADDR_BITS>,
    right_bank: RAM<Signed<16>, ADDR_BITS>,
    // Points to where the next data sample goes (delay 0)
    head_ptr: DFF<Bits<ADDR_BITS>>,
    // Points to where the left data sample comes from
    left_ptr: Signal<Local, Bits<ADDR_BITS>>,
    // Points to where the right data sample comes from
    right_ptr: Signal<Local, Bits<ADDR_BITS>>,
    // Index pointer used
    index: DFF<Bits<ADDR_BITS>>,
    // Number of iterations (taps-1/2)
    iters: Constant<Bits<ADDR_BITS>>,
    // Size of the data buffer (2**ADDR_BITS - 1)
    bufsize: Constant<Bits<32>>,
    // Number of taps
    taps: Constant<Bits<32>>,
    // Sample from left and right banks
    left_sample: Signal<Local, Signed<16>>,
    right_sample: Signal<Local, Signed<16>>,
    // Accumulator
    accum: DFF<Signed<48>>,
    // FIR state
    state: DFF<MACFIRState>,
    // The output of the MAC slice
    mac_output: Signal<Local, Signed<48>>,
    // The next write location for data
    data_write: Signal<Local, Bits<ADDR_BITS>>,
}

impl<const ADDR_BITS: usize> Logic
    for MultiplyAccumulateSymmetricFiniteImpulseResponseFilter<ADDR_BITS>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clocks
        self.coeff_memory.clock.next = self.clock.val();
        self.left_bank.read_clock.next = self.clock.val();
        self.left_bank.write_clock.next = self.clock.val();
        self.right_bank.read_clock.next = self.clock.val();
        self.right_bank.write_clock.next = self.clock.val();
        dff_setup!(self, clock, head_ptr, index, accum, state);
        // Connect the head pointer to the write address of the two bank memories
        self.left_bank.write_address.next = self.head_ptr.d.val();
        self.right_bank.write_address.next = self.head_ptr.d.val();
        // Both banks receive the same data...
        self.left_bank.write_data.next = self.data_in.val();
        self.right_bank.write_data.next = self.data_in.val();
        // The write enable is controlled by the external strobe
        self.left_bank.write_enable.next = self.strobe_in.val();
        self.right_bank.write_enable.next = self.strobe_in.val();
        // The read on the two banks is different...
        self.left_ptr.next = bit_cast::<{ ADDR_BITS }, 32>(
            bit_cast::<32, { ADDR_BITS }>(self.head_ptr.q.val()) + self.bufsize.val()
                - self.taps.val()
                + 1
                + bit_cast::<32, { ADDR_BITS }>(self.index.q.val()),
        );
        // This is a bit awkward.  We want to do wrapping arithmetic, so we need an extra bit,
        // but because of partial const generic support in Rust, we use 32 bits as an
        // upper bound.  This should synthesize just fine.
        self.right_ptr.next = bit_cast::<{ ADDR_BITS }, 32>(
            bit_cast::<32, { ADDR_BITS }>(self.head_ptr.q.val()) + self.bufsize.val()
                - bit_cast::<32, { ADDR_BITS }>(self.index.q.val()),
        );
        self.left_bank.read_address.next = self.left_ptr.val();
        self.right_bank.read_address.next = self.right_ptr.val();
        self.coeff_memory.address.next = self.index.q.val();
        self.left_sample.next = self.left_bank.read_data.val();
        self.right_sample.next = self.right_bank.read_data.val();
        if self.state.q.val() == MACFIRState::CenterTap {
            self.right_sample.next = 0.into();
        }
        // Wire up the accumulator
        self.mac_output.next = signed_bit_cast::<48, 32>(
            (self.left_sample.val() + self.right_sample.val()) * (self.coeff_memory.data.val()),
        ) + self.accum.q.val();
        if self.state.q.val() == MACFIRState::Idle {
            self.mac_output.next = 0.into();
        }
        // Latch prevention...
        self.data_write.next = self.head_ptr.q.val();
        // The output is wired to the accumulator
        self.data_out.next = self.accum.q.val();
        self.strobe_out.next = false;
        self.busy.next = self.state.q.val() != MACFIRState::Idle;
        // State machine.
        match self.state.q.val() {
            MACFIRState::Idle => {
                if self.strobe_in.val() {
                    self.state.d.next = MACFIRState::Dwell;
                }
            }
            MACFIRState::Dwell => {
                self.index.d.next = self.index.q.val() + 1;
                self.state.d.next = MACFIRState::Compute;
            }
            MACFIRState::Compute => {
                self.index.d.next = self.index.q.val() + 1;
                self.accum.d.next = self.mac_output.val();
                if self.index.q.val() == self.iters.val() {
                    self.state.d.next = MACFIRState::CenterTap;
                }
            }
            MACFIRState::CenterTap => {
                self.index.d.next = self.index.q.val() + 1;
                self.accum.d.next = self.mac_output.val();
                self.state.d.next = MACFIRState::Write;
            }
            MACFIRState::Write => {
                self.strobe_out.next = true;
                self.state.d.next = MACFIRState::Idle;
                // Update the data write location (head pointer)
                self.head_ptr.d.next = self.head_ptr.q.val() + 1;
                // Reset the counter
                self.index.d.next = 0.into();
                self.accum.d.next = 0.into();
            }
            _ => {
                self.state.d.next = MACFIRState::Idle;
            }
        }
        self.data_write.next = self.head_ptr.q.val();
    }
}

impl<const ADDR_BITS: usize> MultiplyAccumulateSymmetricFiniteImpulseResponseFilter<ADDR_BITS> {
    pub fn new(coeffs: &[i16]) -> Self {
        let taps = coeffs.len();
        assert!({ ADDR_BITS } >= clog2(taps));
        // Check for symmetry
        for ndx in 0..coeffs.len() {
            assert_eq!(coeffs[ndx], coeffs[taps - 1 - ndx]);
        }
        // Check for odd length
        assert_eq!(coeffs.len() % 2, 1);
        // Create the compact array
        let clen = (coeffs.len() + 1) / 2;
        let coeff_short = coeffs[0..clen].iter().map(|x| *x).collect::<Vec<_>>();
        let coeffs = coeff_short
            .iter()
            .map(|x| x.to_signed_bits())
            .collect::<Vec<_>>();
        Self {
            data_in: Default::default(),
            strobe_in: Default::default(),
            data_out: Default::default(),
            strobe_out: Default::default(),
            clock: Default::default(),
            busy: Default::default(),
            coeff_memory: coeffs.into_iter().into(),
            left_bank: Default::default(),
            right_bank: Default::default(),
            head_ptr: Default::default(),
            left_ptr: Default::default(),
            right_ptr: Default::default(),
            index: Default::default(),
            iters: Constant::new(((taps - 1) / 2).to_bits()),
            bufsize: Constant::new(Bits::<ADDR_BITS>::count().to_bits()),
            left_sample: Default::default(),
            right_sample: Default::default(),
            accum: Default::default(),
            state: Default::default(),
            mac_output: Default::default(),
            data_write: Default::default(),
            taps: Constant::new(taps.to_bits()),
        }
    }
}

#[test]
fn test_fir_is_synthesizable() {
    let coeffs = [1, -2, 3, -2, 1];
    let mut uut = MultiplyAccumulateSymmetricFiniteImpulseResponseFilter::<3>::new(&coeffs);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("fir", &vlog).unwrap();
}
