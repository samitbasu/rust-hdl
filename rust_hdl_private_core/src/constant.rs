use crate::ast::VerilogLiteral;
use crate::atom::{Atom, AtomKind};
use crate::bits::Bits;
use crate::block::Block;
use crate::constraint::PinConstraint;
use crate::logic::Logic;
use crate::probe::Probe;
use crate::signal::{get_signal_id, Signal};
use crate::sim_assert_eq;
use crate::simulate::{Sim, Simulation};
use crate::synth::{Synth, VCDValue};
use crate::type_descriptor::TypeDescriptor;

/// The [Constant] wrapper can hold any [Synth] type
/// and store it in a circuit for use by the HDL kernel.
/// This is the easiest way to compute complex constants
/// in your RustHDL constructors, and then store the
/// results inside the circuit for later use.  Unlike
/// [Signal], [Constant] does not have a `.next` field,
/// so you cannot assign to a [Constant] in the HDL
/// kernel (blocked at compile time).  Note that [Constant]
/// does not `impl Default`.  You must construct it
/// with the appropriate value when the circuit is built.
///
/// Here is a correct usage of a [Constant]
/// ```rust
///  use rust_hdl_private_core::prelude::*;
///
/// #[derive(LogicBlock)]
/// struct AddNum {
///    pub i1: Signal<In, Bits<8>>,
///    pub o1: Signal<Out, Bits<8>>,
///    c1: Constant<Bits<8>>,
/// }
///
/// impl Default for AddNum {
///     fn default() -> Self {
///          Self {
///             i1: Default::default(),
///             o1: Default::default(),
///             c1: Constant::new(42.into()),
///          }
///     }
/// }
///
/// impl Logic for AddNum {
///      #[hdl_gen]
///      fn update(&mut self) {
///         // Note that `self.c1.next` does not exist...
///         self.o1.next = self.i1.val() + self.c1.val();
///      }
/// }
///
///    let mut sim : Simulation<AddNum> = Simulation::default();
///    sim.add_testbench(|mut ep: Sim<AddNum>| {
///        let mut x = ep.init()?;
///        x.i1.next = 13.into();
///        x = ep.wait(1, x)?;
///        sim_assert_eq!(ep, x.o1.val(), 55, x);
///        ep.done(x)
///    });
///    
///    let mut uut = AddNum::default(); uut.connect_all();
///    sim.run(Box::new(uut), 100).unwrap();
///```
#[derive(Copy, Clone, Debug)]
pub struct Constant<T: Synth> {
    val: T,
    id: usize,
}

impl<T: Synth> Constant<T> {
    /// Create a new [Constant] from the given value.
    pub fn new(val: T) -> Constant<T> {
        Constant {
            val,
            id: get_signal_id(),
        }
    }
    /// Retrieve the value of the constant.  Usable in HDL kernels.
    pub fn val(&self) -> T {
        self.val
    }
}

impl<T: Synth> Logic for Constant<T> {
    fn update(&mut self) {}

    fn connect(&mut self) {}
}

impl<T: Synth> Atom for Constant<T> {
    fn bits(&self) -> usize {
        T::BITS
    }

    fn connected(&self) -> bool {
        true
    }

    fn changed(&self) -> bool {
        false
    }

    fn kind(&self) -> AtomKind {
        AtomKind::Constant
    }

    fn descriptor(&self) -> TypeDescriptor {
        T::descriptor()
    }

    fn vcd(&self) -> VCDValue {
        self.val.vcd()
    }

    fn verilog(&self) -> VerilogLiteral {
        self.val.verilog()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn constraints(&self) -> Vec<PinConstraint> {
        vec![]
    }
}

impl<T: Synth> Block for Constant<T> {
    fn connect_all(&mut self) {}

    fn update_all(&mut self) {}

    fn has_changed(&self) -> bool {
        false
    }

    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        probe.visit_atom(name, self);
    }
}
