use crate::docs::vcd2svg::renderable::Renderable;
use num_bigint::BigInt;
use std::fmt::Debug;

pub trait SignalType: PartialEq + Clone + Debug + Default + Renderable {}

impl SignalType for bool {}
impl SignalType for BigInt {}
impl SignalType for String {}

#[derive(Clone, PartialEq, Debug)]
pub struct TimedValue<T: SignalType> {
    pub time: u64,
    pub value: T,
}

pub fn changes<T: SignalType>(vals: &[TimedValue<T>]) -> Vec<TimedValue<T>> {
    if vals.is_empty() {
        vec![]
    } else {
        let mut prev = vals[0].clone();
        let mut ret = vec![prev.clone()];
        for val in vals {
            if val.value.ne(&prev.value) {
                ret.push(val.clone());
                prev = val.clone();
            }
        }
        ret
    }
}
