use rust_hdl::core::prelude::*;

pub const CLOCK_SPEED_12MHZ: u64 = 12_000_000;

pub fn clock() -> Signal<In, Clock> {
    let mut x = Signal::<In, _>::default() ;
    x.add_location(0, "35");
    x.connect();
    x
}

pub fn rgb_leds() -> Signal<Out, Bits<3>> {
    let mut x = Signal::<Out, _>::default();
    for (ndx, uname) in ["39", "40", "41"]
        .iter()
        .enumerate()
    {
        x.add_location(ndx, uname);
    }
    x
}

pub fn red_led() -> Signal<Out, Bit> {
    let mut x = Signal::<Out, _>::default();
    x.add_location(0, "40");
    x.connect();
    x
}

pub fn blue_led() -> Signal<Out, Bit> {
    let mut x = Signal::<Out, _>::default();
    x.add_location(0, "39");
    x.connect();
    x
}

pub fn green_led() -> Signal<Out, Bit> {
    let mut x = Signal::<Out, _>::default();
    x.add_location(0, "41");
    x.connect();
    x
}

