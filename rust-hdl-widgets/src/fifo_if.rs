use rust_hdl_core::prelude::*;

#[derive(LogicInterface)]
pub struct FIFOReadIF<D: Synth> {
    pub read: Signal<In, Bit>,
    pub data_out: Signal<Out, D>,
    pub empty: Signal<Out, Bit>,
    pub almost_empty: Signal<Out, Bit>,
    pub underflow: Signal<Out, Bit>,
}

impl<D: Synth> Default for FIFOReadIF<D> {
    fn default() -> Self {
        Self {
            read: Default::default(),
            data_out: Default::default(),
            empty: Default::default(),
            almost_empty: Default::default(),
            underflow: Default::default(),
        }
    }
}

#[derive(LogicInterface)]
pub struct FIFOWriteIF<D: Synth> {
    pub write: Signal<In, Bit>,
    pub data_in: Signal<In, D>,
    pub full: Signal<Out, Bit>,
    pub almost_full: Signal<Out, Bit>,
    pub overflow: Signal<Out, Bit>,
}

impl<D: Synth> Default for FIFOWriteIF<D> {
    fn default() -> Self {
        Self {
            write: Default::default(),
            data_in: Default::default(),
            full: Default::default(),
            almost_full: Default::default(),
            overflow: Default::default(),
        }
    }
}
