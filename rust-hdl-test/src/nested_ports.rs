use rust_hdl_alchitry_cu::pins::Mhz100;
use rust_hdl_core::prelude::*;
use rust_hdl_macros::{hdl_gen, LogicBlock, LogicInterface};
use std::fs::File;

struct SignalLister {}

impl VerilogVisitor for SignalLister {
    fn visit_signal(&mut self, s: &str) {
        println!("Signal: {}", s);
    }
}

#[test]
fn test_write_modules_nested_ports() {
    #[derive(Clone, Debug, Default, LogicInterface)]
    struct MyBus<F: Domain> {
        pub data: FIFORead<F, 8>,
        pub cmd: FIFORead<F, 3>,
    }

    #[derive(Clone, Debug, Default, LogicInterface)]
    struct FIFORead<F: Domain, const D: usize> {
        pub read: Signal<In, Bit, F>,
        pub output: Signal<Out, Bits<D>, F>,
        pub empty: Signal<Out, Bit, F>,
        pub almost_empty: Signal<Out, Bit, F>,
        pub underflow: Signal<Out, Bit, F>,
    }

    #[derive(Clone, Debug, Default, LogicBlock)]
    struct Widget {
        pub clock: Signal<In, Clock, Mhz100>,
        pub bus: MyBus<Mhz100>,
    }

    impl Logic for Widget {
        fn update(&mut self) {}

        fn connect(&mut self) {
            self.bus.data.almost_empty.connect();
            self.bus.data.empty.connect();
            self.bus.data.underflow.connect();
            self.bus.data.output.connect();
            self.bus.cmd.almost_empty.connect();
            self.bus.cmd.empty.connect();
            self.bus.cmd.underflow.connect();
            self.bus.cmd.output.connect();
        }
    }

    #[derive(Clone, Debug, Default, LogicBlock)]
    struct UUT {
        pub bus: MyBus<Mhz100>,
        widget_a: Widget,
        widget_b: Widget,
        pub clock: Signal<In, Clock, Mhz100>,
        pub select: Signal<In, Bit, Mhz100>,
    }

    impl Logic for UUT {
        #[hdl_gen]
        fn update(&mut self) {
            self.widget_a.clock.next = self.clock.val();
            self.widget_b.clock.next = self.clock.val();

            if self.select.val().raw() {
                self.bus.cmd.underflow.next = self.widget_a.bus.cmd.underflow.val();
                self.bus.cmd.almost_empty.next = self.widget_a.bus.cmd.almost_empty.val();
                self.bus.cmd.empty.next = self.widget_a.bus.cmd.empty.val();
                self.bus.cmd.output.next = self.widget_a.bus.cmd.output.val() + 1_u32;
                self.widget_a.bus.cmd.read.next = self.bus.cmd.read.val();

                self.bus.data.underflow.next = self.widget_a.bus.data.underflow.val();
                self.bus.data.almost_empty.next = self.widget_a.bus.data.almost_empty.val();
                self.bus.data.empty.next = self.widget_a.bus.data.empty.val();
                self.bus.data.output.next = self.widget_a.bus.data.output.val();
                self.widget_a.bus.data.read.next = self.bus.data.read.val();
            } else {
                self.bus.cmd.underflow.next = self.widget_b.bus.cmd.underflow.val();
                self.bus.cmd.almost_empty.next = self.widget_b.bus.cmd.almost_empty.val();
                self.bus.cmd.empty.next = self.widget_b.bus.cmd.empty.val();
                self.bus.cmd.output.next = self.widget_b.bus.cmd.output.val();
                self.widget_b.bus.cmd.read.next = self.bus.cmd.read.val();

                self.bus.data.underflow.next = self.widget_b.bus.data.underflow.val();
                self.bus.data.almost_empty.next = self.widget_b.bus.data.almost_empty.val();
                self.bus.data.empty.next = self.widget_b.bus.data.empty.val();
                self.bus.data.output.next = self.widget_b.bus.data.output.val();
                self.widget_b.bus.data.read.next = self.bus.data.read.val();
            }
        }
    }

    let mut uut = UUT::default();
    uut.clock.connect();
    uut.bus.cmd.read.connect();
    uut.bus.data.read.connect();
    uut.select.connect();
    uut.connect_all();
    check_connected(&uut);
    let mut defines = ModuleDefines::default();
    uut.accept("uut", &mut defines);
    defines.defines();
    let code = uut.hdl();
    let mut sig = SignalLister {};
    let mut gen = VerilogCodeGenerator::new();
    if let rust_hdl_core::ast::Verilog::Combinatorial(q) = code {
        sig.visit_block(&q);
        gen.visit_block(&q);
    }
    println!("Code");
    println!("{}", gen.to_string());
    let mut jnk = File::create("test.vcd").unwrap();
    let dev = write_vcd_header(&mut jnk, &uut);
    let _dev = write_vcd_dump(dev, &uut);
}
