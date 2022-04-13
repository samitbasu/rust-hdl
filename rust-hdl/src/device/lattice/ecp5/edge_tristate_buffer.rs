use crate::core::prelude::*;
use crate::dff_setup;
use crate::widgets::prelude::{TristateBuffer, DFF};

#[derive(LogicBlock, Default)]
pub struct EdgeTristateBuffer<T: Synth> {
    pub to_pin: Signal<In, T>,
    pub from_pin: Signal<Out, T>,
    pub output_enable: Signal<In, Bit>,
    pub clk: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    pub pin: Signal<InOut, T>,
    dff_out: DFF<T>,
    dff_in: DFF<T>,
    buffer: TristateBuffer<T>,
}

fn wrapper_once() -> String {
    format!(
        r##"
    wire bb_to_pin;
    wire bb_from_pin;

    OFS1P3DX obuf(.D(to_pin), .CD(reset), .SP(1'b1), .SCLK(clk), .Q(bb_to_pin));
    IFS1P3DX ibuf(.D(bb_from_pin), .CD(reset), .SP(1'b1), .SCLK(clk), .Q(from_pin));
    BB bb(.I(bb_to_pin), .O(bb_from_pin), .B(pin), .T(~output_enable));
"##
    )
}

fn wrapper_multiple(count: usize) -> String {
    let bufs = (0..count)
        .map(|x| {
            format!(
                r#"
    OFS1P3DX obuf_{x}(.D(to_pin[{x}]), .CD(reset), .SP(1'b1), .SCLK(clk), .Q(bb_to_pin[{x}]));
    IFS1P3DX ibuf_{x}(.D(bb_from_pin[{x}]), .CD(reset), .SP(1'b1), .SCLK(clk), .Q(from_pin[{x}]));
    BB bb_{x}(.I(bb_to_pin[{x}]), .O(bb_from_pin[{x}]), .B(pin[{x}]), .T(~output_enable));
        "#,
                x = x
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r##"
wire [{B}:0] bb_to_pin;
wire [{B}:0] bb_from_pin;

{bufs}
    "##,
        B = count,
        bufs = bufs
    )
}

impl<T: Synth> Logic for EdgeTristateBuffer<T> {
    fn update(&mut self) {
        dff_setup!(self, clk, reset, dff_out, dff_in);
        self.buffer.write_enable.next = self.output_enable.val();
        self.dff_in.d.next = self.buffer.read_data.val();
        self.dff_out.d.next = self.to_pin.val();
        self.buffer.write_data.next = self.dff_out.q.val();
        self.from_pin.next = self.dff_in.q.val();
        Signal::<InOut, T>::link(&mut self.pin, &mut self.buffer.bus);
    }
    fn connect(&mut self) {
        self.dff_out.reset.connect();
        self.dff_in.reset.connect();
        self.dff_out.clock.connect();
        self.dff_in.clock.connect();
        self.buffer.write_enable.connect();
        self.dff_in.d.connect();
        self.dff_out.d.connect();
        self.buffer.write_data.connect();
        self.from_pin.connect();
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
module IFS1P3DX(input D, input SP, input SCLK, input CD, output Q);
endmodule

(* blackbox *)
module OFS1P3DX(input D, input SP, input SCLK, input CD, output Q);
endmodule

(* blackbox *)
module BB(input I, input T, output O, inout B);
endmodule

(* blackbox *)
module DELAYG(input A, output Z);
parameter DEL_MODE = "USER_DEFINED";
parameter DEL_VALUE = 0;
endmodule

            "##
            .into(),
        })
    }
}

#[test]
fn test_edge_buffer_synthesizes() {
    let mut uut = TopWrap::new(EdgeTristateBuffer::<Bits<8>>::default());
    uut.uut.output_enable.connect();
    uut.uut.to_pin.connect();
    uut.uut.clk.connect();
    uut.uut.reset.connect();
    uut.uut.pin.connect();
    uut.connect_all();
    std::fs::write("edge_tristate_buffer.v", generate_verilog(&uut)).unwrap();
    yosys_validate("edge_tristate_buffer", &generate_verilog(&uut)).unwrap();
}
