use rust_hdl__core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct Accum<const N: usize, const M: usize, const P: usize> {
    pub clock: Signal<In, Clock>,
    pub strobe_in: Signal<In, Bit>,
    pub data_in: Signal<In, Bits<N>>,
    pub strobe_out: Signal<Out, Bit>,
    pub data_out: Signal<Out, Bits<M>>,
    accum: DFF<Bits<M>>,
    counter: DFF<Bits<P>>,
    max_count: Constant<Bits<P>>,
}

impl<const N: usize, const M: usize, const P: usize> Logic for Accum<N, M, P> {
    fn update(&mut self) {
        self.accum.clock.next = self.clock.val();
        self.counter.clock.next = self.clock.val();
        self.strobe_out.next = false;
        self.data_out.next = self.accum.q.val();
        self.accum.d.next = self.accum.q.val();
        if self.strobe_in.val() {
            self.accum.d.next = self.accum.q.val() + bit_cast::<M, N>(self.data_in.val());
            self.counter.d.next = self.counter.q.val() + 1;
        }
        if self.counter.q.val() == self.max_count.val() {
            self.strobe_out.next = true;
            self.counter.d.next = 0.into();
            self.accum.d.next = 0.into();
        }
    }
}

impl<const N: usize, const M: usize, const P: usize> Accum<N, M, P> {
    pub fn new(count: usize) -> Self {
        assert!(P >= clog2(count));
        assert!(M >= N + P);
        Self {
            clock: Default::default(),
            strobe_in: Default::default(),
            data_in: Default::default(),
            strobe_out: Default::default(),
            accum: DFF::default(),
            counter: DFF::default(),
            max_count: Constant::new(count.to_bits()),
            data_out: Default::default(),
        }
    }
}

#[test]
fn test_accum_synthesizes() {
    let _p = TopWrap::new(Accum::<32, 40, 6>::new(50));
}
