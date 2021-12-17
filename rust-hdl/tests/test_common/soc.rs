use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(Clone, Debug, Default, LogicInterface)]
pub struct ToControllerBus {
    pub to_controller: Signal<In, Bits<16>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub from_controller: Signal<Out, Bits<16>>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
}

#[derive(LogicBlock)]
pub struct SoCTestChip {
    pub clock: Signal<In, Clock>,
    pub sys_clock: Signal<In, Clock>,
    pub cpu_bus: ToControllerBus,
    from_cpu_fifo: AsynchronousFIFO<Bits<16>, 8, 9, 1>,
    to_cpu_fifo: AsynchronousFIFO<Bits<16>, 8, 9, 1>,
    soc_host: BaseController,
    mosi_port: MOSIPort<16, 8>, // At address
    miso_port: MISOPort<16, 8>,
    data_fifo: SynchronousFIFO<Bits<16>, 8, 9, 1>,
}

impl Default for SoCTestChip {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            sys_clock: Default::default(),
            cpu_bus: Default::default(),
            from_cpu_fifo: Default::default(),
            to_cpu_fifo: Default::default(),
            soc_host: Default::default(),
            mosi_port: MOSIPort::new(0x53_u8.into()),
            miso_port: MISOPort::new(0x54_u8.into()),
            data_fifo: Default::default()
        }
    }
}

impl Logic for SoCTestChip {
    #[hdl_gen]
    fn update(&mut self) {
        self.from_cpu_fifo.write_clock.next = self.clock.val();
        self.to_cpu_fifo.read_clock.next = self.clock.val();
        self.from_cpu_fifo.read_clock.next = self.sys_clock.val();
        self.to_cpu_fifo.write_clock.next = self.sys_clock.val();
        self.soc_host.clock.next = self.sys_clock.val();
        self.mosi_port.clock.next = self.sys_clock.val();
        self.miso_port.clock.next = self.sys_clock.val();
        self.data_fifo.clock.next = self.sys_clock.val();
        // Wire the ports to the host
        self.mosi_port.bus.from_master.next = self.soc_host.bus.from_master.val();
        self.mosi_port.bus.strobe.next = self.soc_host.bus.strobe.val();
        self.mosi_port.bus.addr.next = self.soc_host.bus.addr.val();
        self.miso_port.bus.from_master.next = self.soc_host.bus.from_master.val();
        self.miso_port.bus.strobe.next = self.soc_host.bus.strobe.val();
        self.miso_port.bus.addr.next = self.soc_host.bus.addr.val();
        self.soc_host.bus.to_master.next = self.miso_port.bus.to_master.val() |
            self.mosi_port.bus.to_master.val();
        self.soc_host.bus.ready.next = self.miso_port.bus.ready.val() |
            self.mosi_port.bus.ready.val();
        // Wire the MOSI port to the input of the data_fifo
        self.data_fifo.data_in.next = self.mosi_port.port_out.val() << 1_usize;
        self.data_fifo.write.next = self.mosi_port.strobe_out.val();
        self.mosi_port.ready.next = !self.data_fifo.full.val();
        // Wire the MISO port to the output of the data fifo
        self.miso_port.port_in.next = self.data_fifo.data_out.val();
        self.data_fifo.read.next = self.miso_port.strobe_out.val();
        self.miso_port.ready_in.next = !self.data_fifo.empty.val();
        // Wire the cpu fifos to the host
        self.soc_host.cpu.empty.next = self.from_cpu_fifo.empty.val();
        self.soc_host.cpu.from_bus.next = self.from_cpu_fifo.data_out.val();
        self.from_cpu_fifo.read.next = self.soc_host.cpu.read.val();
        self.soc_host.cpu.full.next = self.to_cpu_fifo.full.val();
        self.to_cpu_fifo.data_in.next = self.soc_host.cpu.to_bus.val();
        self.to_cpu_fifo.write.next = self.soc_host.cpu.write.val();
        // Wire the fifos to the cpu side bus
        self.from_cpu_fifo.data_in.next = self.cpu_bus.to_controller.val();
        self.from_cpu_fifo.write.next = self.cpu_bus.write.val();
        self.cpu_bus.full.next = self.from_cpu_fifo.full.val();
        self.cpu_bus.from_controller.next = self.to_cpu_fifo.data_out.val();
        self.cpu_bus.empty.next = self.to_cpu_fifo.empty.val();
        self.to_cpu_fifo.read.next = self.cpu_bus.read.val();
    }
}

#[test]
fn test_soc_test_chip_synthesizes() {
    let mut uut = SoCTestChip::default();
    uut.sys_clock.connect();
    uut.clock.connect();
    uut.cpu_bus.write.connect();
    uut.cpu_bus.to_controller.connect();
    uut.cpu_bus.read.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("soc_test", &vlog).unwrap();
}