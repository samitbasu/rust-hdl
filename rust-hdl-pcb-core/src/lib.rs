pub mod bom;
pub mod capacitors;
pub mod circuit;
pub mod designator;
pub mod diode;
pub mod epin;
pub mod glyph;
pub mod inductors;
pub mod port;
pub mod prelude;
pub mod resistors;
pub mod schematic_layout;
pub mod smd;
pub mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
