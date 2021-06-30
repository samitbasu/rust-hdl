pub trait Atom {
    fn bits(&self) -> usize;
    fn connected(&self) -> bool;
    fn changed(&self) -> bool;
}
