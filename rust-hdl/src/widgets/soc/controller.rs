use crate::core::prelude::*;
use crate::widgets::bidirectional_bus::FifoBusIn;
use crate::widgets::dff::DFF;
use crate::widgets::soc::bus::SoCBusController;

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
    pub cpu: FifoBusIn<Bits<16>>, // Word-stream from/to the CPU
    pub clock: Signal<In, Clock>, // All in a single clock domain
    state: DFF<BaseControllerState>,
    pub bus: SoCBusController<16, { A }>,
    counter: DFF<Bits<16>>,
    opcode: Signal<Local, Bits<8>>,
}

impl<const A: usize> Logic for BaseController<A> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the logic
        self.state.clk.next = self.clock.val();
        self.counter.clk.next = self.clock.val();
        // Latch prevention
        self.state.d.next = self.state.q.val();
        self.opcode.next = self.cpu.from_bus.val().get_bits::<8>(8);
        self.counter.d.next = self.counter.q.val();
        // Default values for output signals.
        self.cpu.read.next = false;
        self.cpu.to_bus.next = 0_usize.into();
        self.cpu.write.next = false;
        self.bus.clock.next = self.clock.val();
        self.bus.from_controller.next = 0_usize.into();
        self.bus.strobe.next = false;
        self.bus.address.next = 0_usize.into();
        self.bus.address_strobe.next = false;
        match self.state.q.val() {
            BaseControllerState::Idle => {
                if !self.cpu.empty.val() {
                    if self.opcode.val() == 0_u16 {
                        // Skip opcodes that are NOOP
                        self.cpu.read.next = true;
                    } else if self.opcode.val() == 1_u8 {
                        self.state.d.next = BaseControllerState::Ping;
                    } else if self.opcode.val() == 2_u8 {
                        // Latch the address
                        self.bus.address.next = self.cpu.from_bus.val().get_bits::<A>(0);
                        self.bus.address_strobe.next = true;
                        self.cpu.read.next = true;
                        self.state.d.next = BaseControllerState::ReadLoadCount;
                    } else if self.opcode.val() == 3_u8 {
                        // Latch the address
                        self.bus.address.next = self.cpu.from_bus.val().get_bits::<A>(0);
                        self.bus.address_strobe.next = true;
                        self.cpu.read.next = true;
                        self.state.d.next = BaseControllerState::WriteLoadCount;
                    } else if self.opcode.val() == 4_u8 {
                        self.bus.address.next = self.cpu.from_bus.val().get_bits::<A>(0);
                        self.bus.address_strobe.next = true;
                        self.cpu.read.next = true;
                        self.state.d.next = BaseControllerState::PollWait;
                    } else if self.opcode.val() == 5_u8 {
                        self.bus.address.next = self.cpu.from_bus.val().get_bits::<A>(0);
                        self.bus.address_strobe.next = true;
                        self.cpu.read.next = true;
                        self.state.d.next = BaseControllerState::StreamWait;
                    }
                }
            }
            BaseControllerState::Ping => {
                self.cpu.to_bus.next = self.cpu.from_bus.val();
                self.cpu.write.next = true;
                self.cpu.read.next = true;
                self.state.d.next = BaseControllerState::Idle;
            }
            BaseControllerState::ReadLoadCount => {
                if !self.cpu.empty.val() {
                    self.counter.d.next = self.cpu.from_bus.val();
                    self.cpu.read.next = true;
                    self.state.d.next = BaseControllerState::Read;
                }
            }
            BaseControllerState::Read => {
                if self.bus.ready.val() & !self.cpu.full.val() {
                    self.cpu.to_bus.next = self.bus.to_controller.val();
                    self.bus.strobe.next = true;
                    self.cpu.write.next = true;
                    self.counter.d.next = self.counter.q.val() - 1_u32;
                    if self.counter.q.val() == 1_usize {
                        self.state.d.next = BaseControllerState::Idle;
                    }
                }
            }
            BaseControllerState::WriteLoadCount => {
                if !self.cpu.empty.val() {
                    self.counter.d.next = self.cpu.from_bus.val();
                    self.cpu.read.next = true;
                    self.state.d.next = BaseControllerState::Write;
                }
            }
            BaseControllerState::Write => {
                if self.bus.ready.val() & !self.cpu.empty.val() {
                    self.bus.from_controller.next = self.cpu.from_bus.val();
                    self.bus.strobe.next = true;
                    self.cpu.read.next = true;
                    self.counter.d.next = self.counter.q.val() - 1_u32;
                    if self.counter.q.val() == 1_usize {
                        self.state.d.next = BaseControllerState::Idle;
                    }
                }
            }
            BaseControllerState::PollWait => {
                self.state.d.next = BaseControllerState::Poll;
            }
            BaseControllerState::Poll => {
                if !self.cpu.full.val() {
                    self.cpu.to_bus.next = bits::<16>(0xFF00)
                        | bit_cast::<16, 1>(self.bus.ready.val().into());
                    self.cpu.write.next = true;
                    self.state.d.next = BaseControllerState::Idle;
                }
            }
            BaseControllerState::StreamWait => {
                self.state.d.next = BaseControllerState::Stream;
            }
            BaseControllerState::Stream => {
                if self.bus.ready.val() & !self.cpu.full.val() {
                    self.cpu.to_bus.next = self.bus.to_controller.val();
                    self.bus.strobe.next = true;
                    self.cpu.write.next = true;
                }
                if !self.cpu.empty.val() {
                    if self.cpu.from_bus.val().any() {
                        self.state.d.next = BaseControllerState::Idle;
                    }
                    self.cpu.read.next = true;
                }
            }
        }
    }
}

#[test]
fn test_base_controller_is_synthesizable() {
    let mut uut = BaseController::<4>::default();
    uut.clock.connect();
    uut.cpu.from_bus.connect();
    uut.cpu.empty.connect();
    uut.cpu.full.connect();
    uut.bus.to_controller.connect();
    uut.bus.ready.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("base_controller", &vlog).unwrap();
}
