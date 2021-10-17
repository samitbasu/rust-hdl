use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};

#[test]
fn test_opalkelly_xem_6010_synth_pipe() {
    let mut uut = OpalKellyPipeTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "xem_6010_pipe");
}

#[test]
fn test_xem_6010_pipe_in_runtime() -> Result<(), OkError> {
    pipe::test_opalkelly_pipe_in_runtime("xem_6010_pipe/top.bit")
}

#[test]
fn test_opalkelly_xem_6010_synth_pipe_ram() {
    let mut uut = OpalKellyPipeRAMTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "xem_6010_pipe_ram");
}

#[test]
fn test_opalkelly_xem_6010_pipe_ram_runtime() -> Result<(), OkError> {
    pipe::test_opalkelly_pipe_ram_runtime("xem_6010_pipe_ram/top.bit")
}

#[test]
fn test_opalkelly_xem_6010_synth_pipe_fifo() {
    let mut uut = OpalKellyPipeFIFOTest::new::<XEM6010>();
    uut.hi.sig_inout.connect();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "xem_6010_fifo");
}

#[test]
fn test_opalkelly_xem_6010_pipe_fifo_runtime() -> Result<(), OkError> {
    pipe::test_opalkelly_pipe_fifo_runtime("xem_6010_fifo/top.bit")
}

#[test]
fn test_opalkelly_xem_6010_synth_pipe_afifo() {
    let mut uut = OpalKellyPipeAFIFOTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.fast_clock.connect();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "xem_6010_afifo");
}

#[test]
fn test_opalkelly_xem_6010_pipe_afifo_runtime() -> Result<(), OkError> {
    pipe::test_opalkelly_pipe_afifo_runtime("xem_6010_afifo/top.bit")
}

#[test]
fn test_opalkelly_xem_6010_synth_btpipe() {
    let mut uut = OpalKellyBTPipeOutTest::new();
    uut.hi.link_connect_dest();
    uut.fast_clock.connect();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "opalkelly_xem_6010_btpipe");
}

#[test]
fn test_opalkelly_xem_6010_btpipe_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_btpipe/top.bit")?;
    // Read the data in 256*2 = 512 byte blocks
    let mut data = vec![0_u8; 1024 * 128];
    hnd.read_from_block_pipe_out(0xA0, 256, &mut data).unwrap();
    let data_shorts = make_u16_buffer(&data);
    for (ndx, val) in data_shorts.iter().enumerate() {
        assert_eq!(((ndx as u128) & 0xFFFF_u128) as u16, *val);
    }
    Ok(())
}
