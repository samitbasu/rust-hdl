use rust_hdl_core::prelude::*;
use crate::MHz48;

#[derive(Clone, Debug, LogicBlock)]
pub struct OpalKellyWireIn<const N: u8> {
    pub ok1: Signal<In, Bits<31>, MHz48>,
    pub ep_dataout: Signal<Out, Bits<16>, MHz48>,
}

impl<const N: u8> Default for OpalKellyWireIn<N> {
    fn default() -> Self {
        assert!(N < 0x20);
        Self {
            ok1: Signal::default(),
            ep_dataout: Signal::default(),
        }
    }
}

impl<const N: u8> Logic for OpalKellyWireIn<N> {
    fn update(&mut self) {
    }
    fn connect(&mut self) {
        self.ep_dataout.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("OpalKellyWireIn_{:x}", N);
        Verilog::Blackbox(
            BlackBox {
                code: format!(r#"
module {}
    (
    input wire [30:0] ok1,
    output wire [15:0] ep_dataout
    );

    okWireIn mod_wire(.ok1(ok1),
                  .ep_addr({:x}),
                  .ep_dataout(ep_dataout));
endmodule

(* blackbox *)
module okWireIn(
    input wire [30:0] ok1,
    input wire [7:0] ep_addr,
    output wire [15:0] ep_dataout
);
endmodule  "#, name, VerilogLiteral::from(N)),
                name,
            }
        )
    }
}