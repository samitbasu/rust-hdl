use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;
#[cfg(feature = "frontpanel")]
use {
    crate::test_common::tools::ok_test_prelude,
    rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError, OkHandle},
    std::thread::sleep
};

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
        self.data_fifo.clock.next = self.sys_clock.val();
        // Wire the MOSI port to the input of the data_fifo
        self.data_fifo.data_in.next = self.mosi_port.port_out.val() << 1_usize;
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

#[cfg(feature = "frontpanel")]
fn mk_u8(dat: &[u16]) -> Vec<u8> {
    let mut ret = vec![0_u8; dat.len() * 2];
    for (ndx, el) in dat.iter().enumerate() {
        ret[2 * ndx] = (el & 0xFF) as u8;
        ret[2 * ndx + 1] = ((el & 0xFF00) >> 8) as u8;
    }
    ret
}

#[cfg(feature = "frontpanel")]
fn send_ping(hnd: &OkHandle, id: u8) -> Result<(), OkError> {
    hnd.write_to_pipe_in(0x80, &mk_u8(&[0x0100 | (id as u16)]))
}

#[cfg(feature = "frontpanel")]
fn read_ping(hnd: &OkHandle) -> Result<u16, OkError> {
    let mut data = [0x0_u8; 2];
    hnd.read_from_pipe_out(0xA0, &mut data)?;
    Ok(make_u16_buffer(&data)[0])
}

#[cfg(feature = "frontpanel")]
fn write_array(hnd: &OkHandle, address: u8, data: &[u16]) -> Result<(), OkError> {
    let mut msg = vec![0_u16; data.len() + 2];
    msg[0] = 0x0300 | (address as u16);
    msg[1] = data.len() as u16;
    for (ndx, el) in data.iter().enumerate() {
        msg[ndx + 2] = *el;
    }
    // Send the message
    hnd.write_to_pipe_in(0x80, &mk_u8(&msg))
}

#[cfg(feature = "frontpanel")]
fn read_array(hnd: &OkHandle, address: u8, len: usize) -> Result<Vec<u16>, OkError> {
    let mut msg = vec![0_u16; 2];
    msg[0] = 0x0200 | (address as u16);
    msg[1] = len as u16;
    hnd.write_to_pipe_in(0x80, &mk_u8(&msg))?;
    let mut data = vec![0x0_u8; len * 2];
    hnd.read_from_pipe_out(0xA0, &mut data)?;
    Ok(make_u16_buffer(&data))
}

#[cfg(feature = "frontpanel")]
pub fn test_opalkelly_soc_hello(bit_name: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_name)?;
    for iter in 0..100 {
        println!("Iteration {}", iter);
        send_ping(&hnd, 0x67)?;
        let j = read_ping(&hnd)?;
        assert_eq!(j, 0x167);
        //let to_send = [0xDEAD_u16, 0xBEEF, 0xCAFE, 0xBABE];
        let to_send = (0..256).map(|_| rand::random::<u16>()).collect::<Vec<_>>();
        // Send a set of data elements
        write_array(&hnd, 0, &to_send)?;
        sleep(std::time::Duration::from_millis(100));
        // Read them back
        let ret = read_array(&hnd, 1, to_send.len())?;
        for (ndx, val) in ret.iter().enumerate() {
            assert_eq!(*val, to_send[ndx].wrapping_shl(1))
        }
    }
    hnd.close();
    Ok(())
}
