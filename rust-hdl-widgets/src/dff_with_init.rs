use crate::{dff::DFF, dff_setup};
use rust_hdl_core::prelude::*;
use std::ops::BitXor;

/// D Flip-Flop with an initial value
///
/// This is identical to [`DFF`], except that it has an initial value.
///
/// Used to store data for multiple clock cycles. On every rising edge of [`clock`](Self::clock) the data from [`d`](Self::d) is transfered to [`q`](Self::q).
/// The generic parameter can be used to specify the type of data.
///
/// ### Examples
///
/// Create a `DFFWithInit` with an initial value of 42
///
/// ```
/// # use rust_hdl_lib_core::prelude::*;
/// # use rust_hdl_lib_widgets::prelude::*;
///
/// let flip_flop: DFFWithInit<Bits<7>> = DFFWithInit::new(42.into());
/// ```
///
/// Use `DFFWithInit` to store state for a counter which counts to 100 and has an initial value of 50.
///
/// ```
/// # use rust_hdl_lib_core::prelude::*;
/// # use rust_hdl_lib_widgets::prelude::*;
///
/// #[derive(LogicBlock)]
/// struct Counter {
///     pub clock: Signal<In, Clock>,
///     counter: DFFWithInit<Bits<7>>,
/// }
///
/// impl Default for Counter {
///     fn default() -> Self {
///         Self {
///             clock: Default::default(),
///             counter: DFFWithInit::new(50u64.into()),
///         }
///     }
/// }
///
/// impl Logic for Counter {
///     #[hdl_gen]
///     fn update(&mut self) {
///         self.counter.clock.next = self.clock.val();
///         self.counter.d.next = self.counter.q.val();
///         // You can use `dff_setup!(self, clock, clock_counter);` to generate the above code.
///         
///         self.counter.d.next = self.counter.q.val() + 1u64.to_bits();
///
///         // Reset the counter, if the value is greater than 100
///         if self.counter.q.val() >= 100u64.to_bits() {
///             self.counter.d.next = 0.into();
///         }
///     }
/// }
/// ```
///
/// ### Inputs
///
/// * [`clock`](Self::clock) On every rising edge the data from [`d`](Self::d) is stored into the flip-flop.
/// * [`d`](Self::d) Input for data that will be stored on the next rising edge of [`clock`](Self::clock).
///
/// ### Outputs
///  
/// * [`q`](Self::q) Outputs the currently stored data.
///
/// ### Constants
///
/// * [`init`](Self::init) The initial value
///
/// ### Additional info
///
/// It is a good idea to connect [`q`](Self::q) to [`d`](Self::d) to ensure that d is never undriven. The [`dff_setup`] macro can generate that code for you.
#[derive(Clone, Debug, LogicBlock)]
pub struct DFFWithInit<T: Synth + BitXor<Output = T>> {
    /// Input for data that will be stored on the next rising edge of `clock`.
    pub d: Signal<In, T>,
    /// Outputs the currently stored data.
    pub q: Signal<Out, T>,
    /// On every rising edge the data from `d` is stored into the flip-flop. `q` outputs the currently stored data.
    pub clock: Signal<In, Clock>,
    /// The default value
    pub init: Constant<T>,
    pub dff: DFF<T>,
}

impl<T: Synth + BitXor<Output = T>> DFFWithInit<T> {
    pub fn new(init: T) -> Self {
        Self {
            d: Default::default(),
            q: Default::default(),
            clock: Default::default(),
            init: Constant::new(init),
            dff: Default::default(),
        }
    }
}

impl<T: Synth + BitXor<Output = T>> Logic for DFFWithInit<T> {
    #[hdl_gen]
    fn update(&mut self) {
        self.dff.clock.next = self.clock.val();
        self.q.next = self.dff.q.val() ^ self.init.val();
        self.dff.d.next = self.d.val() ^ self.init.val();
    }
}
