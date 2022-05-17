use crate::core::prelude::*;
use crate::hls::bridge::Bridge;
use crate::hls::bus::{SoCBusResponder, SoCPortController};
use crate::hls::prelude::{MISOWidePort, MOSIPort, MOSIWidePort};
use crate::hls::HLSNamedPorts;
use crate::widgets::prelude::*;

#[derive(Debug, Copy, Clone, LogicState, PartialEq)]
enum State {
    Idle,
    Writing,
    Reading,
}

#[derive(LogicBlock)]
pub struct SDRAMControllerTester<const R: usize, const C: usize> {
    pub dram: SDRAMDriver<16>,
    pub upstream: SoCBusResponder<16, 8>,
    local_bridge: Bridge<16, 8, 5>,
    count: MOSIWidePort<32, 16>,
    cmd: MOSIPort<16>,
    write_out: MISOWidePort<32, 16>,
    error_out: MISOWidePort<32, 16>,
    validation_out: MISOWidePort<32, 16>,
    controller: SDRAMBaseController<R, C, 64, 16>,
    lsfr: LFSRSimple,
    entropy_funnel: CrossWidenFIFO<32, 6, 7, 64, 3, 4>,
    output_funnel: CrossNarrowFIFO<64, 3, 4, 32, 6, 7>,
    lsfr_validate: LFSRSimple,
    dram_address: DFF<Bits<32>>,
    error_count: DFF<Bits<32>>,
    validation_count: DFF<Bits<32>>,
    write_count: DFF<Bits<32>>,
    output_pipeline: DFF<Bits<32>>,
    output_avail: DFF<Bit>,
    state: DFF<State>,
    clock: Signal<Local, Clock>,
    reset: Signal<Local, Reset>,
}

impl<const R: usize, const C: usize> SDRAMControllerTester<R, C> {
    pub fn new(cas_delay: u32, timings: MemoryTimings, buffer: OutputBuffer) -> Self {
        Self {
            dram: Default::default(),
            upstream: Default::default(),
            local_bridge: Bridge::new(["count", "cmd", "errors", "valid", "write"]),
            count: Default::default(),
            cmd: Default::default(),
            write_out: Default::default(),
            error_out: Default::default(),
            validation_out: Default::default(),
            controller: SDRAMBaseController::new(cas_delay, timings, buffer),
            lsfr: Default::default(),
            entropy_funnel: CrossWidenFIFO::new(WordOrder::LeastSignificantFirst),
            output_funnel: CrossNarrowFIFO::new(WordOrder::LeastSignificantFirst),
            lsfr_validate: Default::default(),
            dram_address: Default::default(),
            error_count: Default::default(),
            validation_count: Default::default(),
            write_count: Default::default(),
            output_pipeline: Default::default(),
            output_avail: Default::default(),
            state: Default::default(),
            clock: Default::default(),
            reset: Default::default(),
        }
    }
}

impl<const R: usize, const C: usize> HLSNamedPorts for SDRAMControllerTester<R, C> {
    fn ports(&self) -> Vec<String> {
        self.local_bridge.ports()
    }
}

