use std::f64::consts::PI;

use rust_hdl_core::bits::Bits;

pub fn snore<const P: usize>(x: u32) -> Bits::<P> {
    let amp = (f64::exp(f64::sin(((x as f64) - 128.0/2.)*PI/128.0))-0.36787944)*108.0;
    let amp = (amp.max(0.0).min(255.0).floor()/255.0 * (1 << P) as f64) as u8;
    amp.into()
}
