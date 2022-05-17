use super::ok_pipe::{PipeIn, PipeOut};
use super::ok_trigger::{TriggerIn, TriggerOut};
use super::prelude::WireIn;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct OKSPIMasterAddressConfig {
    pub pipe_in_address: u8,
    pub pipe_out_address: u8,
    pub trigger_start_address: u8,
    pub trigger_done_address: u8,
    pub wire_bits_address: u8,
}

impl Default for OKSPIMasterAddressConfig {
    fn default() -> Self {
        Self {
            pipe_in_address: 0x80,
            pipe_out_address: 0xA0,
            trigger_start_address: 0x40,
            trigger_done_address: 0x60,
            wire_bits_address: 0x00,
        }
    }
}

#[derive(LogicBlock)]
pub struct OKSPIMaster {
    pub wires: SPIWiresMaster,
    pub ok1: Signal<In, Bits<31>>,
    pub ok2: Signal<Out, Bits<17>>,
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    pipe_in: PipeIn,
    pipe_out: PipeOut,
    bits: WireIn,
    trigger_start: TriggerIn,
    trigger_done: TriggerOut,
    core: SPIMaster<64>,
    data_outbound: DFF<Bits<64>>,
    output_register: DFF<Bits<16>>,
    data_inbound: DFF<Bits<64>>,
}

impl Logic for OKSPIMaster {
    #[hdl_gen]
    fn update(&mut self) {
        // Link the wires
        SPIWiresMaster::link(&mut self.wires, &mut self.core.wires);
        clock_reset!(self, clock, reset, core);
        dff_setup!(
            self,
            clock,
            reset,
            data_outbound,
            output_register,
            data_inbound
        );
        // Feed the clocks
        self.trigger_done.clk.next = self.clock.val();
        self.trigger_start.clk.next = self.clock.val();
        // Prevent latches
        self.output_register.d.next = self.data_inbound.q.val().get_bits::<16>(48_usize);
        // Connect the ok busses
        self.pipe_in.ok1.next = self.ok1.val();
        self.pipe_out.ok1.next = self.ok1.val();
        self.bits.ok1.next = self.ok1.val();
        self.trigger_start.ok1.next = self.ok1.val();
        self.trigger_done.ok1.next = self.ok1.val();
        self.ok2.next =
            self.pipe_in.ok2.val() | self.pipe_out.ok2.val() | self.trigger_done.ok2.val();
        // Pipe in the SPI outbound register
        if self.pipe_in.write.val() {
            self.data_outbound.d.next = (self.data_outbound.q.val() << 16_usize)
                | bit_cast::<64, 16>(self.pipe_in.dataout.val());
        }
        // Pipe from the SPI inbound register
        self.pipe_out.datain.next = self.output_register.q.val();
        if self.pipe_out.read.val() {
            self.data_inbound.d.next = self.data_inbound.q.val() << 16_usize;
        }
        // Trigger to start the transaction - 1 for normal, 2 for continued
        self.core.data_outbound.next = self.data_outbound.q.val();
        self.core.start_send.next = false;
        self.core.continued_transaction.next = false;
        if self.trigger_start.trigger.val() == 1_u32 {
            self.core.start_send.next = true;
            self.core.continued_transaction.next = false;
        } else if self.trigger_start.trigger.val() == 2_u32 {
            self.core.start_send.next = true;
            self.core.continued_transaction.next = true;
        }
        self.core.bits_outbound.next = self.bits.dataout.val();
        // Reflect transaction done back to the caller
        self.trigger_done.trigger.next = 0_u32.into();
        if self.core.transfer_done.val() {
            self.data_inbound.d.next = self.core.data_inbound.val();
            self.trigger_done.trigger.next = 1_u32.into();
        }
    }
}

impl OKSPIMaster {
    pub fn new(config: OKSPIMasterAddressConfig, spi_config: SPIConfig) -> Self {
        assert_eq!(spi_config.clock_speed, 48_000_000);
        Self {
            wires: Default::default(),
            ok1: Default::default(),
            ok2: Default::default(),
            clock: Default::default(),
            pipe_in: PipeIn::new(config.pipe_in_address),
            pipe_out: PipeOut::new(config.pipe_out_address),
            bits: WireIn::new(config.wire_bits_address),
            trigger_start: TriggerIn::new(config.trigger_start_address),
            trigger_done: TriggerOut::new(config.trigger_done_address),
            core: SPIMaster::new(spi_config),
            data_outbound: Default::default(),
            output_register: Default::default(),
            data_inbound: Default::default(),
            reset: Default::default(),
        }
    }
}

