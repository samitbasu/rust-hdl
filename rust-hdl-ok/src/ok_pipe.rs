use crate::MHz48;
use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, Default, LogicBlock)]
pub struct PipeIn<const N: u8> {
    pub ok1: Signal<In, Bits<31>, MHz48>,
    pub ok2: Signal<Out, Bits<17>, MHz48>,
    pub write: Signal<Out, bool, MHz48>,
    pub dataout: Signal<Out, Bits<16>, MHz48>,
}

impl<const N: u8> Logic for PipeIn<N> {
    fn update(&mut self) {}
    fn connect(&mut self) {
        assert!(N >= 0x80 && N < 0xA0);
        self.ok2.connect();
        self.write.connect();
        self.dataout.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("PipeIn_{:x}", N);
        Verilog::Blackbox(BlackBox {
            code: format!(
                r#"
module {}
    (input wire  [30:0] ok1,
     output wire [16:0] ok2,
     output wire        write,
     output wire [15:0] dataout);

     okPipeIn mod(.ok1(ok1),.ok2(ok2),.ep_write(write),.ep_dataout(dataout),.ep_addr({:x}));
endmodule

(* blackbox *)
module okPipeIn(ok1, ok2, ep_addr, ep_write, ep_dataout);
	input  [30:0] ok1;
	output [16:0] ok2;
	input  [7:0]  ep_addr;
	output        ep_write;
	output [15:0] ep_dataout;
endmodule
                    "#,
                name,
                VerilogLiteral::from(N)
            ),
            name,
        })
    }
}
