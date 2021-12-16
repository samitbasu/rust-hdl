// A simple, local bus for attaching stuff together on the FPGA
use crate::core::prelude::*;
use crate::widgets::dff::DFF;

#[derive(Clone, Debug, Default, LogicInterface)]
pub struct LocalBusM<const D: usize, const A: usize> {
    pub addr: Signal<Out, Bits<A>>,
    pub from_master: Signal<Out, Bits<D>>,
    pub to_master: Signal<In, Bits<D>>,
    pub ready: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
pub struct LocalBusD<const D: usize, const A: usize> {
    pub addr: Signal<In, Bits<A>>,
    pub from_master: Signal<In, Bits<D>>,
    pub to_master: Signal<Out, Bits<D>>,
    pub ready: Signal<Out, Bit>,
    pub strobe: Signal<In, Bit>,
}

// An input port simply stores the value written to it's input back to
// the master.  The address comparison logic is registered to improve the
// timing analysis of the bus.
#[derive(LogicBlock)]
pub struct MISOPort<const D: usize, const A: usize> {
    pub bus: LocalBusD<D, A>,
    pub port_in: Signal<In, Bits<D>>,
    pub clock: Signal<In, Clock>,
    pub ready_in: Signal<In, Bit>,
    pub strobe_out: Signal<Out, Bit>,
    address_active: DFF<Bit>,
    my_address: Constant<Bits<A>>,
}

impl<const D: usize, const A: usize> MISOPort<D, A> {
    pub fn new(address: Bits<A>) -> Self {
        Self {
            bus: Default::default(),
            port_in: Default::default(),
            clock: Default::default(),
            ready_in: Default::default(),
            strobe_out: Default::default(),
            address_active: Default::default(),
            my_address: Constant::new(address),
        }
    }
}

impl<const D: usize, const A: usize> Logic for MISOPort<D, A> {
    #[hdl_gen]
    fn update(&mut self) {
        self.address_active.clk.next = self.clock.val();
        self.address_active.d.next = self.bus.addr.val() == self.my_address.val();
        self.bus.to_master.next = 0_usize.into();
        self.bus.ready.next = false;
        self.strobe_out.next = false;
        if self.address_active.q.val() {
            self.bus.ready.next = self.ready_in.val();
            self.bus.to_master.next = self.port_in.val();
            self.strobe_out.next = self.bus.strobe.val();
        }
    }
}

// An output port simply stores the value written to that memory location
// by the master.  The value is latched.
// The strobe from the master is also forwarded.  This allows you to
// build logic that knows when the value was changed, or treat the
// strobe like a trigger.
#[derive(LogicBlock)]
pub struct MOSIPort<const D: usize, const A: usize> {
    pub bus: LocalBusD<D, A>,
    pub port_out: Signal<Out, Bits<D>>,
    pub clock: Signal<In, Clock>,
    pub strobe_out: Signal<Out, Bit>,
    pub ready: Signal<In, Bit>,
    state: DFF<Bits<D>>,
    address_active: DFF<Bit>,
    my_address: Constant<Bits<A>>,
    strobe: DFF<Bit>,
}

impl<const D: usize, const A: usize> MOSIPort<D, A> {
    pub fn new(address: Bits<A>) -> Self {
        Self {
            bus: Default::default(),
            port_out: Default::default(),
            clock: Default::default(),
            strobe_out: Default::default(),
            ready: Default::default(),
            state: Default::default(),
            address_active: Default::default(),
            my_address: Constant::new(address),
            strobe: Default::default(),
        }
    }
}

impl<const D: usize, const A: usize> Logic for MOSIPort<D, A> {
    #[hdl_gen]
    fn update(&mut self) {
        self.state.clk.next = self.clock.val();
        self.strobe.clk.next = self.clock.val();
        self.address_active.clk.next = self.clock.val();
        self.port_out.next = self.state.q.val();
        self.state.d.next = self.state.q.val();
        self.address_active.d.next = self.bus.addr.val() == self.my_address.val();
        self.bus.ready.next = false;
        self.strobe_out.next = self.strobe.q.val();
        self.strobe.d.next = false;
        if self.address_active.q.val() {
            self.bus.ready.next = self.ready.val();
            if self.bus.strobe.val() {
                self.state.d.next = self.bus.from_master.val();
            }
            self.strobe.d.next = self.bus.strobe.val();
        }
        self.bus.to_master.next = 0_usize.into();
    }
}

#[test]
fn test_local_out_port_is_synthesizable() {
    let mut dev = MOSIPort::<16, 8>::new(53_u8.into());
    dev.bus.from_master.connect();
    dev.bus.addr.connect();
    dev.clock.connect();
    dev.bus.strobe.connect();
    dev.connect_all();
    dev.ready.connect();
    let vlog = generate_verilog(&dev);
    println!("{}", vlog);
    yosys_validate("localout", &vlog).unwrap();
}

#[test]
fn test_local_in_port_is_synthesizable() {
    let mut dev = MISOPort::<16, 8>::new(53_u8.into());
    dev.bus.from_master.connect();
    dev.bus.addr.connect();
    dev.clock.connect();
    dev.bus.strobe.connect();
    dev.port_in.connect();
    dev.connect_all();
    let vlog = generate_verilog(&dev);
    println!("{}", vlog);
    yosys_validate("localin", &vlog).unwrap();
}

#[derive(LogicBlock)]
pub struct MOSIWidePort<const W: usize, const D: usize, const A: usize> {
    pub bus: LocalBusD<D, A>,
    pub clock: Signal<In, Clock>,
    pub port_out: Signal<Out, Bits<W>>,
    pub strobe_out: Signal<Out, Bit>,
    accum: DFF<Bits<W>>,
    state: DFF<Bits<W>>,
    address_active: DFF<Bit>,
    my_address: Constant<Bits<A>>,
    offset: Constant<Bits<W>>,
    modulo: Constant<Bits<8>>,
    count: DFF<Bits<8>>,
    strobe: DFF<Bit>,
}

impl<const W: usize, const D: usize, const A: usize> MOSIWidePort<W, D, A> {
    pub fn new(addr: Bits<A>) -> Self {
        assert!(W > D);
        assert_eq!(W % D, 0);
        assert!(W / D < 256);
        Self {
            bus: Default::default(),
            clock: Default::default(),
            port_out: Default::default(),
            strobe_out: Default::default(),
            accum: Default::default(),
            state: Default::default(),
            address_active: Default::default(),
            my_address: Constant::new(addr),
            offset: Constant::new(D.into()),
            modulo: Constant::new((W / D - 1).into()),
            count: Default::default(),
            strobe: Default::default(),
        }
    }
}

impl<const W: usize, const D: usize, const A: usize> Logic for MOSIWidePort<W, D, A> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal flip flops
        self.accum.clk.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.count.clk.next = self.clock.val();
        self.strobe.clk.next = self.clock.val();
        self.address_active.clk.next = self.clock.val();
        // Compute the select/enable flag
        self.address_active.d.next = self.bus.addr.val() == self.my_address.val();
        // Latch prevention
        self.count.d.next = self.count.q.val();
        self.accum.d.next = self.accum.q.val();
        self.state.d.next = self.state.q.val();
        self.bus.ready.next = false;
        self.strobe_out.next = self.strobe.q.val();
        self.strobe.d.next = false;
        if self.address_active.q.val() {
            self.bus.ready.next = true;
            if self.bus.strobe.val() {
                self.accum.d.next = self.accum.q.val() << self.offset.val()
                    | bit_cast::<W, D>(self.bus.from_master.val());
                self.count.d.next = self.count.q.val() + 1_usize;
                if self.count.q.val() == self.modulo.val() {
                    self.count.d.next = 0_u8.into();
                    self.state.d.next = self.accum.q.val();
                    self.strobe.d.next = true;
                }
            }
        }
        self.port_out.next = self.accum.q.val();
        self.bus.to_master.next = 0_usize.into();
    }
}

