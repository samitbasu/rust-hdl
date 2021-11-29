use crate::core::bits::Bits;
use num_bigint::BigUint;
use std::fmt::{Display, Formatter, LowerHex};
use crate::core::yosys::SynthError;

/// The BlackBox struct provides a way to wrap a blackbox,
/// externally provided IP core.
///
/// You will frequently in the world of FPGA firmware need to
/// wrap an external IP core that provides some functionality
/// you cannot implement yourself.  For example, an analog
/// clock driver circuit that takes a double ended clock and
/// converts it to a single ended clock.  Your synthesis tools
/// may be smart enough to infer such a device, but most likely
/// they will need help.  That is where a black box references
/// come in.  Think of it as a link reference
/// to an external library.  Assuming you are going to validate
/// the code using `yosys`, you will need to satisfy a few additional
/// constraints.
///
/// 1.  You must provide a black box declaration for any module that
/// is external to your RustHDL firmware, or `yosys` will fail to validate
/// the resulting firmware.
/// 2.  You must annotate that black box declaration in such a way that
/// `yosys` knows the module is externally defined.
/// 3.  RustHDL does not currently have the ability to dynamically rename
/// conflicting instances of black box firmware.  So you can only include
/// one instance of each black box firmware in your design.  This is generally
/// _not_ what you want, and the [Wrapper] struct provides a better way
/// to wrap a black box core (with some slight tradeoffs).
///
/// To use the [BlackBox] variant, you will need to provide a custom
/// implementation of the [hdl] function in the [Logic] trait.  This
/// is fairly straightforward to do, so lets look at some examples.
///
/// Example
/// Let's imagine we have a module that represents a custom
/// piece of hardware that takes a differential clock from
/// outside the FPGA, and converts into a single ended clock
/// to drive logic.  This is clearly going to be custom analog
/// circuitry, so it's natural that we need to wrap an external
/// black box primitive to get that functionality.
///
/// The [BlackBox] variant does not allow much flexibility.  Our
/// RustHDL struct that describes the external IP must match
/// the underlying Verilog exactly, or the included IP will not
/// work correctly.  For the clock buffer, we have a Verilog
/// definition from the manufacturer that looks like this
/// ```verilog
/// module IBUFDS(I, B, O);
///    input I;
///    input B;
///    output O;
/// endmodule
/// ```
///
/// It's succinct, but the argument names are pretty terse.
/// Unfortunately, we can't remap them or make it more ergonomic
/// with a [BlackBox].  For those types of operations, we need
/// to use a [Wrapper].  For now, we start with a struct to
/// describe the circuit.  Note that at least the name of the
/// RustHDL struct does not have to be the same as the black box
/// IP core.
///
/// ```rust
/// # use rust_hdl_core::prelude::*;
/// pub struct ClockDriver {
///    pub I: Signal<In, Clock>,
///    pub B: Signal<In, Clock>,
///    pub O: Signal<Out, Clock>,
/// }
/// ```
/// We will use the [LogicBlock] derive macro to add the [Logic]
/// trait to our circuit (so RustHDL can work with it), and the
/// [Default] trait as well, to make it easy to use.
/// The [Logic] trait for this circuit will need to be implemented
/// by hand.
/// ```rust
/// # use rust_hdl_core:: prelude::*;
/// # #[derive(LogicBlock, Default)]
/// # pub struct ClockDriver {
/// #   pub I: Signal<In, Clock>,
/// #   pub B: Signal<In, Clock>,
/// #   pub O: Signal<Out, Clock>,
/// # }
///
/// impl Logic for ClockDriver {
///     fn update(&mut self) {
///         todo!()
///     }
///
///     fn connect(&mut self) {
///         todo!()
///     }
///
///     fn hdl(&self) -> Verilog {
///         todo!()
///     }
/// }
/// ```
///
/// The [Logic] trait requires 3 methods [Logic::update], [Logic::connect],
/// and [Logic::hdl].  The [Logic::update] method is used for simulation, and
/// at the moment, black box modules are not simulatable.  So we can accept
/// the default implementation of this.  The [Logic::connect] method is used
/// to indicate which members of the circuit are _driven_ by the circuit.
/// A better name might have been `drive`, instead of `connect`, but we will
/// stick with the current terminology.  You can think of it in terms of an
/// integrated circuit - outputs, are generally driven and are internally
/// *connected*, while inputs are generally driven from outside and are externally
/// connected.  For our black box, the [Logic::connect] trait implementation
/// is very simple:
/// ```rust
/// # use rust_hdl_core:: prelude::*;
/// # #[derive(LogicBlock, Default)]
/// # pub struct ClockDriver {
/// #   pub I: Signal<In, Clock>,
/// #   pub B: Signal<In, Clock>,
/// #   pub O: Signal<Out, Clock>,
/// # }
///
/// impl Logic for ClockDriver {
///     fn update(&mut self) {
///         // No simulation model
///     }
///
///     fn connect(&mut self) {
///         self.O.connect();
///     }
///
///     fn hdl(&self) -> Verilog {
///         todo!()
///     }
/// }
/// ```
///
/// Now, we need an implementation for the HDL for this Clock driver.
/// That is where we need the [BlackBox] struct.
///
/// ```rust
/// # use rust_hdl_core:: prelude::*;
/// # #[derive(LogicBlock, Default)]
/// # pub struct ClockDriver {
/// #   pub I: Signal<In, Clock>,
/// #   pub B: Signal<In, Clock>,
/// #   pub O: Signal<Out, Clock>,
/// # }
///
/// impl Logic for ClockDriver {
///     fn update(&mut self) {
///     }
///
///     fn connect(&mut self) {
///         self.O.connect();
///     }
///
///      fn hdl(&self) -> Verilog {
///         Verilog::Blackbox(BlackBox {
///           code: r#"
/// (* blackbox *)
/// module IBUFDS(I, B, O)
///   input I;
///   input B;
///   output O;
/// endmodule"#.into(),
///           name: "IBUFDS".into()
///         })
///      }
/// }
/// ```
///
/// Just to re-emphasize the point.  Your FPGA provider will give you a
/// Verilog declaration for the IP core.  You cannot change it!  The names
/// of the signals must be the same, even if Rust complains about your
/// intransigence.
///
/// With all 3 of the methods implemented, we can now create an instance
/// of our clock driver, synthesize it, and test it.  Here is the completed
/// example:
///
/// ```rust
/// # use rust_hdl_core::prelude::*;
///
/// #[derive(LogicBlock, Default)]
/// pub struct ClockDriver {
///   pub I: Signal<In, Clock>,
///   pub B: Signal<In, Clock>,
///   pub O: Signal<Out, Clock>,
/// }
///
/// impl Logic for ClockDriver {
///     fn update(&mut self) {
///     }
///
///     fn connect(&mut self) {
///         self.O.connect();
///     }
///
///      fn hdl(&self) -> Verilog {
///         Verilog::Blackbox(BlackBox {
///           code: r#"
/// (* blackbox *)
/// module IBUFDS(I, B, O);
///   input I;
///   input B;
///   output O;
/// endmodule
/// "#.into(),
///           name: "IBUFDS".into()
///         })
///      }
/// }
///
/// // For technical reasons, the top circuit of a RustHDL firmware
/// // cannot be a black box.  So we use TopWrap to wrap it with one.
/// let mut x = TopWrap::new(ClockDriver::default());
/// x.uut.I.connect(); // Drive the positive clock from outside
/// x.uut.B.connect(); // Drive the negative clock from outside
/// x.connect_all();     // Wire up x and its internal components
/// let v = generate_verilog(&x);  // Generates verilog and validates it
/// yosys_validate("clock", &v)?;
/// # Ok::<(), SynthError>(())
/// ```
///
/// Wrapping Parameteric IP Cores
///
/// The [BlackBox] variant has a couple of peculiarities that we have hidden in this
/// example.  First, note that we pass the module name back to RustHDL in the
/// [BlackBox] instantiation.  This is because RustHDL needs to know what the module
/// is called so it can refer to it in the generated code.
///
/// In general RustHDL tries to avoid contention between modules with the same
/// name by automatically namespacing them.  That means that if you have a
/// module that is used in two different places in your code, it will get
/// two different names.  This is because of the parametric nature of the
/// generated code.  RustHDL does not assume (or know) that your two modules
/// will generate identical Verilog.  So it assumes they will be different
/// and creates two different named instances.
///
/// To see how that works, let's create a minimum example.  For fun, we will
/// use a single bit inverter.
/// ```rust
/// # use rust_hdl_core::prelude::*;
///
/// // First a basic inverter example
/// #[derive(LogicBlock, Default)]
/// struct Inverter {
///    sig_in: Signal<In, Bit>,
///    sig_out: Signal<Out, Bit>,
/// }
///
/// // All it does is set the output signal to the inverse
/// // of the input signal.
/// impl Logic for Inverter {
///    #[hdl_gen]
///    fn update(&mut self) {
///        self.sig_out.next = !self.sig_in.val();
///    }
/// }
///
/// // Now we create a circuit with 2 inverters connected
/// // back to back.  The net result is a do-nothing (buffer?).
/// #[derive(LogicBlock, Default)]
/// struct DoubleKnot {
///    sig_in: Signal<In, Bit>,
///    sig_out: Signal<Out, Bit>,
///    knot_1: Inverter,
///    knot_2: Inverter,
/// }
///
/// impl Logic for DoubleKnot {
///    #[hdl_gen]
///    fn update(&mut self) {
///       self.knot_1.sig_in.next = self.sig_in.val();
///       self.knot_2.sig_in.next = self.knot_1.sig_out.val();
///       self.sig_out.next = self.knot_2.sig_out.val();
///    }
/// }
///
/// // Now, let's create a [DoubleKnot] and see what we get
/// let mut x = DoubleKnot::default();
/// // The `sig_in` input on `x` needs to be driven or connected
/// x.sig_in.connect();
/// x.connect_all();
/// let v = generate_verilog(&x);
/// // If you examine the generated code, you will see it contains
/// // two instances of modules, one named `top$knot_1` and the
/// assert!(v.contains("top$knot_1 knot_1"));
/// // and the second is `top$knot_2`.
/// assert!(v.contains("top$knot_2 knot_2"));
/// ```
/// The problem arises when you use a [BlackBox] Verilog declaration.
/// In particular, RustHDL does not wrap your declaration (the Verilog is
/// just copied to the output), so it does not know that two different
/// instances of the same blockbox IP may represent different things.
/// A classic case is in the case of a parameterized blackbox IP core.
/// In that case, it is up to you to rename the different IP cores so that
/// they do not conflict.  A better way around this is to use the [Wrapper]
/// variant, since that is easier to use in most cases.
#[derive(Debug, Clone)]
pub struct BlackBox {
    pub code: String,
    pub name: String,
}

