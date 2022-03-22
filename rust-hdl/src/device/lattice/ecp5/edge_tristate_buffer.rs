use crate::core::prelude::*;
use crate::widgets::prelude::{DFF, TristateBuffer};

#[derive(LogicBlock, Default)]
pub struct EdgeTristateBuffer<T: Synth> {
    pub to_pin: Signal<In, T>,
    pub from_pin: Signal<Out, T>,
    pub output_enable: Signal<In, Bit>,
    pub clk: Signal<In, Clock>,
    pub pin: Signal<InOut, T>,
    dff_out: DFF<T>,
    dff_in: DFF<T>,
    buffer: TristateBuffer<T>,
}

fn wrapper_once() -> &'static str {
    r##"
    wire bb_input;
    wire bb_output;

    OFS1P3DX obuf(.D(to_pin), .CD(0), .SP(1), .SCLK(clk), .Q(bb_input));
    IFS1P3DX ibuf(.D(bb_output), .CD(0), .SP(1), .SCLK(clk), .Q(from_pin));
    BB bb(.I(bb_input), .O(bb_output), .B(pin), .T(output_enable));
"##
}

fn wrapper_multiple(count: usize) -> String {
    let bufs = (0..count).map(|x|
        format!("
    OFS1P3DX obuf_{x}(.D(to_pin[{x}]), .CD(0), .SP(1), .SCLK(clk), .Q(bb_input[{x}]));
    IFS1P3DX ibuf_{x}(.D(bb_output[{x}]), .CD(0), .SP(1), .SCLK(clk), .Q(from_pin[{x}]));
    BB bb_{x}(.I(bb_input[{x}]), .O(bb_output[{x}]), .B(pin[{x}]), .T(output_enable));
        ", x=x)).collect::<Vec<_>>().join("\n");
    format!(r##"
wire [{B}:0] bb_input;
wire [{B}:0] bb_output;

{bufs}
    "##, B=count, bufs=bufs)
}


impl<T: Synth> Logic for EdgeTristateBuffer<T> {
    fn update(&mut self) {
        self.dff_out.clk.next = self.clk.val();
        self.dff_in.clk.next = self.clk.val();
        self.buffer.write_enable.next = self.output_enable.val();
        self.dff_in.d.next = self.buffer.read_data.val();
        self.dff_out.d.next = self.to_pin.val();
        self.buffer.write_data.next = self.dff_out.q.val();
        self.from_pin.next = self.dff_in.q.val();
        Signal::<InOut, T>::link(&mut self.pin, &mut self.buffer.bus);
    }
    fn connect(&mut self) {
        self.dff_out.clk.connect();
        self.dff_in.clk.connect();
        self.buffer.write_enable.connect();
        self.dff_in.d.connect();
        self.dff_out.d.connect();
        self.buffer.write_data.connect();
        self.from_pin.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code:
            if T::BITS == 1 {
                wrapper_once().to_string()
            } else {
                wrapper_multiple(T::BITS)
            },
            cores: r##"
(* blackbox *)
module IFS1P3DX(input D, input SP, input SCLK, input CD, output Q);
endmodule

(* blackbox *)
module OFS1P3DX(input D, input SP, input SCLK, input CD, output Q);
endmodule

(* blackbox *)
module BB(input I, input T, output O, inout B);
endmodule
            "##.into(),
        })
    }
}

#[test]
fn test_edge_buffer_synthesizes() {
    let mut uut = TopWrap::new(EdgeTristateBuffer::<Bits<8>>::default());
    uut.uut.output_enable.connect();
    uut.uut.to_pin.connect();
    uut.uut.clk.connect();
    uut.uut.pin.connect();
    uut.connect_all();
    std::fs::write("edge_tristate_buffer.v", generate_verilog(&uut)).unwrap();
    yosys_validate("edge_tristate_buffer", &generate_verilog(&uut)).unwrap();
}
