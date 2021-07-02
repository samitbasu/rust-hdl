use crate::logic::Logic;
use crate::probe::Probe;

pub trait Block : Logic {
    fn connect_all(&mut self);
    fn update_all(&mut self);
    fn has_changed(&self) -> bool;
    fn accept(&self, name: &str, probe: &mut dyn Probe);
}
