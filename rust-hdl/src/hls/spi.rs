use crate::core::prelude::*;
use crate::hls::bridge::Bridge;
use crate::hls::bus::{SoCBusResponder, SoCPortController};
use crate::hls::miso_wide_port::MISOWidePort;
use crate::hls::mosi_port::MOSIPort;
use crate::hls::mosi_wide_port::MOSIWidePort;
use crate::hls::HLSNamedPorts;
use crate::widgets::prelude::*;
use crate::widgets::spi::mux::MuxSlaves;

// HLS ports
// 0 - data in
// 1 - data out
// 2 - width in
// 3 - start/type
#[derive(LogicBlock)]
pub struct HLSSPIMaster<const D: usize, const A: usize, const W: usize> {
    pub spi: SPIWiresMaster,
    pub upstream: SoCBusResponder<D, A>,
    bridge: Bridge<D, A, 4>,
    data_outbound: MOSIWidePort<W, D>,
    data_inbound: MISOWidePort<W, D>,
    num_bits: MOSIPort<D>,
    start: MOSIPort<D>,
    core: SPIMaster<W>,
}

impl<const D: usize, const A: usize, const W: usize> HLSNamedPorts for HLSSPIMaster<D, A, W> {
    fn ports(&self) -> Vec<String> {
        self.bridge.ports()
    }
}

impl<const D: usize, const A: usize, const W: usize> Logic for HLSSPIMaster<D, A, W> {
    #[hdl_gen]
    fn update(&mut self) {
        self.core.clock.next = self.bridge.clock_out.val();
        self.core.reset.next = self.bridge.reset_out.val();
        self.core.data_outbound.next = self.data_outbound.port_out.val();
        self.data_inbound.port_in.next = self.core.data_inbound.val();
        self.data_inbound.strobe_in.next = self.core.transfer_done.val();
        self.core.bits_outbound.next = bit_cast::<16, D>(self.num_bits.port_out.val());
        self.core.continued_transaction.next = self.start.port_out.val().get_bit(0);
        self.core.start_send.next = self.start.strobe_out.val();
        SoCBusResponder::<D, A>::link(&mut self.upstream, &mut self.bridge.upstream);
        SoCPortController::<D>::join(&mut self.bridge.nodes[0], &mut self.data_outbound.bus);
        SoCPortController::<D>::join(&mut self.bridge.nodes[1], &mut self.data_inbound.bus);
        SoCPortController::<D>::join(&mut self.bridge.nodes[2], &mut self.num_bits.bus);
        SoCPortController::<D>::join(&mut self.bridge.nodes[3], &mut self.start.bus);
        SPIWiresMaster::link(&mut self.spi, &mut self.core.wires);
        self.num_bits.ready.next = true;
        self.start.ready.next = !self.core.busy.val();
    }
}

impl<const D: usize, const A: usize, const W: usize> HLSSPIMaster<D, A, W> {
    pub fn new(config: SPIConfig) -> Self {
        Self {
            spi: Default::default(),
            upstream: Default::default(),
            bridge: Bridge::new(["data_outbound", "data_inbound", "num_bits", "start_flag"]),
            data_outbound: Default::default(),
            data_inbound: Default::default(),
            num_bits: Default::default(),
            start: Default::default(),
            core: SPIMaster::new(config),
        }
    }
}

#[test]
fn test_hls_spi_master_is_synthesizable() {
    let spi_config = SPIConfig {
        clock_speed: 48_000_000,
        cs_off: true,
        mosi_off: true,
        speed_hz: 1_000_000,
        cpha: true,
        cpol: true,
    };
    let mut uut = HLSSPIMaster::<16, 8, 64>::new(spi_config);
    uut.upstream.link_connect_dest();
    uut.spi.link_connect_dest();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hls_spi", &vlog).unwrap();
}

