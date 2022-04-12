use crate::core::prelude::*;
use crate::widgets::prelude::*;

#[derive(LogicBlock, Default)]
pub struct RegisteredEdgeTristate<const W: usize> {
    pub bus: Signal<InOut, Bits<W>>,
    pub write_enable: Signal<In, Bit>,
    pub write_data: Signal<In, Bits<W>>,
    pub read_data: Signal<Out, Bits<W>>,
    pub clock: Signal<In, Clock>,
    dff_out: DFF<Bits<W>>,
    dff_in: DFF<Bits<W>>,
}

impl<const W: usize> Logic for RegisteredEdgeTristate<W> {
    fn update(&mut self) {
        self.dff_out.clock.next = self.clock.val();
        self.dff_in.clock.next = self.clock.val();
        if self.write_enable.val() {
            self.bus.next = self.dff_out.q.val();
        }
        self.dff_in.d.next = self.bus.val();
        self.read_data.next = self.dff_in.q.val();
        self.bus.set_tristate_is_output(self.write_enable.val());
        self.dff_out.d.next = self.write_data.val();
    }
    fn connect(&mut self) {
        self.dff_out.clock.connect();
        self.dff_in.clock.connect();
        self.dff_in.d.connect();
        self.dff_out.d.connect();
        self.bus.connect();
        self.read_data.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: format!(
                r#"

reg [{WIDTH}:0] dff_in;
reg [{WIDTH}:0] dff_out;
assign bus = write_enable ? dff_out : {WIDTH}'bz;
assign read_data = dff_in;
always @(posedge clock) dff_in <= bus;
always @(posedge clock) dff_out <= write_data;
            "#,
                WIDTH = W - 1
            ),
            cores: r#""#.to_string(),
        })
    }
}

#[test]
fn test_tristate_edge_synthesizes() {
    let mut uut = TopWrap::new(RegisteredEdgeTristate::<8>::default());
    uut.uut.write_data.connect();
    uut.uut.write_enable.connect();
    uut.uut.clock.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("tristate_reg", &vlog).unwrap()
}
