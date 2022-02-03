use crate::core::prelude::*;
use crate::widgets::prelude::*;
use std::time::Duration;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum State {
    Idle,
    AddressCheck,
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
    incoming: DFF<Bits<8>>,
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
            incoming: Default::default(),
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
        self.sda.link(&mut self.phy.sda);
        self.scl.link(&mut self.phy.scl);
        // Clock internal logic
        self.phy.clock.next = self.clock.val();
        self.mem.read_clock.next = self.clock.val();
        self.mem.write_clock.next = self.clock.val();
        self.ptr.clk.next = self.clock.val();
        self.incoming.clk.next = self.clock.val();
        self.outgoing.clk.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.save.clk.next = self.clock.val();
        self.active.clk.next = self.clock.val();
        // Latch prevention
        self.incoming.d.next = self.incoming.q.val();
        self.outgoing.d.next = self.outgoing.q.val();
        self.state.d.next = self.state.q.val();
        self.ptr.d.next = self.ptr.q.val();
        self.save.d.next = self.save.q.val();
        self.active.d.next = self.active.q.val();
        // Wire up the RAM
        self.mem.write_data.next = bit_cast::<16, 8>(self.save.q.val()) << 8_usize
            | bit_cast::<16, 8>(self.incoming.q.val());
        self.mem.write_enable.next = false;
        self.mem.write_address.next = self.ptr.q.val();
        self.mem.read_address.next = self.ptr.q.val();
        self.phy.active.next = self.active.q.val();
        // Default controls
        match self.state.q.val() {
            State::Idle => {
                if self.phy.bus_write.val() {
                    // Check if the address matches
                    if self.phy.from_bus.val().get_bits::<7>(1) == self.address.val() {
                        self.active.d.next = true;
                    } else {
                        self.active.d.next = false;
                    }
                    self.incoming.d.next = self.phy.from_bus.val();
                }
            }
            _ => {}
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