#[derive(LogicBlock)]
pub struct HLSSPIMasterDynamicMode<const D: usize, const A: usize, const W: usize> {
    pub spi: SPIWiresMaster,
    pub upstream: SoCBusResponder<D, A>,
    bridge: Bridge<D, A, 4>,
    data_outbound: MOSIWidePort<W, D>,
    data_inbound: MISOWidePort<W, D>,
    num_bits_mode: MOSIPort<D>,
    start: MOSIPort<D>,
    core: SPIMasterDynamicMode<W>,
}

impl<const D: usize, const A: usize, const W: usize> HLSNamedPorts
    for HLSSPIMasterDynamicMode<D, A, W>
{
    fn ports(&self) -> Vec<String> {
        self.bridge.ports()
    }
}

// Assert D >= 8 + 2

impl<const D: usize, const A: usize, const W: usize> Logic for HLSSPIMasterDynamicMode<D, A, W> {
    #[hdl_gen]
    fn update(&mut self) {
        self.core.clock.next = self.bridge.clock_out.val();
        self.core.reset.next = self.bridge.reset_out.val();
        self.core.data_outbound.next = self.data_outbound.port_out.val();
        self.data_inbound.port_in.next = self.core.data_inbound.val();
        self.data_inbound.strobe_in.next = self.core.transfer_done.val();
        self.core.bits_outbound.next = bit_cast::<16, D>(self.num_bits_mode.port_out.val());
        self.core.continued_transaction.next = self.start.port_out.val().get_bit(0);
        self.core.start_send.next = self.start.strobe_out.val();
        SoCBusResponder::<D, A>::link(&mut self.upstream, &mut self.bridge.upstream);
        SoCPortController::<D>::join(&mut self.bridge.nodes[0], &mut self.data_outbound.bus);
        SoCPortController::<D>::join(&mut self.bridge.nodes[1], &mut self.data_inbound.bus);
        SoCPortController::<D>::join(&mut self.bridge.nodes[2], &mut self.num_bits_mode.bus);
        SoCPortController::<D>::join(&mut self.bridge.nodes[3], &mut self.start.bus);
        SPIWiresMaster::link(&mut self.spi, &mut self.core.wires);
        self.num_bits_mode.ready.next = true;
        self.start.ready.next = !self.core.busy.val();
    }
}

impl<const D: usize, const A: usize, const W: usize> HLSSPIMasterDynamicMode<D, A, W> {
    pub fn new(config: SPIConfigDynamicMode) -> Self {
        Self {
            spi: Default::default(),
            upstream: Default::default(),
            bridge: Bridge::new([
                "data_outbound",
                "data_inbound",
                "num_bits_mode",
                "start_flag",
            ]),
            data_outbound: Default::default(),
            data_inbound: Default::default(),
            num_bits_mode: Default::default(),
            start: Default::default(),
            core: SPIMasterDynamicMode::new(config),
        }
    }
}

#[test]
fn test_hls_spi_master_dynamic_mode_is_synthesizable() {
    let spi_config = SPIConfigDynamicMode {
        clock_speed: 48_000_000,
        cs_off: true,
        mosi_off: true,
        speed_hz: 1_000_000,
    };
    let mut uut = HLSSPIMasterDynamicMode::<16, 8, 64>::new(spi_config);
    uut.upstream.link_connect_dest();
    uut.spi.link_connect_dest();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hsl_spi_dm", &vlog).unwrap();
}

#[derive(LogicBlock)]
pub struct HLSSPIMuxSlaves<const D: usize, const A: usize, const N: usize> {
    pub to_slaves: [SPIWiresMaster; N],
    pub from_bus: SPIWiresSlave,
    pub upstream: SoCBusResponder<D, A>,
    mux: MuxSlaves<N, D>,
    bridge: Bridge<D, A, 1>,
    select: MOSIPort<D>,
}

impl<const D: usize, const A: usize, const N: usize> HLSNamedPorts for HLSSPIMuxSlaves<D, A, N> {
    fn ports(&self) -> Vec<String> {
        self.bridge.ports()
    }
}

