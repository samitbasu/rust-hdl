use crate::bom::Manufacturer;
use crate::circuit::PartDetails;
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, PinKind, PinLocation, EdgeLocation};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::pin;
use crate::glyph::{Glyph, make_ic_body};


pub fn make_ads868x(part_number: &str) -> PartDetails {
    assert!(part_number.starts_with("ADS868"));
    assert!(part_number.ends_with("IPW"));
    let pins = vec![
        pin!("DGND", PowerReturn, 300, South),
        pin!("AVDD", PowerSink, -200, North),
        pin!("AGND", PowerReturn, -200, South),
        pin!("REFIO", Passive, 0, West),
        pin!("REFGND", PowerReturn, -800, West),
        pin!("REFCAP", Passive, -300, West),
        pin!("AIN_P", Passive, 800, West),
        pin!("AIN_GND", Passive, 400, West),
        pin!("~RST", InputInverted, -900, East),
        pin!("SDI", Input, -700, East),
        pin!("CONVST/~CS", InputInverted, -500, East),
        pin!("SCLK", Input, -300, East),
        pin!("SDO-0", Output, -100, East),
        pin!("ALARM/SDO-1/GPO", Output, 400, East),
        pin!("RVS", Output, 700, East),
        pin!("DVDD", PowerSink, 300, North),
    ];
    PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer {
            name: "TI".to_string(),
            part_number: part_number.into(),
        },
        description: "16-bit high-speed single supply SAR ADC".to_string(),
        comment: "".to_string(),
        hide_pin_designators: false,
        pins: pin_list(pins),
        outline: vec![make_ic_body(-800, -1400, 900, 1200)],
        suppliers: vec![],
        designator: Designator {
            kind: DesignatorKind::IntegratedCircuit,
            index: None,
        },
        size: SizeCode::TSSOP(16),
    }
}
