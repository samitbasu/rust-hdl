use crate::core::prelude::*;

// We want to be able to combine a set of signals into a struct

#[derive(Clone, Copy, Debug, PartialEq, Eq, LogicState)]
enum CmdType {
    Noop,
    Read,
    Write,
}


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
        len: 35_usize.into(),
    };

    let y: Bits<9> = x.into();
    assert_eq!(y.get_bits::<{ CmdType::BITS }>(0), 1u32);
    assert_eq!(y.get_bits::<{ bool::BITS }>(2), true);
    assert_eq!(y.get_bits::<6>(3), 35_u32);
    let _x = MIGCmd {
        cmd: CmdType::Write,
        active: false,
        len: 30_usize.into(),
    };
}
