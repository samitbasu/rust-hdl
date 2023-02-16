use rust_hdl_core::prelude::*;

/// Tristate Buffer
///
/// Most FPGAs do not support internal tristate logic.  Instead, the compilers turn tristate
/// logic into a combination of a pair of signals (one in, one out) and an enable line.  However,
/// the real world definitely needs tristate logic, and there are usually dedicated buffers
/// on the FPGA that can drive a tristate line using a pin that is appropriately configured.
///
/// Most FPGA toolchains can infer the tristate buffer when it's at the edge of the design. So
/// when you need a tristate buffer, you can use this struct.  Note that it is generic over
/// the signals being tristated.  So you can include a set of different tristate buffers with
/// a single entity.
///
/// ```rust
/// # use rust_hdl_core::prelude::*;
/// # use rust_hdl_widgets::prelude::*;
///
/// // An example of a simple tristate 8-bit bus
/// #[derive(LogicInterface, Default)]
/// struct EightBitBus {
///    bus: Signal<InOut, Bits<8>>,
/// }
/// ```
#[derive(LogicBlock, Default)]
pub struct TristateBuffer<D: Synth> {
    /// The tristated signals come out of this pin.  This should be a top level signal in your design.
    pub bus: Signal<InOut, D>,
    /// When asserted (true), the bus will attempt to drive `write_data` to the pins.
    pub write_enable: Signal<In, Bit>,
    /// The data to write to the bus.  Ignored when `write_enable` is not active (high).
    pub write_data: Signal<In, D>,
    /// The read back from the bus.  When `write_enable` is false, then this signal represents
    /// the external signals driving the FPGA pins.  For FPGA, this is likely equal to `write_data`
    /// when `write_enable` is true.
    pub read_data: Signal<Out, D>,
}

impl<D: Synth> Logic for TristateBuffer<D> {
    fn update(&mut self) {
        if self.write_enable.val() {
            self.bus.next = self.write_data.val();
        }
        self.read_data.next = self.bus.val();
        self.bus.set_tristate_is_output(self.write_enable.val());
    }

    fn connect(&mut self) {
        self.bus.connect();
        self.read_data.connect();
    }

    fn hdl(&self) -> Verilog {
        Verilog::Custom(format!(
            "\
    assign bus = write_enable ? write_data : {WIDTH}'bz;
always @(*) read_data = bus;",
            WIDTH = D::BITS
        ))
    }
}

#[test]
fn test_tristate_synthesizes() {
    let mut uut = TristateBuffer::<Bits<8>>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("tristate", &vlog).unwrap()
}
