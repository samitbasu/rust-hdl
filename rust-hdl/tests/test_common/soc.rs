use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
pub struct SoCTestChip {
    pub clock: Signal<In, Clock>,
    pub sys_clock: Signal<In, Clock>,
    pub from_cpu: FIFOWriteResponder<Bits<16>>,
    pub to_cpu: FIFOReadResponder<Bits<16>>,
    from_cpu_fifo: AsyncFIFO<Bits<16>, 8, 9, 1>,
    to_cpu_fifo: AsyncFIFO<Bits<16>, 8, 9, 1>,
    soc_host: BaseController<8>,
    bridge: Bridge<16, 8, 2>,
    mosi_port: MOSIPort<16>, // At address
    miso_port: MISOPort<16>,
    data_fifo: SynchronousFIFO<Bits<16>, 8, 9, 1>,
}

impl Default for SoCTestChip {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            sys_clock: Default::default(),
            from_cpu: Default::default(),
            to_cpu: Default::default(),
            from_cpu_fifo: Default::default(),
            to_cpu_fifo: Default::default(),
            soc_host: Default::default(),
            bridge: Bridge::new(["mosi", "miso"]),
            mosi_port: Default::default(),
            miso_port: Default::default(),
            data_fifo: Default::default(),
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
        // Connect the controller to the bridge
        SoCBusController::<16, 8>::join(&mut self.soc_host.bus, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.mosi_port.bus);
        SoCPortController::<16>::join(&mut self.bridge.nodes[1], &mut self.miso_port.bus);
        clock!(self, sys_clock, data_fifo);
        // Wire the MOSI port to the input of the data_fifo
        self.data_fifo.data_in.next = self.mosi_port.port_out.val() << 1;
        self.data_fifo.write.next = self.mosi_port.strobe_out.val();
        self.mosi_port.ready.next = !self.data_fifo.full.val();
        // Wire the MISO port to the output of the data fifo
        self.miso_port.port_in.next = self.data_fifo.data_out.val();
        self.data_fifo.read.next = self.miso_port.strobe_out.val();
        self.miso_port.ready_in.next = !self.data_fifo.empty.val();
        // Wire the cpu fifos to the host
        FIFOWriteResponder::<Bits<16>>::link(&mut self.from_cpu, &mut self.from_cpu_fifo.bus_write);
        FIFOReadResponder::<Bits<16>>::link(&mut self.to_cpu, &mut self.to_cpu_fifo.bus_read);
        FIFOReadResponder::<Bits<16>>::join(
            &mut self.from_cpu_fifo.bus_read,
            &mut self.soc_host.from_cpu,
        );
        FIFOWriteResponder::<Bits<16>>::join(
            &mut self.to_cpu_fifo.bus_write,
            &mut self.soc_host.to_cpu,
        );
    }
}
