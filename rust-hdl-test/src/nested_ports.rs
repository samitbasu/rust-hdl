use rust_hdl_core::bits::{Bit, Bits};
use rust_hdl_core::block::Block;
use rust_hdl_core::clock::Clock;
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::logic::Logic;
use rust_hdl_core::module_defines::ModuleDefines;
use rust_hdl_core::signal::Signal;
use rust_hdl_macros::{hdl_gen, LogicBlock, LogicInterface};
use rust_hdl_core::verilog_visitor::VerilogVisitor;
use rust_hdl_core::verilog_gen::VerilogCodeGenerator;

struct SignalLister {}

impl VerilogVisitor for SignalLister {
    fn visit_signal(&mut self, s: &str) {
        println!("Signal: {}", s);
    }
}

#[test]
fn test_write_modules_nested_ports() {
    #[derive(Clone, Debug, Default, LogicInterface)]
    struct MyBus {
        pub data: FIFORead<8>,
        pub cmd: FIFORead<3>,
    }

    #[derive(Clone, Debug, Default, LogicInterface)]
    struct FIFORead<const D: usize> {
        pub read: Signal<In, Bit>,
        pub output: Signal<Out, Bits<D>>,
        pub empty: Signal<Out, Bit>,
        pub almost_empty: Signal<Out, Bit>,
        pub underflow: Signal<Out, Bit>,
    }

    #[derive(Clone, Debug, Default, LogicBlock)]
    struct Widget {
        pub clock: Signal<In, Clock>,
        pub bus: MyBus,
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
        pub bus: MyBus,
        widget_a: Widget,
        widget_b: Widget,
        pub clock: Signal<In, Clock>,
        pub select: Signal<In, Bit>,
    }

    impl Logic for UUT {
        #[hdl_gen]
        fn update(&mut self) {
            self.widget_a.clock.next = self.clock.val;
            self.widget_b.clock.next = self.clock.val;

            if self.select.val {
                self.bus.cmd.underflow.next = self.widget_a.bus.cmd.underflow.val;
                self.bus.cmd.almost_empty.next = self.widget_a.bus.cmd.almost_empty.val;
                self.bus.cmd.empty.next = self.widget_a.bus.cmd.empty.val;
                self.bus.cmd.output.next = self.widget_a.bus.cmd.output.val;
                self.widget_a.bus.cmd.read.next = self.bus.cmd.read.val;

                self.bus.data.underflow.next = self.widget_a.bus.data.underflow.val;
                self.bus.data.almost_empty.next = self.widget_a.bus.data.almost_empty.val;
                self.bus.data.empty.next = self.widget_a.bus.data.empty.val;
                self.bus.data.output.next = self.widget_a.bus.data.output.val;
                self.widget_a.bus.data.read.next = self.bus.data.read.val;
            } else {
                self.bus.cmd.underflow.next = self.widget_b.bus.cmd.underflow.val;
                self.bus.cmd.almost_empty.next = self.widget_b.bus.cmd.almost_empty.val;
                self.bus.cmd.empty.next = self.widget_b.bus.cmd.empty.val;
                self.bus.cmd.output.next = self.widget_b.bus.cmd.output.val;
                self.widget_b.bus.cmd.read.next = self.bus.cmd.read.val;

                self.bus.data.underflow.next = self.widget_b.bus.data.underflow.val;
                self.bus.data.almost_empty.next = self.widget_b.bus.data.almost_empty.val;
                self.bus.data.empty.next = self.widget_b.bus.data.empty.val;
                self.bus.data.output.next = self.widget_b.bus.data.output.val;
                self.widget_b.bus.data.read.next = self.bus.data.read.val;
            }
        }

        fn connect(&mut self) {
            self.bus.cmd.underflow.connect();
            self.bus.cmd.almost_empty.connect();
            self.bus.cmd.empty.connect();
            self.bus.cmd.output.connect();
            self.widget_a.bus.cmd.read.connect();
            self.widget_a.bus.data.read.connect();
            self.widget_b.bus.cmd.read.connect();
            self.widget_b.bus.data.read.connect();
            self.widget_a.clock.connect();
            self.widget_b.clock.connect();
        }
    }

    let mut uut = UUT::default();
    uut.clock.connect();
    uut.bus.cmd.read.connect();
    uut.bus.data.read.connect();
    uut.select.connect();
    uut.connect_all();
    //        check_connected(&uut);
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
}
