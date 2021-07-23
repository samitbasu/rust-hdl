use std::fmt::Debug;

// We don't want clock types to be multibit or other weird things...
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Clock<D: Domain>(pub bool, pub std::marker::PhantomData<D>);

pub const NANOS_PER_FEMTO : f64 = 1_000_000.0;

pub fn freq_hz_to_period_femto(freq: f64) -> f64 {
    (1.0e15 / freq).round()
}

#[test]
fn test_freq_to_period_mapping() {
    assert_eq!(freq_hz_to_period_femto(1.0), 1.0e15)
}

impl<D: Domain> std::ops::Not for Clock<D> {
    type Output = Clock<D>;

    fn not(self) -> Self::Output {
        Clock(!self.0, self.1)
    }
}

pub trait Domain : Debug + PartialEq + Copy + Default + Clone {
    const FREQ: u64;
}

impl<D: Domain> Domain for Clock<D> {
    const FREQ: u64 = D::FREQ;
}

#[macro_export]
macro_rules! make_domain {
    ($name: ident, $freq: expr) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name {}

        impl Domain for $name {
            const FREQ: u64 = $freq;
        }
    }
}