#[test]
fn test_ok_spi_master_synthesizes() {
    let spi_config = SPIConfig {
        clock_speed: 48_000_000,
        cs_off: true,
        mosi_off: true,
        speed_hz: 1_000_000,
        cpha: true,
        cpol: true,
    };
    let mut uut = OKSPIMaster::new(Default::default(), spi_config);
    uut.connect_all();
    yosys_validate("ok_spi_synth", &generate_verilog(&uut)).unwrap();
}

#[test]
fn test_ok_spi_master_works() {
    #[derive(LogicBlock)]
    pub struct TopOK {
        wires: SPIWiresMaster,
        ok1: Signal<In, Bits<31>>,
        ok2: Signal<Out, Bits<17>>,
        clock: Signal<In, Clock>,
        reset: Signal<In, Reset>,
        core: OKSPIMaster,
        slave: SPISlave<64>,
    }

    impl Logic for TopOK {
        #[hdl_gen]
        fn update(&mut self) {
            SPIWiresMaster::link(&mut self.wires, &mut self.core.wires);
            self.core.ok1.next = self.ok1.val();
            self.ok2.next = self.core.ok2.val();
            clock_reset!(self, clock, reset, core, slave);
            SPIWiresMaster::join(&mut self.wires, &mut self.slave.wires);
        }
    }

    impl TopOK {
        fn new() -> TopOK {
            let spi_config = SPIConfig {
                clock_speed: 48_000_000,
                cs_off: true,
                mosi_off: true,
                speed_hz: 1_000_000,
                cpha: true,
                cpol: true,
            };
            Self {
                wires: Default::default(),
                ok1: Default::default(),
                ok2: Default::default(),
                clock: Default::default(),
                reset: Default::default(),
                core: OKSPIMaster::new(Default::default(), spi_config),
                slave: SPISlave::new(spi_config),
            }
        }
    }

    let mut uut = TopOK::new();
    uut.slave.data_outbound.connect();
    uut.slave.bits.connect();
    uut.slave.start_send.connect();
    uut.slave.continued_transaction.connect();
    uut.slave.disabled.connect();
    uut.connect_all();
    yosys_validate("ok_spi", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<TopOK>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<TopOK>| {
        let mut x = sim.init()?;
        reset_sim!(sim, clock, reset, x);
        wait_clock_cycle!(sim, clock, x, 20);
        wait_clock_true!(sim, clock, x);
        x.slave.data_outbound.next = 0xcafebabe5ea15e5e_u64.into();
        x.slave.bits.next = 64_u32.into();
        x.slave.start_send.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.slave.start_send.next = false;
        for sample in [0x1234_u16, 0x5678_u16, 0xdead_u16, 0xbeef_u16] {
            x.core.pipe_in.dataout.next = sample.into();
            x.core.pipe_in.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.core.pipe_in.write.next = false;
        }
        wait_clock_cycle!(sim, clock, x);
        sim_assert_eq!(sim, x.core.data_outbound.q.val(), 0x12345678deadbeef_u64, x);
        x.core.bits.dataout.next = 64_u32.into();
        x.core.trigger_start.trigger.next = 1_u32.into();
        wait_clock_cycle!(sim, clock, x);
        x.core.trigger_start.trigger.next = 0_u32.into();
        x = sim.watch(|x| x.slave.transfer_done.val(), x)?;
        sim_assert_eq!(sim, x.slave.data_inbound.val(), 0x12345678deadbeef_u64, x);
        for sample in [0xcafe_u16, 0xbabe_u16, 0x5ea1_u16, 0x5e5e_u16] {
            x.core.pipe_out.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            sim_assert!(sim, x.core.pipe_out.datain.val() == sample, x);
            x.core.pipe_out.read.next = false;
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create("/tmp/ok_spi.vcd").unwrap(),
    )
    .unwrap()
}