/// The Wrapper struct provides a more convenient and flexible way to wrap external
/// IP cores than [BlackBox].
///
/// While you can wrap IP cores with [BlackBox], it has some limitations.
/// There are two significant limits to using [BlackBox] to wrap IP cores,
/// and [Wrapper] tries to fix them both.
///
/// - If your IP cores are parametric (for example, they take a
/// parameter to determine an address or bitwidth), you must give them
/// unique names to avoid problems with your toolchain.
/// - You cannot rename or otherwise change any of the signal names
/// going into the IP core when you use [BlackBox].
///
/// Using [Wrapper] addresses both problems.  To address the first
/// problem, RustHDL (when using [Wrapper]), creates a wrapper module
/// that hides the wrapped core from the global scope.  This additional
/// level of scoping allows you to parameterize/customize the external
/// IP core, without causing conflicts at the global scope.  The
/// second problem is addressed also, since the [Wrapper] struct allows
/// you to write Verilog "glue code" to either simplify or otherwise
/// fix up the interface between the IP core and the RustHDL firmware
/// around it.
///
/// To use the [Wrapper] you must provide a custom implementation of the [hdl]
/// function in the [Logic] trait.  The [Wrapper] variant has two members.
/// The first member is the [code] where you can write the Verilog glue
/// code to instantiate the IP core, parameterize it, and connect it to the
/// inputs and outputs of the RustHDL object.  The second member is the [cores]
/// member, where you provide whatever blackbox code is required for the
/// toolchain to accept your verilog.  This typically varies by toolchain.
/// To get `yosys` to accept the verilog, you will need to provide `(* blackbox *)`
/// attributes and module definitions for each external IP core.
///
/// Let's look at some examples.
///
/// Examples
///
/// In the [BlackBox] case, we looked at wrapping a clock buffer into an IP
/// core.  Let's redo the same exercise, but with slightly better ergonomics.
/// Here is the definition of the IP core provided by the FPGA vendor
/// ```verilog
/// module IBUFDS(I, B, O);
///    input I;
///    input B;
///    output O;
/// endmodule
/// ```
/// This core is very simple, but we will try and improve the ergonomics
/// of it, and add a simulation model.
///
/// ```rust
/// # use rust_hdl_core::prelude::*;
/// pub struct ClockDriver {
///    pub clock_p: Signal<In, Clock>,
///    pub clock_n: Signal<In, Clock>,
///    pub sys_clock: Signal<Out, Clock>,
/// }
/// ```
///
/// This time, our ClockDriver can use reasonable signal names, because
/// we will use the glue layer to connect it to the IP core.  That glue
/// layer is very helpful for remapping signals, combining them or
/// assigning constant values.
///
/// We will also add a simulation model this time, to demonstrate how
/// to do that for an external core.
///
/// As in the case of [BlackBox], we will use
/// the [LogicBlock] derive macro to add the [Logic]
/// trait to our circuit (so RustHDL can work with it), and the
/// [Default] trait as well, to make it easy to use.
/// The [Logic] trait for this circuit will need to be implemented
/// by hand.
///
/// ```rust
/// # use rust_hdl_core:: prelude::*;
/// # #[derive(LogicBlock, Default)]
/// # pub struct ClockDriver {
/// #   pub clock_p: Signal<In, Clock>,
/// #   pub clock_n: Signal<In, Clock>,
/// #   pub sys_clock: Signal<Out, Clock>,
/// # }
///
/// impl Logic for ClockDriver {
///     fn update(&mut self) {
///         todo!()
///     }
///
///     fn connect(&mut self) {
///         todo!()
///     }
///
///     fn hdl(&self) -> Verilog {
///         todo!()
///     }
/// }
/// ```
///
/// The [Logic] trait requires 3 methods [Logic::update], [Logic::connect],
/// and [Logic::hdl].  The [Logic::update] method is used for simulation, and
/// at the moment, black box modules are not simulatable.  So we can accept
/// the default implementation of this.  The [Logic::connect] method is used
/// to indicate which members of the circuit are _driven_ by the circuit.
/// A better name might have been `drive`, instead of `connect`, but we will
/// stick with the current terminology.  You can think of it in terms of an
/// integrated circuit - outputs, are generally driven and are internally
/// *connected*, while inputs are generally driven from outside and are externally
/// connected.
///
/// We also want to create a simulation model for our IP core.  This is how
/// RustHDL will know how to include the behavior of the core when it is
/// integrated into simulations.  You can skip this step, of course, but
/// then your black box IP cores will be pretty useless for simulation
/// purposes.
///
/// A double-to-single ended clock driver is a fairly complicated
/// piece of analog circuitry.  It normally sends a clock edge when
/// the positive and negative going clocks cross.  For well behaved
/// differential clocks (which is likely the case in simulation),
/// this amounts to just buffering the positive clock, and ignoring
/// the negative clock.  We will need to build a simulation model that
/// includes enough detail to make it useful, but obviously, the
/// fidelity will be limited.  For this example, we will opt to simply
/// ignore the negative going clock, and forwarding the positive going clock
/// (not a good idea in practice, but for simulations it's fine).
///
/// ```rust
/// # use rust_hdl_core:: prelude::*;
/// # #[derive(LogicBlock, Default)]
/// # pub struct ClockDriver {
/// #   pub clock_p: Signal<In, Clock>,
/// #   pub clock_n: Signal<In, Clock>,
/// #   pub sys_clock: Signal<Out, Clock>,
/// # }
///
/// impl Logic for ClockDriver {
///     fn update(&mut self) {
///         self.sys_clock.next = self.clock_p.val();
///     }
///
///     fn connect(&mut self) {
///         self.sys_clock.connect();
///     }
///
///     fn hdl(&self) -> Verilog {
///         todo!()
///     }
/// }
/// ```
///
/// Now, we need an implementation for the HDL for this Clock driver.
/// That is where we need the [Wrapper] struct.
///
/// ```rust
/// # use rust_hdl_core::prelude::*;
/// # #[derive(LogicBlock, Default)]
/// # pub struct ClockDriver {
/// #   pub clock_p: Signal<In, Clock>,
/// #   pub clock_n: Signal<In, Clock>,
/// #   pub sys_clock: Signal<Out, Clock>,
/// # }
///
/// impl Logic for ClockDriver {
///     fn update(&mut self) {
///         self.sys_clock.next = self.clock_p.val();
///     }
///
///     fn connect(&mut self) {
///         self.sys_clock.connect();
///     }
///
///      fn hdl(&self) -> Verilog {
///         Verilog::Wrapper(Wrapper {
///           code: r#"
///     // We can remap the names here
///     IBUFDS ibufds_inst(.I(clock_p), .B(clock_n), .O(sys_clock));
///
/// "#.into(),
///           cores: r#"
/// (* blackbox *)
/// module IBUFDS(I, B, O)
///   input I;
///   input B;
///   output O;
/// endmodule"#.into(),
///         })
///      }
/// }
/// ```
///
/// With all 3 of the methods implemented, we can now create an instance
/// of our clock driver, synthesize it, and test it.  Here is the completed
/// example:
///
/// ```rust
/// # use rust_hdl_core::prelude::*;
///
/// #[derive(LogicBlock, Default)]
/// pub struct ClockDriver {
///   pub clock_p: Signal<In, Clock>,
///   pub clock_n: Signal<In, Clock>,
///   pub sys_clock: Signal<Out, Clock>,
/// }
///
/// impl Logic for ClockDriver {
///     fn update(&mut self) {
///         self.sys_clock.next = self.clock_p.val();
///     }
///
///     fn connect(&mut self) {
///         self.sys_clock.connect();
///     }
///
///      fn hdl(&self) -> Verilog {
///         Verilog::Wrapper(Wrapper {
///           code: r#"
///     // This is basically arbitrary Verilog code that lives inside
///     // a scoped module generated by RustHDL.  Whatever IP cores you
///     // use here must have accompanying core declarations in the
///     // cores string, or they will fail verification.
///     //
///     // In this simple case, we remap the names here
///     IBUFDS ibufds_inst(.I(clock_p), .B(clock_n), .O(sys_clock));
///
/// "#.into(),
///           cores: r#"
/// (* blackbox *)
/// module IBUFDS(I, B, O);
///   input I;
///   input B;
///   output O;
/// endmodule"#.into(),
///         })
///      }
/// }
///
/// // Let's create our ClockDriver.  No [TopWrap] is required here.
/// let mut x = ClockDriver::default();
/// x.clock_p.connect(); // Drive the positive clock from outside
/// x.clock_n.connect(); // Drive the negative clock from outside
/// x.connect_all();     // Wire up x and its internal components
/// let v = generate_verilog(&x);  // Generates verilog and validates it
/// yosys_validate("clock", &v)?;
/// # Ok::<(), SynthError>(())
/// ```
///
#[derive(Debug, Clone)]
pub struct Wrapper {
    pub code: String,
    pub cores: String,
}

