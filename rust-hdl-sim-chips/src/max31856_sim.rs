use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
enum MAX31856State {
    Ready,
    GettingCmd,
    ReadCmd,
    WaitSlaveIdle,
    WriteCmd,
    DoWrite,
}


#[derive(LogicBlock)]
pub struct MAX31856Simulator {
    // Slave SPI bus
    pub mosi: Signal<In, Bit>,
    pub mclk: Signal<In, Bit>,
    pub msel: Signal<In, Bit>,
    pub miso: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    // RAM that stores the memory contents
    reg_ram: RAM<Bits<8>, 4>,
    // Used to handle auto conversions
    auto_conversions_enabled: DFF<Bit>,
    auto_conversion_strobe: Strobe<32>,
    auto_conversion_counter: DFF<Bits<19>>,
    // Separate bits out of the SPI message
    cmd: Signal<Local, Bits<8>>,
    reg_index: Signal<Local, Bits<4>>,
    rw_flag: Signal<Local, Bit>,
    // The SPI slave device
    spi_slave: SPISlave<64>,
    // FSM state:
    state: DFF<MAX31856State>,
    reg_write_index: DFF<Bits<4>>,
}

const MAX31856_REG_INITS: [u8; 16] = [0x00, 0x03, 0xFF, 0x7F, 0xC0, 0x7F, 0xFF, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

impl MAX31856Simulator {
    pub fn new(config: SPIConfig) -> Self {
        let reg_ram = MAX31856_REG_INITS.iter().map(|x| Bits::<8>::from(*x)).into();
        Self {
            mosi: Default::default(),
            mclk: Default::default(),
            msel: Default::default(),
            miso: Default::default(),
            clock: Default::default(),
            reg_ram,
            auto_conversions_enabled: Default::default(),
            auto_conversion_strobe: Strobe::new(config.clock_speed, 10.0),
            auto_conversion_counter: Default::default(),
            cmd: Default::default(),
            reg_index: Default::default(),
            rw_flag: Default::default(),
            spi_slave: SPISlave::new(config),
            state: DFF::new(MAX31856State::Ready),
            reg_write_index: Default::default()
        }
    }
}

impl Logic for MAX31856Simulator {
    #[hdl_gen]
    fn update(&mut self) {
        self.spi_slave.mosi.next = self.mosi.val();
        self.spi_slave.mclk.next = self.mclk.val();
    }
}