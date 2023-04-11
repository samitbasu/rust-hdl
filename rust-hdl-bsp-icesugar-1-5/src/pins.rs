use rust_hdl::core::prelude::*;

pub const CLOCK_SPEED_12MHZ: u64 = 12_000_000;

pub fn clock() -> Signal<In, Clock> {
    let mut x = Signal::<In, _>::default() ;
    x.add_location(0, "35");
    x.connect();
    x
}

