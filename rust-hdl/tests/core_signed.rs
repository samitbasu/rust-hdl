use rust_hdl::prelude::*;

#[derive(LogicBlock)]
struct CircuitSigned {
    x: Signal<In, Signed<32>>,
    y: Constant<Signed<32>>,
    z: Signal<Out, Signed<32>>,
}

impl Logic for CircuitSigned {
    #[hdl_gen]
    fn update(&mut self) {
        self.z.next = self.x.val() + self.y.val();
    }
}

impl Default for CircuitSigned {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Constant::new(Signed::from(-4935)),
            z: Default::default(),
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

// A test comparitor that makes a signed comparison internally
#[derive(LogicBlock, Default)]
struct TestCircuit {
    x: Signal<In, Bits<32>>,
    y: Signal<In, Bits<32>>,
    z: Signal<Out, Bit>,
}

impl Logic for TestCircuit {
    #[hdl_gen]
    fn update(&mut self) {
        self.z.next = signed_cast(self.x.val()) < signed_cast(self.y.val());
    }
}

#[test]
fn to_signed_bits_inline_works_issue_2() {
    use rand::Rng;
    let mut uut = TestCircuit::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    assert!(vlog.contains("$signed(x)"));
    assert!(vlog.contains("$signed(y)"));
    yosys_validate("to_signed_bits", &vlog).unwrap();
    (0..100_000).for_each(|_| {
        let (x, y) = rand::thread_rng().gen::<(u32, u32)>();
        uut.x.next = (x as u64).into();
        uut.y.next = (y as u64).into();
        assert!(simulate(&mut uut, 10));
        let cmp = (x as i32) < (y as i32);
        assert_eq!(uut.z.val(), cmp);
    });
}
