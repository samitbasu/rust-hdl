use crate::epin::EPin;
use std::collections::BTreeMap;

pub fn drop_char(txt: &str) -> &str {
    let len = txt.len();
    &txt[..(len-1)]
}

pub fn pin_list(pins: Vec<EPin>) -> BTreeMap<u64, EPin> {
    let mut map = BTreeMap::new();
    for pin in pins.into_iter().enumerate() {
        map.insert((pin.0+1) as u64, pin.1);
    }
    map
}
