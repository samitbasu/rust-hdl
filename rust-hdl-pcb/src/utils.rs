pub fn drop_char(txt: &str) -> &str {
    let len = txt.len();
    &txt[..(len-1)]
}
