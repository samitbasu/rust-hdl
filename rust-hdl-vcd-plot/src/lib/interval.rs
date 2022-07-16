#[derive(Clone, PartialEq)]
pub struct Interval<T: PartialEq + Clone> {
    pub start_time: u64,
    pub end_time: u64,
    pub value: T,
    pub start_x: f64,
    pub end_x: f64,
    pub label: String,
}

impl<T: PartialEq + Clone> Interval<T> {
    pub fn is_empty(&self) -> bool {
        self.end_x == self.start_x
    }
}
