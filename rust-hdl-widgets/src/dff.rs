use rust_hdl_core::prelude::*;

/// D Flip-Flop
///
/// Used to store data for multiple clock cycles. On every rising edge of [`clock`](Self::clock) the data from [`d`](Self::d) is transfered to [`q`](Self::q).
/// The generic parameter can be used to specify the type of data.
///
/// ### Examples
///
/// In this example DFF is used to store state for an internal counter which counts to 100.
///
/// ```
/// # use rust_hdl_lib_core::prelude::*;
/// # use rust_hdl_lib_widgets::prelude::*;
///
/// #[derive(LogicBlock)]
/// struct Counter {
///     pub clock: Signal<In, Clock>,
///     counter: DFF<Bits<7>>,
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
/// ### Additional info
///
/// It is a good idea to connect `q` to `d` to ensure that d is never undriven. The [`dff_setup`] macro can generate that code for you.
#[derive(Clone, Debug, LogicBlock)]
pub struct DFF<T: Synth> {
    /// Input for data that will be stored on the next rising edge of `clock`.
    pub d: Signal<In, T>,
    /// Outputs the currently stored data.
    pub q: Signal<Out, T>,
    /// On every rising edge the data from `d` is stored into the flip-flop. `q` outputs the currently stored data.
    pub clock: Signal<In, Clock>,
}

impl<T: Synth> Default for DFF<T> {
    fn default() -> DFF<T> {
        Self {
            d: Signal::default(),
            q: Signal::default(),
            clock: Signal::default(),
        }
    }
}

impl<T: Synth> Logic for DFF<T> {
    fn update(&mut self) {
        if self.clock.pos_edge() {
            self.q.next = self.d.val()
        }
    }
    fn connect(&mut self) {
        self.q.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Custom(format!(
            "\
initial begin
   q = {:x};
end

always @(posedge clock) begin
   q <= d;
end
      ",
            T::default().verilog()
        ))
    }
    fn timing(&self) -> Vec<TimingInfo> {
        vec![TimingInfo {
            name: "dff".into(),
            clock: "clock".into(),
            inputs: vec!["d".into()],
            outputs: vec!["q".into()],
        }]
    }
}

/// Generate boilerplate connections for one or more [`DFF`]s
///
/// You probably want to connect the input and output of every [`DFF`] together, to ensure that the input is never undriven.
/// Usually you also want to connect a clock to all [`DFF`]s. This macro can generate that boilerplate for you.
///
/// ### Example
///
/// Connect the clock `your_clock` to `your_flip_flop`
///
/// ```
/// # use rust_hdl_lib_core::prelude::*;
/// # use rust_hdl_lib_widgets::prelude::*;
///
/// # #[derive(LogicBlock)]
/// # struct Demo {
/// #     pub your_clock: Signal<In, Clock>,
/// #     your_flip_flop: DFF<Bits<7>>,
/// # }
///  
/// # impl Logic for Demo {
/// #     #[hdl_gen]
/// #     fn update(&mut self) {
///         dff_setup!(self, your_clock, your_flip_flop);
/// #     }
/// # }
/// ```
///
/// Using `dff_setup` you can write
///
/// ```
/// # use rust_hdl_lib_core::prelude::*;
/// # use rust_hdl_lib_widgets::prelude::*;
///
/// #[derive(LogicBlock)]
/// struct Demo {
///     pub clock: Signal<In, Clock>,
///     first_flip_flop: DFF<Bits<7>>,
///     second_flip_flop: DFF<Bits<32>>,
///     third_flip_flop: DFF<Bits<5>>,
/// }
///
/// impl Logic for Demo {
///     #[hdl_gen]
///     fn update(&mut self) {
///         dff_setup!(self, clock, first_flip_flop, second_flip_flop, third_flip_flop);
///
///         // Expands to:
///         // self.first_flip_flop.clock.next = self.clock.val();
///         // self.first_flip_flop.d.next = self.first_flip_flop.q.val();
///         // self.second_flip_flop.clock.next = self.clock.val();
///         // self.second_flip_flop.d.next = self.second_flip_flop.q.val();
///         // self.third_flip_flop.clock.next = self.clock.val();
///         // self.third_flip_flop.d.next = self.third_flip_flop.q.val();
///     }
/// }
/// ```
#[macro_export]
macro_rules! dff_setup {
    ($self: ident, $clock: ident, $($dff: ident),+) => {
        $($self.$dff.clock.next = $self.$clock.val());+;
        $($self.$dff.d.next = $self.$dff.q.val());+;
    }
}
