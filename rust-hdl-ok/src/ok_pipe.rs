use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct BTPipeIn {
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub write: Signal<Out, bool>,
    pub blockstrobe: Signal<Out, bool>,
    pub dataout: Signal<Out, Bits<16>>,
    pub ready: Signal<In, bool>,
    _n: u8,
}

impl BTPipeIn {
    pub fn new(n: u8) -> Self {
        assert!(n >= 0x80 && n < 0xA0);
        Self {
            ok1: Default::default(),
            ok2: Default::default(),
            write: Default::default(),
            blockstrobe: Default::default(),
            dataout: Default::default(),
            ready: Default::default(),
            _n: n,
        }
    }
}

impl Logic for BTPipeIn {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.ok2.connect();
        self.write.connect();
        self.blockstrobe.connect();
        self.dataout.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("BTPipeIn_{:x}", self._n);
        Verilog::Blackbox(BlackBox {
            code: format!(
                r#"
module {}
    (input wire  [30:0] ok1,
     output wire [16:0] ok2,
     output wire        write,
     output wire        blockstrobe,
     output wire [15:0] dataout,
     input wire         ready);

     okBTPipeIn mod(.ok1(ok1),.ok2(ok2),.ep_write(write),
     .ep_blockstrobe(blockstrobe), .ep_dataout(dataout),
     .ep_ready(ready),.ep_addr({:x}));
endmodule

(* blackbox *)
module okBTPipeIn(ok1, ok2, ep_addr, ep_write, ep_blockstrobe, ep_dataout, ep_ready);
	input  [30:0] ok1;
	output [16:0] ok2;
	input  [7:0]  ep_addr;
	output        ep_write;
	output        ep_blockstrobe;
	output [15:0] ep_dataout;
	input         ep_ready;
endmodule
                    "#,
                name,
                VerilogLiteral::from(self._n)
            ),
            name,
        })
    }
}

#[test]
fn test_bt_pipein_synthesizes() {
    let mut uut = rust_hdl_synth::TopWrap::new(BTPipeIn::new(0x80));
    uut.uut.ok1.connect();
    uut.uut.ready.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("btpipein", &generate_verilog(&uut)).unwrap();
}

#[derive(Clone, Debug, LogicBlock)]
pub struct PipeIn {
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub write: Signal<Out, bool>,
    pub dataout: Signal<Out, Bits<16>>,
    _n: u8,
}

impl PipeIn {
    pub fn new(n: u8) -> Self {
        assert!(n >= 0x80 && n < 0xA0);
        Self {
            ok1: Default::default(),
            ok2: Default::default(),
            write: Default::default(),
            dataout: Default::default(),
            _n: n,
        }
    }
}

impl Logic for PipeIn {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.ok2.connect();
        self.write.connect();
        self.dataout.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("PipeIn_{:x}", self._n);
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
                VerilogLiteral::from(self._n)
            ),
            name,
        })
    }
}

#[test]
fn test_pipein_synthesizes() {
    let mut uut = rust_hdl_synth::TopWrap::new(PipeIn::new(0x80));
    uut.uut.ok1.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("pipein", &generate_verilog(&uut)).unwrap();
}

#[derive(Clone, Debug, LogicBlock)]
pub struct PipeOut {
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub read: Signal<Out, Bit>,
    pub datain: Signal<In, Bits<16>>,
    _n: u8,
}

impl PipeOut {
    pub fn new(n: u8) -> Self {
        assert!(n >= 0xA0 && n < 0xC0);
        Self {
            ok1: Default::default(),
            ok2: Default::default(),
            read: Default::default(),
            datain: Default::default(),
            _n: n,
        }
    }
}

impl Logic for PipeOut {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.ok2.connect();
        self.read.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("PipeOut_{:x}", self._n);
        Verilog::Blackbox(BlackBox {
            code: format!(
                r#"
module {}
    (input wire [30:0]  ok1,
     output wire [16:0] ok2,
     output wire        read,
     input wire [15:0] datain);

     okPipeOut mod(.ok1(ok1), .ok2(ok2), .ep_read(read), .ep_datain(datain), .ep_addr({:x}));
endmodule

(* blackbox *)
module okPipeOut(ok1, ok2, ep_addr, ep_read, ep_datain);
	input  [30:0] ok1;
	output [16:0] ok2;
	input  [7:0]  ep_addr;
	output        ep_read;
	input  [15:0] ep_datain;
endmodule
                "#,
                name,
                VerilogLiteral::from(self._n)
            ),
            name,
        })
    }
}

#[test]
fn test_pipeout_synthesizes() {
    use rust_hdl_synth::yosys_validate;

    let mut uut = rust_hdl_synth::TopWrap::new(PipeOut::new(0xA0));
    uut.uut.ok1.connect();
    uut.uut.datain.connect();
    uut.connect_all();
    yosys_validate("pipeout", &generate_verilog(&uut)).unwrap();
}

#[derive(Clone, Debug, LogicBlock)]
pub struct BTPipeOut {
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub read: Signal<Out, Bit>,
    pub blockstrobe: Signal<Out, Bit>,
    pub datain: Signal<In, Bits<16>>,
    pub ready: Signal<In, Bit>,
    _n: u8,
}

impl BTPipeOut {
    pub fn new(n: u8) -> Self {
        assert!(n >= 0xA0 && n < 0xC0);
        Self {
            ok1: Default::default(),
            ok2: Default::default(),
            read: Default::default(),
            blockstrobe: Default::default(),
            datain: Default::default(),
            ready: Default::default(),
            _n: n,
        }
    }
}

impl Logic for BTPipeOut {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.ok2.connect();
        self.read.connect();
        self.blockstrobe.connect();
    }
    fn hdl(&self) -> Verilog {
        let name = format!("BTPipeOut_{:x}", self._n);
        Verilog::Blackbox(BlackBox {
            code: format!(
                r#"
module {}
    (input wire [30:0]  ok1,
     output wire [16:0] ok2,
     output wire        read,
     output wire        blockstrobe,
     input wire [15:0]  datain,
     input wire         ready);

     okBTPipeOut mod(.ok1(ok1), .ok2(ok2),
        .ep_read(read), .ep_datain(datain),
        .ep_blockstrobe(blockstrobe), .ep_ready(ready),
        .ep_addr({:x}));
endmodule

(* blackbox *)
module okBTPipeOut(ok1, ok2, ep_addr, ep_read, ep_blockstrobe, ep_datain, ep_ready);
	input  [30:0] ok1;
	output [16:0] ok2;
	input  [7:0]  ep_addr;
	output        ep_read;
	output        ep_blockstrobe;
	input  [15:0] ep_datain;
	input         ep_ready;
endmodule
                "#,
                name,
                VerilogLiteral::from(self._n)
            ),
            name,
        })
    }
}

#[test]
fn test_btpipeout_synthesizes() {
    let mut uut = rust_hdl_synth::TopWrap::new(BTPipeOut::new(0xA0));
    uut.uut.ok1.connect();
    uut.uut.datain.connect();
    uut.uut.ready.connect();
    uut.connect_all();
    rust_hdl_synth::yosys_validate("btpipeout", &generate_verilog(&uut)).unwrap();
}