impl<const R: usize, const C: usize> Logic for SDRAMControllerTester<R, C> {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.local_bridge.upstream);
        SDRAMDriver::<16>::link(&mut self.dram, &mut self.controller.sdram);
        self.clock.next = self.upstream.clock.val();
        self.reset.next = self.upstream.reset.val();
        clock_reset!(self, clock, reset, controller, lsfr, lsfr_validate);
        dff_setup!(
            self,
            clock,
            reset,
            dram_address,
            error_count,
            validation_count,
            write_count,
            output_pipeline,
            output_avail,
            state
        );
        self.controller.clock.next = self.upstream.clock.val();
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[0], &mut self.count.bus);
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[1], &mut self.cmd.bus);
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[2], &mut self.error_out.bus);
        SoCPortController::<16>::join(
            &mut self.local_bridge.nodes[3],
            &mut self.validation_out.bus,
        );
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[4], &mut self.write_out.bus);
        self.cmd.ready.next = false;
        self.controller.data_in.next = self.entropy_funnel.data_out.val();
        self.entropy_funnel.data_in.next = self.lsfr.num.val();
        self.lsfr.strobe.next = !self.entropy_funnel.full.val();
        self.entropy_funnel.write.next = !self.entropy_funnel.full.val();
        self.entropy_funnel.write_clock.next = self.upstream.clock.val();
        self.entropy_funnel.write_reset.next = self.reset.val();
        self.entropy_funnel.read_clock.next = self.upstream.clock.val();
        self.entropy_funnel.read_reset.next = self.reset.val();
        self.controller.data_in.next = self.entropy_funnel.data_out.val();
        self.controller.cmd_address.next = self.dram_address.q.val();
        self.controller.write_not_read.next = false;
        self.controller.cmd_strobe.next = false;
        self.entropy_funnel.read.next = false;
        self.output_funnel.data_in.next = self.controller.data_out.val();
        self.output_funnel.write.next = self.controller.data_valid.val();
        self.output_funnel.write_clock.next = self.upstream.clock.val();
        self.output_funnel.write_reset.next = self.reset.val();
        self.output_funnel.read_clock.next = self.upstream.clock.val();
        self.output_funnel.read_reset.next = self.reset.val();
        self.error_out.strobe_in.next = false;
        self.validation_out.strobe_in.next = false;
        self.write_out.strobe_in.next = false;
        match self.state.q.val() {
            State::Idle => {
                self.cmd.ready.next = true;
                if self.cmd.strobe_out.val() {
                    self.error_count.d.next = 0_usize.into();
                    self.dram_address.d.next = 0_usize.into();
                    self.state.d.next = State::Writing;
                    self.validation_count.d.next = 0_usize.into();
                    self.write_count.d.next = 0_usize.into();
                }
            }
            State::Writing => {
                if self.write_count.q.val() >= self.count.port_out.val() {
                    if !self.controller.busy.val() {
                        self.dram_address.d.next = 0_usize.into();
                        self.state.d.next = State::Reading;
                        self.write_out.strobe_in.next = true;
                    }
                } else if !self.controller.busy.val() & !self.entropy_funnel.empty.val() {
                    self.controller.write_not_read.next = true;
                    self.controller.cmd_strobe.next = true;
                    self.dram_address.d.next = self.dram_address.q.val() + 4_usize;
                    self.entropy_funnel.read.next = true;
                    self.write_count.d.next = self.write_count.q.val() + 4_usize;
                }
            }
            State::Reading => {
                if self.dram_address.q.val() >= self.count.port_out.val() {
                    if self.validation_count.q.val() >= self.count.port_out.val() {
                        self.state.d.next = State::Idle;
                        self.error_out.strobe_in.next = true;
                        self.validation_out.strobe_in.next = true;
                    }
                } else if !self.controller.busy.val() & !self.output_funnel.full.val() {
                    self.controller.write_not_read.next = false;
                    self.controller.cmd_strobe.next = true;
                    self.dram_address.d.next = self.dram_address.q.val() + 4_usize;
                }
            }
            _ => {
                self.state.d.next = State::Idle;
            }
        }
        // Do the validation piece - we need a pipeline register
        // to meet timing.
        // Feed the output_pipeline register
        self.output_funnel.read.next = false;
        if !self.output_avail.q.val() & !self.output_funnel.empty.val() {
            self.output_pipeline.d.next = self.output_funnel.data_out.val();
            self.output_funnel.read.next = true;
            self.output_avail.d.next = true;
        }
        // Validate the output_pipeline value
        self.lsfr_validate.strobe.next = false;
        if self.output_avail.q.val() {
            if self.output_pipeline.q.val() != self.lsfr_validate.num.val() {
                self.error_count.d.next = self.error_count.q.val() + 2_usize;
            }
            self.output_avail.d.next = false;
            self.lsfr_validate.strobe.next = true;
            self.validation_count.d.next = self.validation_count.q.val() + 2_usize;
        }
        self.error_out.port_in.next = self.error_count.q.val();
        self.validation_out.port_in.next = self.validation_count.q.val();
        self.write_out.port_in.next = self.write_count.q.val();
    }
}

#[test]
fn test_sdram_controller_tester_synthesizes() {
    let mut uut = SDRAMControllerTester::<6, 4>::new(
        3,
        MemoryTimings::fast_boot_sim(100e6),
        OutputBuffer::DelayOne,
    );
    uut.connect_all();
    yosys_validate("sdram_controller_tester_hls", &generate_verilog(&uut)).unwrap();
}
