// We don't want clock types to be multibit or other weird things...
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Clock(pub bool);

impl std::ops::Not for Clock {
    type Output = Clock;

    fn not(self) -> Self::Output {
        Clock(!self.0)
    }
}
