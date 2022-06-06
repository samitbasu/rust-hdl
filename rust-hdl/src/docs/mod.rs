use embed_doc_image::embed_doc_image;

///
/// ## Blinky!
///
///The definitive example in FPGA firmware land is a simple LED blinker.  This typically
///involves a clock that is fed to the FPGA with a pre-defined frequency, and an output
///signal that can control an LED.  Because we don't know what FPGA we are using, we will
///do this in simulation first.  We want a blink that is 250 msec long every second, and
/// our clock speed is (a comically slow) 10kHz.  Here is a minimal working Blinky! example:
///
///```rust
///use std::time::Duration;
///use rust_hdl::core::prelude::*;
///use rust_hdl::widgets::prelude::*;
///
///const CLOCK_SPEED_HZ : u64 = 10_000;
///
///
///#[derive(LogicBlock)]
///struct Blinky {
///    pub clock: Signal<In, Clock>,
///    pulser: Pulser,
///    pub led: Signal<Out, Bit>,
///}
///
///impl Default for Blinky {
///   fn default() -> Self {
///       Self {
///         clock: Default::default(),
///         pulser: Pulser::new(CLOCK_SPEED_HZ, 1.0, Duration::from_millis(250)),
///         led: Default::default(),
///       }
///    }
///}
///
///impl Logic for Blinky {
///    #[hdl_gen]
///    fn update(&mut self) {
///       self.pulser.clock.next = self.clock.val();
///       self.pulser.enable.next = true.into();
///       self.led.next = self.pulser.pulse.val();
///    }
///}
///
///let mut sim = simple_sim!(Blinky, clock, CLOCK_SPEED_HZ, ep, {
///    let mut x = ep.init()?;
///    wait_clock_cycles!(ep, clock, x, 4*CLOCK_SPEED_HZ);
///    ep.done(x)
///});
///
///let mut uut = Blinky::default();
///uut.connect_all();
///sim.run_to_file(Box::new(uut), 5*SIMULATION_TIME_ONE_SECOND, "blinky.vcd").unwrap();
///```
///Running the above (a release run is highly recommended) will generate a `vcd` file (which is
/// a trace file for FPGAs and hardware in general).  You can open this using e.g., `gtkwave`.
/// Here you will see (with some manipulation of the UI)
///
/// ![Full Simulation Time][full_sim_time]
///
/// And a zoom in shows the 250 msec pulse.
///
/// ![Pulse detail][pulse_detail]
///
#[embed_doc_image("full_sim_time", "images/blinky_all.png")]
#[embed_doc_image("pulse_detail", "images/blinky_pulse.png")]
pub struct BlinkyExample;
