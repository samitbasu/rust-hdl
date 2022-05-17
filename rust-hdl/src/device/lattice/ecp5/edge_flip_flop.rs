use crate::core::prelude::*;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct EdgeFlipFlop<T: Synth> {
    pub d: Signal<In, T>,
    pub q: Signal<Out, T>,
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
}

fn wrapper_once() -> &'static str {
    r##"
OFS1P3DX inst_OFS1P3DX(.SCLK(clock), .SP(1'b1), .D(d), .Q(q), .CD(reset));
    "##
}

fn wrapper_multiple(count: usize) -> String {
    (0..count)
        .map(|x| {
            format!(
                "
OFS1P3DX ofs_{x}(.SCLK(clock), .SP(1'b1), .D(d[{x}]), .Q(q[{x}]), .CD(reset));
",
                x = x
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl<T: Synth> Logic for EdgeFlipFlop<T> {
    fn update(&mut self) {
        if self.clock.pos_edge() | self.reset.pos_edge() {
            if self.reset.val().into() {
                self.q.next = T::default();
            } else {
                self.q.next = self.d.val()
            }
        }
    }
    fn connect(&mut self) {
        self.q.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: if T::BITS == 1 {
                wrapper_once().to_string()
            } else {
                wrapper_multiple(T::BITS)
            },
            cores: r##"
(* blackbox *)
module OFS1P3DX(input D, input SP, input SCLK, input CD, output Q);
endmodule
            "##
            .into(),
        })
    }
}

#[test]
fn test_eflop_synthesizes() {
    let mut uut = EdgeFlipFlop::<Bits<8>>::default();
    uut.connect_all();
    yosys_validate("eflop", &generate_verilog(&uut)).unwrap();
}