#[derive(Debug, Clone)]
pub enum Verilog {
    /// Use [Empty] when you do not want a module represented in Verilog at all
    Empty,
    #[doc(hidden)]
    Combinatorial(VerilogBlock),
    /// Custom Verilog for a RustHDL module
    Custom(String),
    /// Blackbox for referencing IP cores.
    Blackbox(BlackBox),
    /// Wrap an external IP core or Verilog code into a RustHDL module.
    Wrapper(Wrapper),
}

impl Default for Verilog {
    fn default() -> Self {
        Self::Empty
    }
}

#[doc(hidden)]
pub type VerilogBlock = Vec<VerilogStatement>;

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum VerilogStatement {
    Assignment(VerilogExpression, VerilogExpression),
    SliceAssignment {
        base: String,
        width: usize,
        offset: VerilogExpression,
        replacement: VerilogExpression,
    },
    If(VerilogConditional),
    Match(VerilogMatch),
    Loop(VerilogLoop),
    Comment(String),
    Link(Vec<VerilogLink>),
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum VerilogLink {
    Forward(VerilogLinkDetails),
    Backward(VerilogLinkDetails),
    Bidirectional(VerilogLinkDetails),
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VerilogLinkDetails {
    pub my_name: String,
    pub owner_name: String,
    pub other_name: String,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VerilogIndexAssignment {
    pub target: VerilogExpression,
    pub index: VerilogExpression,
    pub value: VerilogExpression,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VerilogConditional {
    pub test: VerilogExpression,
    pub then: VerilogBlock,
    pub otherwise: VerilogBlockOrConditional,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VerilogLoop {
    pub index: String,
    pub from: VerilogLiteral,
    pub to: VerilogLiteral,
    pub block: VerilogBlock,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum VerilogBlockOrConditional {
    Block(VerilogBlock),
    Conditional(Box<VerilogStatement>),
    None,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VerilogMatch {
    pub test: VerilogExpression,
    pub cases: Vec<VerilogCase>,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VerilogCase {
    pub condition: String,
    pub block: VerilogBlock,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VerilogLiteral {
    val: BigUint,
    bits: usize,
}

impl VerilogLiteral {
    pub fn as_usize(&self) -> usize {
        let m = self.val.to_u32_digits();
        match m.len() {
            0 => 0,
            1 => m[0] as usize,
            _ => panic!("Loop index is too large!"),
        }
    }
}

impl From<bool> for VerilogLiteral {
    fn from(x: bool) -> Self {
        let bi: BigUint = if x { 1_u32 } else { 0_u32 }.into();
        VerilogLiteral { val: bi, bits: 1 }
    }
}

macro_rules! define_literal_from_uint {
    ($name: ident, $width: expr) => {
        impl From<$name> for VerilogLiteral {
            fn from(x: $name) -> Self {
                let bi: BigUint = x.into();
                VerilogLiteral {
                    val: bi,
                    bits: $width,
                }
            }
        }
    };
}

define_literal_from_uint!(u128, 128);
define_literal_from_uint!(u64, 64);
define_literal_from_uint!(u32, 32);
define_literal_from_uint!(u16, 16);
define_literal_from_uint!(u8, 8);
#[cfg(target_pointer_width = "64")]
define_literal_from_uint!(usize, 64);
#[cfg(target_pointer_width = "32")]
define_literal_from_uint!(usize, 32);

impl<const N: usize> From<Bits<N>> for VerilogLiteral {
    fn from(x: Bits<N>) -> Self {
        let mut z = BigUint::default();
        for i in 0..N {
            z.set_bit(i as u64, x.get_bit(i));
        }
        VerilogLiteral { val: z, bits: N }
    }
}

impl Display for VerilogLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bits = self.bits;
        Display::fmt(&bits, f)?;
        Display::fmt("'", f)?;
        if bits % 4 != 0 && self.bits < 20 {
            Display::fmt("b", f)?;
            std::fmt::Binary::fmt(&self.val, f)
        } else {
            Display::fmt("h", f)?;
            std::fmt::LowerHex::fmt(&self.val, f)
        }
    }
}

impl LowerHex for VerilogLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bits = self.bits;
        Display::fmt(&bits, f)?;
        Display::fmt("'h", f)?;
        LowerHex::fmt(&self.val, f)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum VerilogExpression {
    Signal(String),
    Literal(VerilogLiteral),
    Cast(Box<VerilogExpression>, usize),
    Paren(Box<VerilogExpression>),
    Binary(Box<VerilogExpression>, VerilogOp, Box<VerilogExpression>),
    Unary(VerilogOpUnary, Box<VerilogExpression>),
    Index(Box<VerilogExpression>, Box<VerilogExpression>),
    Slice(Box<VerilogExpression>, usize, Box<VerilogExpression>),
    IndexReplace(
        Box<VerilogExpression>,
        Box<VerilogExpression>,
        Box<VerilogExpression>,
    ),
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum VerilogOp {
    Add,
    Sub,
    Mul,
    LogicalAnd,
    LogicalOr,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum VerilogOpUnary {
    Not,
    Neg,
    All,
    Any,
    Xor,
}
