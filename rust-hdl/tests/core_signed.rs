use rust_hdl::core::prelude::*;

#[derive(LogicBlock)]
struct CircuitSigned {
    x: Signal<In, Signed<32>>,
    y: Constant<Signed<32>>,
}

impl Logic for CircuitSigned {
    fn update(&mut self) {
        self.x.next = self.x.val();
    }
}

impl Default for CircuitSigned {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Constant::new(Signed::from(-4935)),
        }
    }
}

#[test]
fn signed_vals_synthesize() {
    let mut uut = CircuitSigned::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("signed", &vlog).unwrap()
}
