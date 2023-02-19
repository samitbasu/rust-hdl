use rust_hdl::core::prelude::*;

pub const CLOCK_SPEED_100MHZ: u64 = 100_000_000;

pub fn clock() -> Signal<In, Clock> {
    let mut x = Signal::<In, _>::default();
    x.add_location(0, "P7");
    x.connect();
    x
}

pub fn leds() -> Signal<Out, Bits<8>> {
    let mut x = Signal::<Out, _>::default();
    for (ndx, uname) in ["J11", "K11", "K12", "K14", "L12", "L14", "M12", "N14"]
        .iter()
        .enumerate()
    {
        x.add_location(ndx, uname);
    }
    x
}

pub fn map_alchitry_pin_to_cu_pad(pin: &str) -> &str {
    match pin {
        "A2" => "M1",

        "A3" => "L1",

        "A5" => "J1",

        "A6" => "J3",

        "A8" => "G1",

        "A9" => "G3",

        "A11" => "E1",

        "A12" => "D1",

        "A14" => "C1",

        "A15" => "B1",

        "A17" => "D3",

        "A18" => "C3",

        "A20" => "A1",

        "A21" => "A2",

        "A23" => "A3",

        "A24" => "A4",

        "A27" => "A5",

        "A28" => "C5",

        "A30" => "D5",

        "A31" => "C4",

        "A33" => "D4",

        "A34" => "E4",

        "A36" => "F4",

        "A37" => "F3",

        "A39" => "H4",

        "A40" => "G4",

        "A42" => "H1",

        "A43" => "H3",

        "A45" => "K3",

        "A46" => "K4",

        "A48" => "N1",

        "A49" => "P1",
        _ => {
            panic!("Unknown pin");
        }
    }
}
