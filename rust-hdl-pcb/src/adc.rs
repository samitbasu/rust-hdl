use crate::circuit::PartDetails;
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::epin::{EPin, PinKind};
use crate::utils::pin_list;

pub fn make_ads868x(part_number: &str) -> PartDetails {
    assert!(part_number.starts_with("ADS868"));
    assert!(part_number.ends_with("IPW"));
    let pins = vec![
        EPin::new("DGND", PinKind::PowerReturn),
        EPin::new("AVDD", PinKind::PowerSink),
        EPin::new("AGND", PinKind::PowerReturn),
        EPin::new("REFIO", PinKind::Passive),
        EPin::new("REFGND", PinKind::PowerReturn),
        EPin::new("REFCAP", PinKind::Passive),
        EPin::new("AIN_P", PinKind::Passive),
        EPin::new("AIN_GND", PinKind::Passive),
        EPin::new("RST", PinKind::InputInverted),
        EPin::new("SDI", PinKind::Input),
        EPin::new("CONVST/CS", PinKind::InputInverted),
        EPin::new("SCLK", PinKind::Input),
        EPin::new("SDO-0", PinKind::Output),
        EPin::new("ALARM/SDO-1/GPO", PinKind::Output),
        EPin::new("RVS", PinKind::Output),
        EPin::new("DVDD", PinKind::PowerSink)
    ];
    PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer { name: "TI".to_string(), part_number: part_number.into() },
        description: "16-bit high-speed single supply SAR ADC".to_string(),
        comment: "".to_string(),
        hide_pin_designators: false,
        pins: pin_list(pins),
        suppliers: vec![],
        designator: Designator { kind: DesignatorKind::IntegratedCircuit, index: None },
        size: SizeCode::TSSOP(16)
    }

}