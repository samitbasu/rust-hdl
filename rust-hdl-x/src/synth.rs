pub trait Synth: Copy + PartialEq {
    const BITS: usize;
    fn bits(self) -> usize {
        Self::BITS
    }
    fn get(&self, index: usize) -> bool;
    fn set(&mut self, index: usize, val: bool);
    fn pack(self) -> Vec<bool> {
        self.iter().collect()
    }
    fn unpack(&mut self, v: &[bool]) {
        assert_eq!(v.len(), Self::BITS);
        v.iter().enumerate().take(Self::BITS).for_each(|(i, b)| {
            self.set(i, *b);
        });
    }
    fn iter(&self) -> BitIter<Self> {
        BitIter {
            synth: self,
            index: 0,
        }
    }
    fn bin(&self) -> String {
        self.iter()
            .map(|b| if b { '1' } else { '0' })
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    }
}

pub struct BitIter<'a, T: Synth> {
    synth: &'a T,
    index: usize,
}

impl<'a, T: Synth> Iterator for BitIter<'a, T> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < T::BITS {
            let val = self.synth.get(self.index);
            self.index += 1;
            Some(val)
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BitSpec {
    pub offset: usize,
    pub width: usize,
}

pub trait SynthStruct: Synth {
    fn static_offset(name: &'static str) -> usize;
    fn offset(self, name: &'static str) -> usize {
        Self::static_offset(name)
    }
}

pub trait SynthTuple: Synth {
    fn static_offset(index: usize) -> usize;
    fn offset(self, index: usize) -> usize {
        Self::static_offset(index)
    }
}

pub trait SynthArray: Synth {
    fn static_offset(index: usize) -> usize;
    fn offset(self, index: usize) -> usize {
        Self::static_offset(index)
    }
}

impl<S: Synth, const N: usize> Synth for [S; N] {
    const BITS: usize = N * S::BITS;
    fn get(&self, index: usize) -> bool {
        assert!(index < Self::BITS);
        let array_index = index / S::BITS;
        let synth_index = index % S::BITS;
        self[array_index].get(synth_index)
    }
    fn set(&mut self, index: usize, val: bool) {
        assert!(index < Self::BITS);
        let array_index = index / S::BITS;
        let synth_index = index % S::BITS;
        self[array_index].set(synth_index, val)
    }
}

impl<S: Synth, const N: usize> SynthArray for [S; N] {
    fn static_offset(index: usize) -> usize {
        index * S::BITS
    }
}

impl Synth for u8 {
    const BITS: usize = 8;
    fn get(&self, index: usize) -> bool {
        assert!(index < Self::BITS as usize);
        *self & (1 << index) != 0
    }
    fn set(&mut self, index: usize, val: bool) {
        assert!(index < Self::BITS as usize);
        if val {
            *self |= 1 << index;
        } else {
            *self &= !(1 << index);
        }
    }
}

impl Synth for u16 {
    const BITS: usize = 16;
    fn get(&self, index: usize) -> bool {
        assert!(index < Self::BITS as usize);
        *self & (1 << index) != 0
    }
    fn set(&mut self, index: usize, val: bool) {
        assert!(index < Self::BITS as usize);
        if val {
            *self |= 1 << index;
        } else {
            *self &= !(1 << index);
        }
    }
}

impl Synth for u32 {
    const BITS: usize = 32;
    fn get(&self, index: usize) -> bool {
        assert!(index < Self::BITS as usize);
        *self & (1 << index) != 0
    }
    fn set(&mut self, index: usize, val: bool) {
        assert!(index < Self::BITS as usize);
        if val {
            *self |= 1 << index;
        } else {
            *self &= !(1 << index);
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
struct MyStruct {
    a: u8,
    b: u16,
    c: u32,
}

impl Synth for MyStruct {
    const BITS: usize = <u8 as Synth>::BITS + <u16 as Synth>::BITS + <u32 as Synth>::BITS;
    fn get(&self, index: usize) -> bool {
        assert!(index < Self::BITS);
        if index < <u8 as Synth>::BITS {
            self.a.get(index)
        } else if index < <u8 as Synth>::BITS + <u16 as Synth>::BITS {
            self.b.get(index - <u8 as Synth>::BITS)
        } else {
            self.c
                .get(index - <u8 as Synth>::BITS - <u16 as Synth>::BITS)
        }
    }
    fn set(&mut self, index: usize, val: bool) {
        assert!(index < Self::BITS);
        if index < <u8 as Synth>::BITS {
            self.a.set(index, val)
        } else if index < <u8 as Synth>::BITS + <u16 as Synth>::BITS {
            self.b.set(index - <u8 as Synth>::BITS, val)
        } else {
            self.c
                .set(index - <u8 as Synth>::BITS - <u16 as Synth>::BITS, val)
        }
    }
}

impl SynthStruct for MyStruct {
    fn static_offset(name: &'static str) -> usize {
        match name {
            "a" => 0,
            "b" => <u8 as Synth>::BITS,
            "c" => <u8 as Synth>::BITS + <u16 as Synth>::BITS,
            _ => panic!("Unknown field name"),
        }
    }
}

impl<A: Synth, B: Synth> Synth for (A, B) {
    const BITS: usize = A::BITS + B::BITS;
    fn get(&self, index: usize) -> bool {
        assert!(index < Self::BITS);
        if index < A::BITS {
            self.0.get(index)
        } else {
            self.1.get(index - A::BITS)
        }
    }
    fn set(&mut self, index: usize, val: bool) {
        assert!(index < Self::BITS);
        if index < A::BITS {
            self.0.set(index, val)
        } else {
            self.1.set(index - A::BITS, val)
        }
    }
}

impl<A: Synth, B: Synth> SynthTuple for (A, B) {
    fn static_offset(index: usize) -> usize {
        match index {
            0 => 0,
            1 => A::BITS,
            _ => panic!("Unknown index"),
        }
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
    const BITS: usize = <u8 as Synth>::BITS
        + <u16 as Synth>::BITS
        + <u32 as Synth>::BITS
        + <MyStruct as Synth>::BITS
        + <(u16, MyStruct) as Synth>::BITS
        + <u8 as Synth>::BITS;
    fn get(&self, index: usize) -> bool {
        assert!(index < Self::BITS);
        if index < <u8 as Synth>::BITS {
            self.a.get(index)
        } else if index < <u8 as Synth>::BITS + <u16 as Synth>::BITS {
            self.b.get(index - <u8 as Synth>::BITS)
        } else if index < <u8 as Synth>::BITS + <u16 as Synth>::BITS + <u32 as Synth>::BITS {
            self.c
                .get(index - <u8 as Synth>::BITS - <u16 as Synth>::BITS)
        } else if index
            < <u8 as Synth>::BITS
                + <u16 as Synth>::BITS
                + <u32 as Synth>::BITS
                + <MyStruct as Synth>::BITS
        {
            self.d
                .get(index - <u8 as Synth>::BITS - <u16 as Synth>::BITS - <u32 as Synth>::BITS)
        } else if index
            < <u8 as Synth>::BITS
                + <u16 as Synth>::BITS
                + <u32 as Synth>::BITS
                + <MyStruct as Synth>::BITS
                + <(u16, MyStruct) as Synth>::BITS
        {
            self.e.get(
                index
                    - <u8 as Synth>::BITS
                    - <u16 as Synth>::BITS
                    - <u32 as Synth>::BITS
                    - <MyStruct as Synth>::BITS,
            )
        } else {
            self.f.get(
                index
                    - <u8 as Synth>::BITS
                    - <u16 as Synth>::BITS
                    - <u32 as Synth>::BITS
                    - <MyStruct as Synth>::BITS
                    - <(u16, MyStruct) as Synth>::BITS,
            )
        }
    }
    fn set(&mut self, index: usize, val: bool) {
        assert!(index < Self::BITS);
        if index < <u8 as Synth>::BITS {
            self.a.set(index, val)
        } else if index < <u8 as Synth>::BITS + <u16 as Synth>::BITS {
            self.b.set(index - <u8 as Synth>::BITS, val)
        } else if index < <u8 as Synth>::BITS + <u16 as Synth>::BITS + <u32 as Synth>::BITS {
            self.c
                .set(index - <u8 as Synth>::BITS - <u16 as Synth>::BITS, val)
        } else if index
            < <u8 as Synth>::BITS
                + <u16 as Synth>::BITS
                + <u32 as Synth>::BITS
                + <MyStruct as Synth>::BITS
        {
            self.d.set(
                index - <u8 as Synth>::BITS - <u16 as Synth>::BITS - <u32 as Synth>::BITS,
                val,
            )
        } else if index
            < <u8 as Synth>::BITS
                + <u16 as Synth>::BITS
                + <u32 as Synth>::BITS
                + <MyStruct as Synth>::BITS
                + <(u16, MyStruct) as Synth>::BITS
        {
            self.e.set(
                index
                    - <u8 as Synth>::BITS
                    - <u16 as Synth>::BITS
                    - <u32 as Synth>::BITS
                    - <MyStruct as Synth>::BITS,
                val,
            )
        } else {
            self.f.set(
                index
                    - <u8 as Synth>::BITS
                    - <u16 as Synth>::BITS
                    - <u32 as Synth>::BITS
                    - <MyStruct as Synth>::BITS
                    - <(u16, MyStruct) as Synth>::BITS,
                val,
            )
        }
    }
}

impl SynthStruct for MyComplexStruct {
    fn static_offset(name: &'static str) -> usize {
        match name {
            "a" => 0,
            "b" => <u8 as Synth>::BITS,
            "c" => <u8 as Synth>::BITS + <u16 as Synth>::BITS,
            "d" => <u8 as Synth>::BITS + <u16 as Synth>::BITS + <u32 as Synth>::BITS,
            "e" => {
                <u8 as Synth>::BITS
                    + <u16 as Synth>::BITS
                    + <u32 as Synth>::BITS
                    + <MyStruct as Synth>::BITS
            }
            "f" => {
                <u8 as Synth>::BITS
                    + <u16 as Synth>::BITS
                    + <u32 as Synth>::BITS
                    + <MyStruct as Synth>::BITS
                    + <(u16, MyStruct) as Synth>::BITS
            }
            _ => panic!("Unknown field name"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_my_struct_properties() {
        assert_eq!(MyStruct::static_offset(stringify!(a)), 0);
        assert_eq!(MyStruct::static_offset(stringify!(b)), 8);
        assert_eq!(MyStruct::static_offset(stringify!(c)), 24);
    }

    #[test]
    fn test_my_tuple_properties() {
        let tmp = (0_u8, 0_u16);
        assert_eq!(tmp.offset(0), 0);
        assert_eq!(tmp.offset(1), 8);
    }

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
    }

    #[test]
    fn test_array_case() {
        let mut k = [0_u8; 4];
        println!("bits {}", k.bits());
        k[1] = 0xFF;
        println!("{}", k.bin());
    }
}
