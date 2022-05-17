use crate::test_common::tools::ok_test_prelude;
use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};
use std::thread::sleep;
use std::time::{Duration, Instant};


#[cfg(test)]
pub fn test_opalkelly_ddr_stress_runtime(bit_file: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_file)?;
    hnd.reset_firmware(0);
    sleep(Duration::from_millis(100));
    hnd.set_wire_in(1, 1);
    hnd.update_wire_ins();
    // Read the data in 256*2 = 512 byte blocks
    let mut counter = 0;
    for _ in 0..32 {
        let mut data = vec![0_u8; 1024 * 1024];
        let now = Instant::now();
        hnd.read_from_block_pipe_out(0xA0, 256, &mut data).unwrap();
        let elapsed = (Instant::now() - now).as_micros();
        println!(
            "Download rate is {} mbps",
            (data.len() as f32 * 8.0) / (elapsed as f32 * 1e-6) / 1e6
        );
        let data_shorts = make_u16_buffer(&data);
        let mut data_words = vec![];
        for i in 0..data_shorts.len() / 2 {
            let lo_word = data_shorts[2 * i] as u32;
            let hi_word = data_shorts[2 * i + 1] as u32;
            data_words.push((hi_word << 16) | lo_word);
        }
        for val in data_words {
            assert_eq!(((counter as u128) & 0xFFFFFFFF_u128) as u32, val);
            counter += 1;
        }
    }
    hnd.set_wire_in(1, 0);
    hnd.update_wire_ins();
    hnd.close();
    Ok(())
}
