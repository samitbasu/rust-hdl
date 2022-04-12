mod test_common;

#[cfg(feature = "frontpanel")]
use crate::test_common::tools::ok_test_prelude;
#[cfg(feature = "frontpanel")]
use rust_hdl::bsp::ok_core::ok_hls_bridge::{
    disable_streaming, drain_stream, enable_streaming, ping_bridge, read_data_from_address,
    stream_read, write_data_to_address, OKHLSBridgeAddressConfig, OpalKellyHLSBridge,
};
use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::bsp::ok_xem6010::XEM6010;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;
#[cfg(feature = "frontpanel")]
use rust_hdl_ok_frontpanel_sys::OkError;

#[cfg(feature = "frontpanel")]
#[derive(LogicBlock)]
struct OpalKellyHLSBridgeTest {
    hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    hls_bridge: OpalKellyHLSBridge<8>,
    port_bridge: Bridge<16, 8, 3>,
    mosi_port: MOSIPort<16>,
    miso_port: MISOPort<16>,
    stream_port: MISOPort<16>,
    data_fifo: SynchronousFIFO<Bits<16>, 8, 9, 1>,
    stream_cnt: DFF<Bits<16>>,
}

#[cfg(feature = "frontpanel")]
impl Default for OpalKellyHLSBridgeTest {
    fn default() -> Self {
        let port_bridge = Bridge::new(["mosi", "miso", "stream"]);
        Self {
            hi: XEM6010::hi(),
            ok_host: XEM6010::ok_host(),
            hls_bridge: Default::default(),
            port_bridge,
            mosi_port: Default::default(),
            miso_port: Default::default(),
            stream_port: Default::default(),
            data_fifo: Default::default(),
            stream_cnt: Default::default(),
        }
    }
}

#[cfg(feature = "frontpanel")]
impl Logic for OpalKellyHLSBridgeTest {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        self.hls_bridge.ti_clk.next = self.ok_host.ti_clk.val();
        self.stream_cnt.clock.next = self.ok_host.ti_clk.val();
        self.hls_bridge.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.hls_bridge.ok2.val();
        SoCBusController::<16, 8>::join(&mut self.hls_bridge.bus, &mut self.port_bridge.upstream);
        SoCPortController::<16>::join(&mut self.port_bridge.nodes[0], &mut self.mosi_port.bus);
        SoCPortController::<16>::join(&mut self.port_bridge.nodes[1], &mut self.miso_port.bus);
        SoCPortController::<16>::join(&mut self.port_bridge.nodes[2], &mut self.stream_port.bus);
        self.data_fifo.clock.next = self.ok_host.ti_clk.val();
        // Wire the MOSI port to the input of the data_fifo
        self.data_fifo.data_in.next = self.mosi_port.port_out.val() << 1_usize;
        self.data_fifo.write.next = self.mosi_port.strobe_out.val();
        self.mosi_port.ready.next = !self.data_fifo.full.val();
        // Wire the MISO port to the output of the data fifo
        self.miso_port.port_in.next = self.data_fifo.data_out.val();
        self.data_fifo.read.next = self.miso_port.strobe_out.val();
        self.miso_port.ready_in.next = !self.data_fifo.empty.val();
        // Connect a stream port to generate a rolling counter
        self.stream_cnt.d.next = self.stream_cnt.q.val();
        self.stream_port.ready_in.next = true;
        self.stream_port.port_in.next = self.stream_cnt.q.val();
        self.stream_cnt.d.next = self.stream_cnt.q.val() + self.stream_port.strobe_out.val();
    }
}

#[cfg(feature = "frontpanel")]
#[test]
fn test_ok_hls_bridge_test_synthesizes() {
    let mut uut = OpalKellyHLSBridgeTest::default();
    uut.hi.link_connect_dest();
    uut.connect_all();
    yosys_validate("ok_hls_bridge_test", &generate_verilog(&uut)).unwrap();
}

#[cfg(feature = "frontpanel")]
#[test]
fn test_ok_hls_bridge_test_synth() {
    let mut uut = OpalKellyHLSBridgeTest::default();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/hls_bridge"));
    test_ok_hls_runtime(target_path!("xem_6010/hls_bridge/top.bit")).unwrap();
}

#[cfg(feature = "frontpanel")]
fn test_ok_hls_runtime(bit_name: &str) -> Result<(), OkError> {
    use std::time::Instant;
    let hnd = ok_test_prelude(bit_name)?;
    let config = OKHLSBridgeAddressConfig::default();
    let uut = OpalKellyHLSBridgeTest::default();
    println!("{:?}", uut.port_bridge.ports());
    for iter in 0..100 {
        ping_bridge(&hnd, &config, 0x67 + iter)?;
        let to_send = (0..256).map(|_| rand::random::<u16>()).collect::<Vec<_>>();
        // Send a set of data elements
        write_data_to_address(&hnd, &config, 0, &to_send)?;
        let ret = read_data_from_address(&hnd, &config, 1, to_send.len())?;
        for (ndx, val) in ret.iter().enumerate() {
            assert_eq!(*val, to_send[ndx].wrapping_shl(1))
        }
        println!("Iteration {} Passed ", iter);
    }
    ping_bridge(&hnd, &config, 0x67)?;
    // Enable streaming on endpoint 2
    enable_streaming(&hnd, &config, 0x02)?;
    // Read 1 MB of data
    let mut t_data = vec![];
    let now = Instant::now();
    while t_data.len() < 1024 * 1024 * 32 {
        let mut data = stream_read(&hnd, &config, 1024 * 4096)?;
        t_data.append(&mut data);
    }
    let elapsed = Instant::now() - now;
    // Stop the streaming
    disable_streaming(&hnd, &config)?;
    let mut dregs = drain_stream(&hnd, &config)?;
    t_data.append(&mut dregs);
    println!("{} microseconds", elapsed.as_micros());
    println!(
        "{} bits per second",
        t_data.len() as f32 * 16.0 / (elapsed.as_micros() as f32 * 1e-6)
    );
    for (ndx, el) in t_data.iter().enumerate() {
        assert_eq!(*el, ndx as u16)
    }
    println!("Data seems correct");
    Ok(())
}
