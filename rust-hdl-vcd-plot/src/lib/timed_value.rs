#[derive(Clone, PartialEq)]
pub struct TimedValue<T: PartialEq + Clone> {
    pub time: u64,
    pub value: T,
}
