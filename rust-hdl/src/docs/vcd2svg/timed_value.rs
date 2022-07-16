#[derive(Clone, PartialEq)]
pub struct TimedValue<T: PartialEq + Clone> {
    pub time: u64,
    pub value: T,
}

pub fn changes<T: PartialEq + Clone>(vals: &[TimedValue<T>]) -> Vec<TimedValue<T>> {
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
