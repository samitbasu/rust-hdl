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
mod simulate;
mod has_changed;
mod update_all;
mod check_connected;
mod shortbitvec;
mod bits;
mod bitvec;

#[cfg(test)]
mod tests {
    use crate::signal::Signal;
    use crate::direction::{In, Out};
    use crate::bits::{Bit, Bits};
    use crate::clock::Clock;
    use crate::constant::Constant;
    use crate::logic::Logic;
    use crate::block::Block;
    use crate::visitor::Visitor;
    use crate::visitor_mut::VisitorMut;
    use crate::scoped_visitor::ScopedVisitor;
    use crate::simulate::simulate;
    use crate::dff::DFF;
    use crate::check_connected::check_connected;

    struct Strobe<const N: usize> {
        pub enable: Signal<In, Bit>,
        pub strobe: Signal<Out, Bit>,
        pub clock: Signal<In, Clock>,
        pub strobe_incr: Constant<Bits<N>>,
        counter: DFF<Bits<N>>,
    }

    impl<const N: usize> Strobe<N> {
        pub fn new() -> Self {
            let mut ret = Self {
                enable: Signal::<In, Bit>::new(),
                strobe: Signal::<Out, Bit>::new_with_default(false),
                clock: Signal::<In, Clock>::new(),
                strobe_incr: Constant::new(1_usize.into()),
                counter: DFF::new(0_usize.into()),
            };
            ret.strobe.connect();
            ret.counter.clk.connect();
            ret.counter.d.connect();
            ret
        }
    }

    impl<const N: usize> Logic for Strobe<N> {
        fn update(&mut self) {
            self.counter.clk.next = self.clock.val;
            if self.enable.val {
                self.counter.d.next = self.counter.q.val + 1; //self.strobe_incr.val;
            }
            self.strobe.next = self.enable.val & !self.counter.q.val.any();
        }
    }

    impl<const N: usize> Block for Strobe<N> {
        #[inline(always)]
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit(self);
            self.enable.accept(visitor);
            self.strobe.accept(visitor);
            self.clock.accept(visitor);
            self.counter.accept(visitor);
        }

        #[inline(always)]
        fn accept_mut(&mut self, visitor: &mut dyn VisitorMut) {
            visitor.visit(self);
            self.enable.accept_mut(visitor);
            self.strobe.accept_mut(visitor);
            self.clock.accept_mut(visitor);
            self.counter.accept_mut(visitor);
        }

        fn accept_scoped(&self, name: &str, visitor: &mut dyn ScopedVisitor) {
            visitor.visit_start_scope(name, self);
            self.enable.accept_scoped("enable", visitor);
            self.strobe.accept_scoped("strobe", visitor);
            self.clock.accept_scoped("clock", visitor);
            self.counter.accept_scoped("counter", visitor);
            visitor.visit_end_scope(name, self);
        }
    }


    #[test]
    fn test_visit_version() {
        let mut uut: Strobe<4> = Strobe::new();
        // Simulate 100 clock cycles
        uut.enable.next = true;
        println!("Starting");
        uut.clock.connect();
        uut.enable.connect();
        check_connected(&uut);
//        let scopes = list_atoms(&uut);
//        println!("{:#?}", scopes);
        let mut strobe_count = 0;
        for clock in 0..100_000_000 {
            uut.clock.next = Clock(clock % 2 == 0);
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            if uut.strobe.val {
                strobe_count += 1;
            }
        }
        assert_eq!(strobe_count, 6_250_000);
    }

}
