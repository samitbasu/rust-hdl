mod direction;
mod synth;
mod clock;
mod constant;
mod atom;
mod signal;
mod logic;
mod block;
mod visitor;
mod visitor_mut;
mod scoped_visitor;
mod dff;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
