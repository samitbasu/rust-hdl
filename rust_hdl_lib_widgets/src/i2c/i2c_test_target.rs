use array_init::array_init;

use crate::prelude::*;
use rust_hdl_lib_core::prelude::*;

#[derive(LogicBlock)]
pub struct I2CTestBus<const N: usize> {
    pub endpoints: [I2CBusReceiver; N],
    pub sda_state: Signal<Local, Bit>,
    pub scl_state: Signal<Local, Bit>,
}

impl<const N: usize> Default for I2CTestBus<N> {
    fn default() -> Self {
        Self {
            endpoints: array_init(|_| Default::default()),
            sda_state: Default::default(),
            scl_state: Default::default(),
        }
    }
}

impl<const N: usize> Logic for I2CTestBus<N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.sda_state.next = true;
        self.scl_state.next = true;
        for ndx in 0..N {
            self.sda_state.next = self.sda_state.val() & !self.endpoints[ndx].sda.drive_low.val();
            self.scl_state.next = self.scl_state.val() & !self.endpoints[ndx].scl.drive_low.val();
        }
        for ndx in 0..N {
            self.endpoints[ndx].sda.line_state.next = self.sda_state.val();
            self.endpoints[ndx].scl.line_state.next = self.scl_state.val();
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    GetPointer,
    GetMSB,
    GetLSB,
    WriteMSB,
    WriteLSB,
    CheckMSBAck,
    CheckLSBAck,
}

// Provides a simple read/write memory on an I2C bus
// The memory is 16 bits wide, and there are 16 addresses.
#[derive(LogicBlock)]
pub struct I2CTestTarget {
    // The I2C data lines must have external pullups.
    pub i2c: I2CBusDriver,
    pub clock: Signal<In, Clock>,
    phy: I2CTarget,
    mem: RAM<Bits<16>, 4>,
    ptr: DFF<Bits<4>>,
    address: Constant<Bits<7>>,
    outgoing: DFF<Bits<8>>,
    save: DFF<Bits<8>>,
    state: DFF<State>,
    active: DFF<Bit>,
}

impl I2CTestTarget {
    pub fn new(address: u8) -> Self {
        assert_eq!(address & 0x80, 0, "I2C addresses must be 7 bits");
        Self {
            i2c: Default::default(),
            clock: Default::default(),
            phy: Default::default(),
            mem: Default::default(),
            ptr: Default::default(),
            address: Constant::new(address.to_bits()),
            outgoing: Default::default(),
            save: Default::default(),
            state: Default::default(),
            active: Default::default(),
        }
    }
}

impl Logic for I2CTestTarget {
    #[hdl_gen]
    fn update(&mut self) {
        I2CBusDriver::link(&mut self.i2c, &mut self.phy.i2c);
        // Clock internal logic
        clock!(self, clock, phy);
        self.mem.read_clock.next = self.clock.val();
        self.mem.write_clock.next = self.clock.val();
        dff_setup!(self, clock, ptr, outgoing, save, state, active);
        // Latch prevention
        // Wire up the RAM
        self.mem.write_data.next =
            bit_cast::<16, 8>(self.save.q.val()) << 8 | bit_cast::<16, 8>(self.phy.from_bus.val());
        self.mem.write_enable.next = false;
        self.mem.write_address.next = self.ptr.q.val();
        self.mem.read_address.next = self.ptr.q.val();
        self.phy.active.next = self.active.q.val();
        self.phy.to_bus.next = 0.into();
        self.phy.write_enable.next = false;
        // Default controls
        match self.state.q.val() {
            State::Idle => {
                if self.phy.bus_write.val() {
                    // Check if the address matches
                    if self.phy.from_bus.val().get_bits::<7>(1) == self.address.val() {
                        self.active.d.next = true;
                        if !self.phy.from_bus.val().get_bit(0) {
                            self.state.d.next = State::GetPointer;
                        } else {
                            self.state.d.next = State::WriteMSB;
                        }
                    } else {
                        self.active.d.next = false;
                    }
                }
            }
            State::GetPointer => {
                if self.phy.bus_write.val() {
                    self.ptr.d.next = self.phy.from_bus.val().get_bits::<4>(0);
                    self.state.d.next = State::GetMSB;
                }
            }
            State::GetMSB => {
                if self.phy.bus_write.val() {
                    self.save.d.next = self.phy.from_bus.val();
                    self.state.d.next = State::GetLSB;
                }
            }
            State::GetLSB => {
                self.mem.write_enable.next = self.phy.bus_write.val();
                if self.phy.bus_write.val() {
                    self.state.d.next = State::Idle;
                }
            }
            State::WriteMSB => {
                if self.phy.write_ok.val() {
                    self.phy.to_bus.next = self.mem.read_data.val().get_bits::<8>(8);
                    self.phy.write_enable.next = true;
                    self.state.d.next = State::CheckMSBAck;
                }
            }
            State::CheckMSBAck => {
                if self.phy.ack.val() {
                    self.state.d.next = State::WriteLSB;
                }
                if self.phy.nack.val() {
                    self.state.d.next = State::Idle;
                }
            }
            State::WriteLSB => {
                if self.phy.write_ok.val() {
                    self.phy.to_bus.next = self.mem.read_data.val().get_bits::<8>(0);
                    self.phy.write_enable.next = true;
                    self.state.d.next = State::CheckLSBAck;
                }
            }
            State::CheckLSBAck => {
                if self.phy.ack.val() {
                    self.state.d.next = State::Idle;
                }
                if self.phy.nack.val() {
                    self.state.d.next = State::Idle;
                }
            }
            _ => {
                self.state.d.next = State::Idle;
            }
        }
        if self.phy.stop.val() {
            self.state.d.next = State::Idle;
            self.active.d.next = false;
        }
    }
}

#[test]
fn test_i2c_test_target_synthesizable() {
    let mut uut = I2CTestTarget::new(0x53);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("i2c_test_target", &vlog).unwrap()
}
