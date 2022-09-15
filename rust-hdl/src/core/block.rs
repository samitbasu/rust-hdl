use crate::core::logic::Logic;
use crate::core::probe::Probe;

/// The [Block] trait is required for all circuitry that
/// can be simulated by RustHDL.  If you want to be able
/// to simulate a circuit, the corresponding struct must
/// impl [Block].  Normally, this is done via the `#[derive(LogicBlock)]`
/// construct, and you will rarely, if ever, need to 
/// impl the [Block] trait yourself.
pub trait Block: Logic {
    /// Connects the internal signals of the circuit - used to initialize the circuit
    fn connect_all(&mut self);
    /// Propogate changes from inputs to outputs within the circuit
    fn update_all(&mut self);
    /// Returns `true` if anything in the circuit has changed (outputs or internal state)
    fn has_changed(&self) -> bool;
    /// The visitor pattern - allows a circuit to be probed by a [Probe] struct.
    fn accept(&self, name: &str, probe: &mut dyn Probe);
}

impl<B: Block> Block for Vec<B> {
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
