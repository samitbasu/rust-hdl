use crate::logic::Logic;
use crate::probe::Probe;

pub trait Block: Logic {
    fn connect_all(&mut self);
    fn update_all(&mut self);
    fn has_changed(&self) -> bool;
    fn accept(&self, name: &str, probe: &mut dyn Probe);
}

impl<B: Block, const P: usize> Block for [B; P] {
    fn connect_all(&mut self) {
        for x in self {
            x.connect_all();
        }
    }

    fn update_all(&mut self) {
        for x in self {
            x.update_all();
        }
    }

    fn has_changed(&self) -> bool {
        for x in self {
            if x.has_changed() {
                return true;
            }
        }
        false
    }

    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        for x in self.iter().enumerate() {
            let name = format!("{}${}", name, x.0);
            x.1.accept(&name, probe);
        }
    }
}
