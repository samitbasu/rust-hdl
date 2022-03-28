use crate::core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct IODelay<T: Synth> {
    pub a: Signal<In, T>,
    pub z: Signal<Out, T>,
    _delay: u8,
}

impl<T: Synth> IODelay<T> {
    pub fn new(delay: u8) -> Self {
        Self {
            a: Default::default(),
            z: Default::default(),
            _delay: delay,
        }
    }
}

fn wrapper_once(delay: u8) -> String {
    format!(
        r##"
defparam udel_dataini0.DEL_VALUE = {delay} ;
defparam udel_dataini0.DEL_MODE = "USER_DEFINED" ;
DELAYG udel_dataini0 (.A(buf_dataini0), .Z(dataini_t0));
    "##,
        delay = delay
    )
}

fn wrapper_multiple(count: usize, delay: u8) -> String {
    (0..count)
        .map(|x| {
            format!(
                r##"
defparam udel_datain_{x}.DEL_VALUE = {delay} ;
defparam udel_datain_{x}.DEL_MODE = "USER_DEFINED" ;
DELAYG udel_datain_{x} (.A(a[{x}]), .Z(z[{x}]));
    "##,
                x = x,
                delay = delay
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl<T: Synth> Logic for IODelay<T> {
    fn update(&mut self) {
        self.z.next = self.a.val();
    }
    fn connect(&mut self) {
        self.z.connect();
    }
    fn hdl(&self) -> Verilog {
        Verilog::Wrapper(Wrapper {
            code: if T::BITS == 1 {
                wrapper_once(self._delay)
            } else {
                wrapper_multiple(T::BITS, self._delay)
            },
            cores: r##"
(* blackbox *)
module DELAYG(input A, output Z);
parameter DEL_MODE = "USER_DEFINED";
parameter DEL_VALUE = 0;
endmodule
            "##
            .to_string(),
        })
    }
}

#[test]
fn test_iodelay_synthesizes() {
    let mut uut = TopWrap::new(IODelay::<Bits<8>>::new(25));
    uut.uut.a.connect();
    uut.connect_all();
    println!("{}", generate_verilog(&uut));
    yosys_validate("iodelay", &generate_verilog(&uut)).unwrap();
}
