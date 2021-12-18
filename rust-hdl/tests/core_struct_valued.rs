use rust_hdl::core::prelude::*;

#[derive(Clone, Debug, PartialEq, Copy, Default)]
pub struct MOSI<const D: usize, const A: usize> {
    pub addr: Bits<A>,
    pub from_master: Bits<D>,
    pub strobe: Bit,
}

impl<const D: usize, const A: usize> Synth for MOSI<D, A> {
    const BITS: usize = Bits::<A>::BITS + Bits::<D>::BITS + Bit::BITS;
    const ENUM_TYPE: bool = false;
    const TYPE_NAME: &'static str = "MOSI_DA";
    const SIGNED: bool = false;

    fn vcd(self) -> VCDValue {
        todo!()
    }

    fn verilog(self) -> VerilogLiteral {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Copy, Default)]
pub struct MISO<const D: usize> {
    pub to_master: Bits<D>,
    pub ready: Bit,
}

impl<const D: usize> Synth for MISO<D> {
    const BITS: usize = Bits::<D>::BITS + Bit::BITS;
    const ENUM_TYPE: bool = false;
    const TYPE_NAME: &'static str = "MISO_D";
    const SIGNED: bool = false;

    fn vcd(self) -> VCDValue {
        todo!()
    }

    fn verilog(self) -> VerilogLiteral {
        todo!()
    }
}

#[derive(LogicBlock, Default)]
pub struct DemoCircuit<const D: usize, const A: usize> {
    pub sig_in: Signal<In, MISO<D>>,
    pub sig_out: Signal<Out, MOSI<D, A>>,
    pub clock: Signal<In, Clock>,
    addr: Signal<Local, Bits<A>>,
}

impl<const D: usize, const A: usize> Logic for DemoCircuit<D, A> {
    #[hdl_gen]
    fn update(&mut self) {
        self.addr.next = 1_usize.into();
        self.sig_out.next.addr = self.addr.val();
        self.sig_out.next.from_master = self.sig_in.val().to_master;
    }
}

#[test]
fn test_demo_synthesizes() {
    let mut uut = DemoCircuit::<16, 8>::default();
    uut.sig_in.connect();
    uut.clock.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("struct", &vlog).unwrap();
}
