use rust_hdl_core::prelude::*;

pub fn xem_6010_leds() -> Signal<Out, Bits<8>, Async> {
    let mut x = Signal::default();
    for (ndx, name) in [
        "Y17", "AB17", "AA14", "AB14", "AA16", "AB16", "AA10", "AB10",
    ]
        .iter()
        .enumerate()
    {
        x.add_location(ndx, name);
        x.add_signal_type(ndx, SignalType::LowVoltageCMOS_3v3);
    }
    x
}
