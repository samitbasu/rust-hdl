use std::{
    cell::RefCell,
    fmt::{Display, Formatter, LowerHex},
    io::BufWriter,
    rc::Rc,
};

pub use crate::log::{ClockDetails, LogBuilder};
pub use crate::loggable::Loggable;
pub use log::TagID;
pub use logger::Logger;
use rust_hdl::prelude::{freq_hz_to_period_femto, Bits};
pub use rust_hdl_x_macro::Loggable;
use synchronous::Synchronous;

//use synchronous::Synchronous;

//mod bit_iter;
//mod bit_slice;
mod rev_bit_iter;
//mod derive_vcd;
//mod pulser;
//pub mod shot;
//mod spi_controller;
//mod synchronous;
//pub mod basic_tracer;
//pub mod counter;
//pub mod no_trace;
pub mod synchronous;
//pub mod traceable;
//pub mod tracer;
//pub mod tracer_builder;
//mod vcd;
pub mod basic_logger;
pub mod basic_logger_builder;
pub mod log;
pub mod loggable;
pub mod logger;

#[ignore]
#[test]
fn bits_benchmark() {
    let tic = std::time::Instant::now();
    let x = rust_hdl::core::bits::Bits::<65>::from(0x12345678);
    let y = rust_hdl::core::bits::Bits::<65>::from(0x1);
    let mut z = rust_hdl::core::bits::Bits::<65>::from(0x0);
    for i in 0..1000000 {
        let _ = x.get_bit(i % 32);
        let _ = y.get_bit(i % 32);
        z = z + y;
    }
    let toc = std::time::Instant::now();
    println!("Time to run bit benchmark: {:?}", toc - tic);
}

/* #[ignore]
#[test]
fn uint_benchmark() {
    let tic = std::time::Instant::now();
    let x = uint!(0x12345678_U65);
    let y = uint!(0x1_U65);
    let mut z = uint!(0x0_U65);
    for i in 0..1000000 {
        let _ = x.bit(i % 32);
        let _ = y.bit(i % 32);
        z += y;
    }
    let toc = std::time::Instant::now();
    println!("Time to run uint benchmark: {:?}", toc - tic);
}
 */

pub fn single_clock_simulation<S: Synchronous>(
    mut logger: impl Logger,
    obj: S,
    period_in_fs: u64,
    num_cycles: u64,
    mut test_bench: impl FnMut(u64, S::Output) -> S::Input,
) -> bool {
    let mut state = S::State::default();
    let mut new_state = S::State::default();
    let mut output = S::Output::default();
    for cycle in 0..num_cycles {
        logger.set_time_in_fs(cycle * period_in_fs);
        let mut input = test_bench(cycle, output);
        loop {
            (output, new_state) = obj.compute(&mut logger, input, state);
            let next_input = test_bench(cycle, output);
            if next_input == input {
                break;
            }
            input = next_input;
        }
        state = new_state;
    }
    true
}

/*
For combinatorial test logic...

Suppose we have a fifo.
inputs:
<write>
<read>
outputs:
<full>
<empty>

Then there is a logic connection between outputs and inputs.
As such, we must first know the outputs before we can decide
what the input should be.

If the design contains combinatorial paths from input to
output, then we can end up in oscillations.

let mut prev_output;
let prev_input = test_bench(clock, prev_output);
let (output, new_state) = compute(logger, prev_input, state);
let next_input = test_bench(clock, output);
while next_input != prev_input {
    let (output, new_state) = compute(logger, next_input, state);
}

*/
