use crate::core::prelude::*;
use crate::widgets::prelude::*;

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
    // The I2C data line.  Must have an external pullup
    pub sda: Signal<InOut, Bit>,
    // The I2C Clock line.  Must have an external pullup
    pub scl: Signal<InOut, Bit>,
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
            sda: Default::default(),
            scl: Default::default(),
            clock: Default::default(),
            phy: Default::default(),
            mem: Default::default(),
            ptr: Default::default(),
            address: Constant::new(address.into()),
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
        Signal::<InOut, Bit>::link(&mut self.sda,&mut self.phy.sda);
        Signal::<InOut, Bit>::link(&mut self.scl,&mut self.phy.scl);
        // Clock internal logic
        self.phy.clock.next = self.clock.val();
        self.mem.read_clock.next = self.clock.val();
        self.mem.write_clock.next = self.clock.val();
        self.ptr.clk.next = self.clock.val();
        self.outgoing.clk.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.save.clk.next = self.clock.val();
        self.active.clk.next = self.clock.val();
        // Latch prevention
        self.outgoing.d.next = self.outgoing.q.val();
        self.state.d.next = self.state.q.val();
        self.ptr.d.next = self.ptr.q.val();
        self.save.d.next = self.save.q.val();
        self.active.d.next = self.active.q.val();
        // Wire up the RAM
        self.mem.write_data.next = bit_cast::<16, 8>(self.save.q.val()) << 8_usize
            | bit_cast::<16, 8>(self.phy.from_bus.val());
        self.mem.write_enable.next = false;
        self.mem.write_address.next = self.ptr.q.val();
        self.mem.read_address.next = self.ptr.q.val();
        self.phy.active.next = self.active.q.val();
        self.phy.to_bus.next = 0_usize.into();
        self.phy.write_enable.next = false;
        // Default controls
        match self.state.q.val() {
            State::Idle => {
                if self.phy.bus_write.val() {
                    // Check if the address matches
                    if self.phy.from_bus.val().get_bits::<7>(1) == self.address.val() {
                        self.active.d.next = true;
                        if !self.phy.from_bus.val().get_bit(0_usize) {
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
                    self.phy.to_bus.next = self.mem.read_data.val().get_bits::<8>(8_usize);
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
                    self.phy.to_bus.next = self.mem.read_data.val().get_bits::<8>(0_usize);
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
        }
        if self.phy.stop.val() {
            self.state.d.next = State::Idle;
            self.active.d.next = false;
        }
    }
}

#[test]
fn test_i2c_test_target_synthesizable() {
    let mut uut = TopWrap::new(I2CTestTarget::new(0x53));
    uut.uut.clock.connect();
    uut.uut.sda.connect();
    uut.uut.scl.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("i2c_test_target", &vlog).unwrap()
}
