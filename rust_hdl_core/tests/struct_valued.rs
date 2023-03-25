use rust_hdl_core::prelude::*;

// We want to be able to combine a set of signals into a struct
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, LogicState)]
enum CmdType {
    Noop,
    Read,
    Write,
}

#[cfg(test)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, LogicStruct)]
struct MIGCmd {
    pub cmd: CmdType,
    pub active: Bit,
    pub len: Bits<6>,
}

#[test]
fn test_composite() {
    assert_eq!(MIGCmd::BITS, 9);
    let x = MIGCmd {
        cmd: CmdType::Read,
        active: true,
        len: 35.into(),
    };

    let y: Bits<9> = x.into();
    assert_eq!(y.get_bits::<{ CmdType::BITS }>(0), 1);
    assert_eq!(y.get_bits::<{ bool::BITS }>(2), true);
    assert_eq!(y.get_bits::<6>(3), 35);
    let _x = MIGCmd {
        cmd: CmdType::Write,
        active: false,
        len: 30.into(),
    };
}

#[derive(Clone, Debug, Default, Copy, PartialEq, LogicStruct)]
struct CoreConfig {
    pub foo: Bits<6>,
    pub bar: Bits<32>,
    pub baz: Bits<16>,
}

#[derive(LogicBlock)]
struct TestBlock {
    pub f: Signal<Out, Bits<6>>,
    pub g: Signal<Out, Bits<32>>,
    pub h: Signal<Out, Bits<16>>,
    vals: Constant<CoreConfig>,
}

impl Logic for TestBlock {
    #[hdl_gen]
    fn update(&mut self) {
        self.f.next = self.vals.val().foo;
        self.g.next = self.vals.val().bar;
        self.h.next = self.vals.val().baz;
    }
}

impl Default for TestBlock {
    fn default() -> Self {
        Self {
            f: Default::default(),
            g: Default::default(),
            h: Default::default(),
            vals: Constant::new(CoreConfig {
                foo: 7.into(),
                bar: 32.into(),
                baz: 8.into(),
            }),
        }
    }
}

#[test]
fn test_test_block_synthesizes() {
    let mut uut = TestBlock::default();
    uut.connect_all();
    yosys_validate("test_block", &generate_verilog(&uut)).unwrap();
}