impl<const D: usize, const A: usize, const N: usize> Logic for HLSSPIMuxSlaves<D, A, N> {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<D, A>::link(&mut self.upstream, &mut self.bridge.upstream);
        SoCPortController::<D>::join(&mut self.bridge.nodes[0], &mut self.select.bus);
        for i in 0_usize..N {
            SPIWiresMaster::link(&mut self.to_slaves[i], &mut self.mux.to_slaves[i]);
        }
        self.mux.sel.next = self.select.port_out.val();
        self.select.ready.next = true;
        SPIWiresSlave::link(&mut self.from_bus, &mut self.mux.from_master);
    }
}

impl<const D: usize, const A: usize, const N: usize> Default for HLSSPIMuxSlaves<D, A, N> {
    fn default() -> Self {
        assert!((1 << D) > N);
        Self {
            to_slaves: array_init::array_init(|_| Default::default()),
            upstream: Default::default(),
            from_bus: Default::default(),
            mux: Default::default(),
            bridge: Bridge::new(["select"]),
            select: Default::default(),
        }
    }
}

#[test]
fn test_hls_spi_mux_slaves_is_synthesizable() {
    let mut uut = TopWrap::new(HLSSPIMuxSlaves::<8, 16, 4>::default());
    uut.uut.upstream.link_connect_dest();
    for i in 0..4 {
        uut.uut.to_slaves[i].link_connect_source();
        uut.uut.to_slaves[i].miso.connect();
    }
    uut.uut.from_bus.link_connect_dest();
    uut.connect_all();
    yosys_validate("hls_spi_mux_slaves", &generate_verilog(&uut)).unwrap()
}

#[derive(LogicBlock)]
pub struct HLSSPIMuxMasters<const D: usize, const A: usize, const N: usize> {
    pub from_masters: [SPIWiresSlave; N],
    pub upstream: SoCBusResponder<D, A>,
    pub to_bus: SPIWiresMaster,
    mux: MuxMasters<N, D>,
    bridge: Bridge<D, A, 1>,
    select: MOSIPort<D>,
}

impl<const D: usize, const A: usize, const N: usize> HLSNamedPorts for HLSSPIMuxMasters<D, A, N> {
    fn ports(&self) -> Vec<String> {
        self.bridge.ports()
    }
}

impl<const D: usize, const A: usize, const N: usize> Logic for HLSSPIMuxMasters<D, A, N> {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<D, A>::link(&mut self.upstream, &mut self.bridge.upstream);
        SoCPortController::<D>::join(&mut self.bridge.nodes[0], &mut self.select.bus);
        for i in 0_usize..N {
            SPIWiresSlave::link(&mut self.from_masters[i], &mut self.mux.from_masters[i]);
        }
        self.mux.sel.next = self.select.port_out.val();
        self.select.ready.next = true;
        SPIWiresMaster::link(&mut self.to_bus, &mut self.mux.to_bus);
    }
}

impl<const D: usize, const A: usize, const N: usize> Default for HLSSPIMuxMasters<D, A, N> {
    fn default() -> Self {
        assert!((1 << D) > N);
        Self {
            from_masters: array_init::array_init(|_| Default::default()),
            upstream: Default::default(),
            to_bus: Default::default(),
            mux: Default::default(),
            bridge: Bridge::new(["select"]),
            select: Default::default(),
        }
    }
}

#[test]
fn test_hls_spi_mux_is_synthesizable() {
    let mut uut = TopWrap::new(HLSSPIMuxMasters::<8, 16, 4>::default());
    uut.uut.upstream.link_connect_dest();
    for i in 0..4 {
        uut.uut.from_masters[i].link_connect_dest();
    }
    uut.uut.to_bus.link_connect_source();
    uut.uut.to_bus.miso.connect();
    uut.connect_all();
    yosys_validate("hls_spi_mux", &generate_verilog(&uut)).unwrap()
}