#[derive(LogicBlock)]
pub struct MISOWidePort<const W: usize, const D: usize, const A: usize> {
    pub bus: LocalBusD<D, A>,
    pub port_in: Signal<In, Bits<W>>,
    pub strobe_in: Signal<In, Bit>,
    pub clock: Signal<In, Clock>,
    accum: DFF<Bits<W>>,
    address_active: DFF<Bit>,
    my_address: Constant<Bits<A>>,
    offset: Constant<Bits<W>>,
    shift: Constant<Bits<W>>,
    modulo: Constant<Bits<8>>,
    count: DFF<Bits<8>>,
    ready: DFF<Bit>,
}

impl<const W: usize, const D: usize, const A: usize> MISOWidePort<W, D, A> {
    pub fn new(addr: Bits<A>) -> Self {
        assert!(W > D);
        assert_eq!(W % D, 0);
        assert!(W / D < 256);
        Self {
            bus: Default::default(),
            port_in: Default::default(),
            strobe_in: Default::default(),
            clock: Default::default(),
            accum: Default::default(),
            address_active: Default::default(),
            my_address: Constant::new(addr),
            offset: Constant::new(D.into()),
            shift: Constant::new((W - D).into()),
            modulo: Constant::new((W / D).into()),
            count: Default::default(),
            ready: Default::default(),
        }
    }
}

impl<const W: usize, const D: usize, const A: usize> Logic for MISOWidePort<W, D, A> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal flip flops
        self.accum.clk.next = self.clock.val();
        self.address_active.clk.next = self.clock.val();
        self.count.clk.next = self.clock.val();
        self.ready.clk.next = self.clock.val();
        // Latch prevention
        self.accum.d.next = self.accum.q.val();
        self.address_active.d.next = self.bus.addr.val() == self.my_address.val();
        self.count.d.next = self.count.q.val();
        self.bus.ready.next = false;
        // On the strobe in, load the new value into our accumulator
        if self.strobe_in.val() {
            self.accum.d.next = self.port_in.val();
            self.count.d.next = self.modulo.val();
        }
        self.bus.to_master.next = 0_usize.into();
        self.ready.d.next = self.count.q.val().any() & self.address_active.q.val();
        if self.address_active.q.val() {
            self.bus.to_master.next = self.accum.q.val().get_bits::<D>(self.shift.val().into());
            self.bus.ready.next = self.ready.q.val() & self.count.q.val().any();
            if self.bus.strobe.val() {
                self.accum.d.next = self.accum.q.val() << self.offset.val();
                self.count.d.next = self.count.q.val() - 1_usize;
            }
        }
    }
}

#[test]
fn test_local_in_wide_port_is_synthesizable() {
    let mut dev = MISOWidePort::<64, 16, 8>::new(53_u8.into());
    dev.bus.from_master.connect();
    dev.bus.addr.connect();
    dev.bus.strobe.connect();
    dev.clock.connect();
    dev.port_in.connect();
    dev.strobe_in.connect();
    dev.connect_all();
    let vlog = generate_verilog(&dev);
    println!("{}", vlog);
    yosys_validate("local_wide_in", &vlog).unwrap();
}
