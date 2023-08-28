use std::{default, fmt::Binary};

use crate::kind::SynthKind;

pub trait Synth: Copy + PartialEq {
    fn static_kind() -> SynthKind;
    fn kind(self) -> SynthKind {
        Self::static_kind()
    }
    fn bin(self) -> Vec<bool>;
}

impl<S: Synth, const N: usize> Synth for [S; N] {
    fn static_kind() -> SynthKind {
        SynthKind::Array {
            base: Box::new(S::static_kind()),
            size: N,
        }
    }
    fn bin(self) -> Vec<bool> {
        self.iter().flat_map(|x| x.bin()).collect()
    }
}

impl Synth for u8 {
    fn static_kind() -> SynthKind {
        SynthKind::Bits {
            digits: vec![false; 8],
        }
    }
    fn bin(self) -> Vec<bool> {
        (0..8).map(|x| self & (1 << x) != 0).collect()
    }
}

impl Synth for u16 {
    fn static_kind() -> SynthKind {
        SynthKind::Bits {
            digits: vec![false; 16],
        }
    }
    fn bin(self) -> Vec<bool> {
        (0..16).map(|x| self & (1 << x) != 0).collect()
    }
}

impl Synth for u32 {
    fn static_kind() -> SynthKind {
        SynthKind::Bits {
            digits: vec![false; 32],
        }
    }
    fn bin(self) -> Vec<bool> {
        (0..32).map(|x| self & (1 << x) != 0).collect()
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
enum State {
    #[default]
    A,
    B,
    C,
}

impl Synth for State {
    fn static_kind() -> SynthKind {
        SynthKind::Enum {
            variants: vec![
                ("A".to_string(), SynthKind::Empty),
                ("B".to_string(), SynthKind::Empty),
                ("C".to_string(), SynthKind::Empty),
            ],
        }
    }
    fn bin(self) -> Vec<bool> {
        match self {
            State::A => vec![false, false],
            State::B => vec![true, false],
            State::C => vec![false, true],
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
struct MyTuple(u8, u16);

impl Synth for MyTuple {
    fn static_kind() -> SynthKind {
        SynthKind::Tuple {
            elements: vec![<u8 as Synth>::static_kind(), <u16 as Synth>::static_kind()],
        }
    }
    fn bin(self) -> Vec<bool> {
        self.0.bin().into_iter().chain(self.1.bin()).collect()
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
struct MyStruct {
    a: u8,
    b: u16,
    c: u32,
}

impl Synth for MyStruct {
    fn static_kind() -> SynthKind {
        SynthKind::Struct {
            fields: vec![
                ("a".to_string(), <u8 as Synth>::static_kind()),
                ("b".to_string(), <u16 as Synth>::static_kind()),
                ("c".to_string(), <u32 as Synth>::static_kind()),
            ],
        }
    }
    fn bin(self) -> Vec<bool> {
        self.a
            .bin()
            .into_iter()
            .chain(self.b.bin())
            .chain(self.c.bin())
            .collect()
    }
}

impl<A: Synth, B: Synth> Synth for (A, B) {
    fn static_kind() -> SynthKind {
        SynthKind::Tuple {
            elements: vec![A::static_kind(), B::static_kind()],
        }
    }
    fn bin(self) -> Vec<bool> {
        self.0.bin().into_iter().chain(self.1.bin()).collect()
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
struct MyComplexStruct {
    a: u8,
    b: u16,
    c: u32,
    d: MyStruct,
    e: (u16, MyStruct),
    f: u8,
}

impl Synth for MyComplexStruct {
    fn static_kind() -> SynthKind {
        SynthKind::Struct {
            fields: vec![
                ("a".to_string(), <u8 as Synth>::static_kind()),
                ("b".to_string(), <u16 as Synth>::static_kind()),
                ("c".to_string(), <u32 as Synth>::static_kind()),
                ("d".to_string(), <MyStruct as Synth>::static_kind()),
                ("e".to_string(), <(u16, MyStruct) as Synth>::static_kind()),
                ("f".to_string(), <u8 as Synth>::static_kind()),
            ],
        }
    }
    fn bin(self) -> Vec<bool> {
        self.a
            .bin()
            .into_iter()
            .chain(self.b.bin())
            .chain(self.c.bin())
            .chain(self.d.bin())
            .chain(self.e.bin())
            .chain(self.f.bin())
            .collect()
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
struct EnumWithU8 {
    a: u8,
    b: State,
}

impl Synth for EnumWithU8 {
    fn static_kind() -> SynthKind {
        SynthKind::Struct {
            fields: vec![
                ("a".to_string(), <u8 as Synth>::static_kind()),
                ("b".to_string(), <State as Synth>::static_kind()),
            ],
        }
    }
    fn bin(self) -> Vec<bool> {
        self.a.bin().into_iter().chain(self.b.bin()).collect()
    }
}

// You can also match on a value
// which means that u8 should have impl SynthEnum on it
// with DISCRIMINANT_BITS = 8, and all static_payload_size --> 0

#[derive(Copy, Clone, PartialEq)]
enum ADT {
    A(u8),
    B(u16),
    C(u32),
}

impl Synth for ADT {
    fn static_kind() -> SynthKind {
        SynthKind::Enum {
            variants: vec![
                ("A".to_string(), <u8 as Synth>::static_kind()),
                ("B".to_string(), <u16 as Synth>::static_kind()),
                ("C".to_string(), <u32 as Synth>::static_kind()),
            ],
        }
    }
    fn bin(self) -> Vec<bool> {
        match self {
            ADT::A(x) => [false, false].into_iter().chain(x.bin()).collect(),
            ADT::B(x) => [true, false].into_iter().chain(x.bin()).collect(),
            ADT::C(x) => [true, true].into_iter().chain(x.bin()).collect(),
        }
    }
}

pub fn binary(x: impl Synth) -> String {
    x.bin()
        .into_iter()
        .map(|x| if x { '1' } else { '0' })
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_my_struct_properties() {
        assert_eq!(MyStruct::static_kind().bits(), 57);
    }

    #[test]
    fn test_my_tuple_properties() {
        let tmp = (0_u8, 7_u16);
        println!("{}", binary(tmp));
    }

    /*
        #[test]
        fn test_my_complex_struct_properties() {
            let foo: MyComplexStruct = Default::default();
            assert_eq!(MyComplexStruct::static_offset(stringify!(a)), 0);
            assert_eq!(MyComplexStruct::static_offset(stringify!(b)), foo.a.bits());
            assert_eq!(
                MyComplexStruct::static_offset(stringify!(c)),
                foo.a.bits() + foo.b.bits()
            );
            assert_eq!(
                MyComplexStruct::static_offset(stringify!(d)),
                foo.a.bits() + foo.b.bits() + foo.c.bits()
            );
            assert_eq!(
                MyComplexStruct::static_offset(stringify!(e)),
                foo.a.bits() + foo.b.bits() + foo.c.bits() + foo.d.bits()
            );
            assert_eq!(
                MyComplexStruct::static_offset(stringify!(f)),
                foo.a.bits() + foo.b.bits() + foo.c.bits() + foo.d.bits() + foo.e.bits()
            );
        }

        #[test]
        fn test_nested_indexing() {
            let mut k = MyComplexStruct::default();
            k.e.1.b = 0xDEAD;
            println!("{}", k.bin());
            let dummy_ptr = (&k) as *const _;
            let member_ptr = (&k.e.1.b) as *const _;
            let offset = member_ptr as usize - dummy_ptr as usize;
            println!("Offset: {}", offset);
            let base = k.offset("e") + k.e.offset(1) + k.e.1.offset("b");
            let j = k.bin().chars().skip(base).take(16).collect::<String>();
            println!("j: {}", j);
            // In the proc macro, we can convert an expression like `k.e.1.b` into something like:
            // Slice(k, k.field_offset("e") + k.e.index_offset(1) + k.e.1.field_offset("b"), k.e.1.b.bits())
            // On the left hand side, something like:
            // k.e.1.b = 0xDEAD
            // Should translate into
            // Assign(Slice(k, k.field_offset("e") + k.e.index_offset(1) + k.e.1.field_offset("b"), k.e.1.b.bits()), 0xDEAD)
            // If we have a local like:
            // let foo = <expr>;
            // Then we can allocate a global, and use "<expr>::BITS" as the size of the global.
            // We can avoid side effects by constructing a method for finding the type of an expression.
            // This won't actually work because there is no variable "k" in play at the time this expression executes.
        }

        #[test]
        fn test_array_case() {
            let mut k = [0_u8; 4];
            println!("bits {}", k.bits());
            k[1] = 0xFF;
            println!("{}", k.bin());
        }

        #[test]
        fn test_side_effects() {
            let c = false;
            let foo = {
                println!("Hello");
                if c {
                    "blo"
                } else {
                    "bar"
                }
            };
            println!("Foo: {}", foo);
        }
    */
    #[test]
    fn test_enum_withu8() {
        let k = EnumWithU8 { b: State::B, a: 0 };
        println!("{}", binary(k));
    }
}
