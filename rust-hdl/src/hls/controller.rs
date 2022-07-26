use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadController, FIFOWriteController, SoCBusController};
use crate::widgets::prelude::*;

// For now, we will hard code the op codes
// 00 - NOOP
// 01 - PING
// 02 - READ
// 03 - WRITE
// 04 - POLL
// 05 - STREAM (send any non-zero value to stop streaming)

#[derive(LogicState, Debug, Copy, Clone, PartialEq)]
enum BaseControllerState {
    Idle,
    Ping,
    ReadLoadCount,
    Read,
    Write,
    WriteLoadCount,
    PollWait,
    Poll,
    StreamWait,
    Stream,
}

// This version of the SOCController takes 8-bit sequences as inputs,
// and communicates with a 16 bit bus.  Other designs are possible,
// but the internal logic needs to handle the differences in address
// space bits, data widths, etc.
#[derive(LogicBlock, Default)]
pub struct BaseController<const A: usize> {
    pub from_cpu: FIFOReadController<Bits<16>>, // Word-stream from the CPU
    pub to_cpu: FIFOWriteController<Bits<16>>,  // Word-stream to the CPU
    pub clock: Signal<In, Clock>,               // All in a single clock domain
    state: DFF<BaseControllerState>,
    pub bus: SoCBusController<16, { A }>,
    counter: DFF<Bits<16>>,
    opcode: Signal<Local, Bits<8>>,
}

impl<const A: usize> Logic for BaseController<A> {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, state, counter);
        // Latch prevention
        self.opcode.next = self.from_cpu.data.val().get_bits::<8>(8);
        // Default values for output signals.
        self.from_cpu.read.next = false;
        self.to_cpu.data.next = 0.into();
        self.to_cpu.write.next = false;
        self.bus.clock.next = self.clock.val();
        self.bus.from_controller.next = 0.into();
        self.bus.strobe.next = false;
        self.bus.address.next = 0.into();
        self.bus.address_strobe.next = false;
        match self.state.q.val() {
            BaseControllerState::Idle => {
                if !self.from_cpu.empty.val() {
                    if self.opcode.val() == 0 {
                        // Skip opcodes that are NOOP
                        self.from_cpu.read.next = true;
                    } else if self.opcode.val() == 1 {
                        self.state.d.next = BaseControllerState::Ping;
                    } else if self.opcode.val() == 2 {
                        // Latch the address
                        self.bus.address.next = self.from_cpu.data.val().get_bits::<A>(0);
                        self.bus.address_strobe.next = true;
                        self.from_cpu.read.next = true;
                        self.state.d.next = BaseControllerState::ReadLoadCount;
                    } else if self.opcode.val() == 3 {
                        // Latch the address
                        self.bus.address.next = self.from_cpu.data.val().get_bits::<A>(0);
                        self.bus.address_strobe.next = true;
                        self.from_cpu.read.next = true;
                        self.state.d.next = BaseControllerState::WriteLoadCount;
                    } else if self.opcode.val() == 4 {
                        self.bus.address.next = self.from_cpu.data.val().get_bits::<A>(0);
                        self.bus.address_strobe.next = true;
                        self.from_cpu.read.next = true;
                        self.state.d.next = BaseControllerState::PollWait;
                    } else if self.opcode.val() == 5 {
                        self.bus.address.next = self.from_cpu.data.val().get_bits::<A>(0);
                        self.bus.address_strobe.next = true;
                        self.from_cpu.read.next = true;
                        self.state.d.next = BaseControllerState::StreamWait;
                    }
                }
            }
            BaseControllerState::Ping => {
                self.to_cpu.data.next = self.from_cpu.data.val();
                self.to_cpu.write.next = true;
                self.from_cpu.read.next = true;
                self.state.d.next = BaseControllerState::Idle;
            }
            BaseControllerState::ReadLoadCount => {
                if !self.from_cpu.empty.val() {
                    self.counter.d.next = self.from_cpu.data.val();
                    self.from_cpu.read.next = true;
                    self.state.d.next = BaseControllerState::Read;
                }
            }
            BaseControllerState::Read => {
                if self.bus.ready.val() & !self.to_cpu.full.val() {
                    self.to_cpu.data.next = self.bus.to_controller.val();
                    self.bus.strobe.next = true;
                    self.to_cpu.write.next = true;
                    self.counter.d.next = self.counter.q.val() - 1;
                    if self.counter.q.val() == 1 {
                        self.state.d.next = BaseControllerState::Idle;
                    }
                }
            }
            BaseControllerState::WriteLoadCount => {
                if !self.from_cpu.empty.val() {
                    self.counter.d.next = self.from_cpu.data.val();
                    self.from_cpu.read.next = true;
                    self.state.d.next = BaseControllerState::Write;
                }
            }
            BaseControllerState::Write => {
                if self.bus.ready.val() & !self.from_cpu.empty.val() {
                    self.bus.from_controller.next = self.from_cpu.data.val();
                    self.bus.strobe.next = true;
                    self.from_cpu.read.next = true;
                    self.counter.d.next = self.counter.q.val() - 1;
                    if self.counter.q.val() == 1 {
                        self.state.d.next = BaseControllerState::Idle;
                    }
                }
            }
            BaseControllerState::PollWait => {
                self.state.d.next = BaseControllerState::Poll;
            }
            BaseControllerState::Poll => {
                if !self.to_cpu.full.val() {
                    self.to_cpu.data.next =
                        bits::<16>(0xFF00) | bit_cast::<16, 1>(self.bus.ready.val().into());
                    self.to_cpu.write.next = true;
                    self.state.d.next = BaseControllerState::Idle;
                }
            }
            BaseControllerState::StreamWait => {
                self.state.d.next = BaseControllerState::Stream;
            }
            BaseControllerState::Stream => {
                if self.bus.ready.val() & !self.to_cpu.full.val() {
                    self.to_cpu.data.next = self.bus.to_controller.val();
                    self.bus.strobe.next = true;
                    self.to_cpu.write.next = true;
                }
                if !self.from_cpu.empty.val() {
                    if self.from_cpu.data.val().any() {
                        self.state.d.next = BaseControllerState::Idle;
                    }
                    self.from_cpu.read.next = true;
                }
            }
            _ => {
                self.state.d.next = BaseControllerState::Idle;
            }
        }
    }
}

#[test]
fn test_base_controller_is_synthesizable() {
    let mut uut = BaseController::<4>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("base_controller", &vlog).unwrap();
}
