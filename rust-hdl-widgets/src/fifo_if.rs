use rust_hdl_core::prelude::*;

#[derive(LogicInterface)]
pub struct FIFOReadIF<D: Synth, T: Domain> {
    pub read: Signal<In, Bit, T>,
    pub data_out: Signal<Out, D, T>,
    pub empty: Signal<Out, Bit, T>,
    pub almost_empty: Signal<Out, Bit, T>,
    pub underflow: Signal<Out, Bit, T>,
}

impl<D: Synth, T: Domain> Default for FIFOReadIF<D, T> {
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
pub struct FIFOWriteIF<D: Synth, T: Domain> {
    pub write: Signal<In, Bit, T>,
    pub data_in: Signal<In, D, T>,
    pub full: Signal<Out, Bit, T>,
    pub almost_full: Signal<Out, Bit, T>,
    pub overflow: Signal<Out, Bit, T>,
}

impl<D: Synth, T: Domain> Default for FIFOWriteIF<D, T> {
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
