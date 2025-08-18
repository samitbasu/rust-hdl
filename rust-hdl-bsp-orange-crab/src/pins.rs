use rust_hdl::core::prelude::*;

/// Represents the clock speed for the FPGA: 48MHz.
pub const CLOCK_SPEED: u64 = 48_000_000;

/// Represents the GPIO pins available on the Orange Crab board.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pin {
    /// Reset
    Rst,
    /// Supply voltage power 3.3V
    P3V3,
    /// Reference voltage
    Aref,
    /// Ground
    Gnd0,
    /// GPIO A0
    GpioA0,
    /// GPIO A1
    GpioA1,
    /// GPIO A2
    GpioA2,
    /// GPIO A3
    GpioA3,
    /// GPIO A4
    GpioA4,
    /// GPIO A4
    GpioA5,
    /// SCK
    Sck,
    /// MOSI
    Mosi,
    /// MISO
    Miso,
    /// GPIO 0
    Gpio0,
    /// GPIO 1
    Gpio1,
    /// Ground
    Gnd1,
    /// Battery voltage
    Vbat,
    /// Enable
    Enable,
    /// USB voltage
    Vusb,
    /// GPIO 13
    Gpio13,
    /// GPIO 12
    Gpio12,
    /// GPIO 11
    Gpio11,
    /// GPIO 10
    Gpio10,
    /// GPIO 9
    Gpio9,
    /// GPIO 6
    Gpio6,
    /// GPIO 5
    Gpio5,
    /// SCL
    Scl,
    /// SDA
    Sda,
    /// System clock
    Clock,
    /// RGB LED: red
    LedRed,
    /// RGB LED: green
    LedGreen,
    /// RGB LED: blue
    LedBlue,
    /// Button
    Btn,
    /// Reset (inverse)
    Rstn,
    /// USB positive
    UsbP,
    /// USB negative
    UsbN,
    /// USB pu
    UsbPu,
    /// QSPI Chip Select (inverse)
    QspiCsn,
    /// QSPI Clock
    QspiClock,
    /// QSPI DQ[0]
    QspiDq0,
    /// QSPI DQ[1]
    QspiDq1,
    /// QSPI DQ[2]
    QspiDq2,
    /// QSPI DQ[3]
    QspiDq3,
    /// SD DAT[0]
    SdDat0,
    /// SD DAT[1]
    SdDat1,
    /// SD DAT[2]
    SdDat2,
    /// SD DAT[3]
    SdDat3,
    /// SD Command
    SdCmd,
    /// SD Clock
    SdClk,
    /// SD Card Detect
    SdCd,
    /// Analog Mux[0]
    AnaMux0,
    /// Analog Mux[1]
    AnaMux1,
    /// Analog Mux[2]
    AnaMux2,
    /// Analog Mux[3]
    AnaMux3,
    /// Analog Sense Low
    AnaSenseL,
    /// Analog Sense High
    AnaSenseH,
    /// Analog Control[0]
    AnaCtrl0,
    /// Analog Control[1]
    AnaCtrl1,
}

impl Pin {
    /// Creates a new [Pin].
    pub const fn new() -> Self {
        Self::Rst
    }

    /// Gets the pin name.
    pub const fn pin(&self) -> &'static str {
        match self {
            Self::Rst => "PROGRAM",
            Self::P3V3 => "3V3",
            Self::Aref => "AREF",
            Self::Gnd0 => "GND",
            Self::GpioA0 => "PR38B",
            Self::GpioA1 => "PR38D",
            Self::GpioA2 => "PR38C",
            Self::GpioA3 => "PR26A",
            Self::GpioA4 => "PR26B",
            Self::GpioA5 => "PB13B",
            Self::Sck => "PR13A",
            Self::Mosi => "PB4B",
            Self::Miso => "PB4A",
            Self::Gpio0 => "PB6A",
            Self::Gpio1 => "PB6B",
            Self::Gnd1 => "GND",
            Self::Vbat => "VBAT",
            Self::Enable => "EN",
            Self::Vusb => "VUSB",
            Self::Gpio13 => "PR35A",
            Self::Gpio12 => "PR29D",
            Self::Gpio11 => "PT32B",
            Self::Gpio10 => "PT67A",
            Self::Gpio9 => "PT67B",
            Self::Gpio6 => "PT33A",
            Self::Gpio5 => "PT29A",
            Self::Scl => "PT33B",
            Self::Sda => "PT29B",
            Self::Clock => "clk48",
            Self::LedRed => "rgb_led0_r",
            Self::LedGreen => "rgb_led0_g",
            Self::LedBlue => "rgb_led0_b",
            Self::Btn => "BTN",
            Self::Rstn => "RSTn",
            Self::UsbP => "USB_p",
            Self::UsbN => "USB_n",
            Self::UsbPu => "USB_pu",
            Self::QspiCsn => "QSPI_CSn",
            Self::QspiClock => "QSPI_CLK",
            Self::QspiDq0 => "QSPI_DQ[0]",
            Self::QspiDq1 => "QSPI_DQ[1]",
            Self::QspiDq2 => "QSPI_DQ[2]",
            Self::QspiDq3 => "QSPI_DQ[3]",
            Self::SdDat0 => "SD_DAT[0]",
            Self::SdDat1 => "SD_DAT[1]",
            Self::SdDat2 => "SD_DAT[2]",
            Self::SdDat3 => "SD_DAT[3]",
            Self::SdCmd => "SD_CMD",
            Self::SdClk => "SD_CLK",
            Self::SdCd => "SD_CD",
            Self::AnaMux0 => "ANA_MUX[0]",
            Self::AnaMux1 => "ANA_MUX[1]",
            Self::AnaMux2 => "ANA_MUX[2]",
            Self::AnaMux3 => "ANA_MUX[3]",
            Self::AnaSenseL => "ANA_SENSE_L",
            Self::AnaSenseH => "ANA_SENSE_H",
            Self::AnaCtrl0 => "ANA_CTRL[0]",
            Self::AnaCtrl1 => "ANA_CTRL[1]",
        }
    }

    /// Gets the pad name.
    pub const fn pad(&self) -> &'static str {
        match self {
            Self::Rst => "T15",
            Self::P3V3 => "3V3",
            Self::Aref => "AREF",
            Self::Gnd0 => "GND",
            Self::GpioA0 => "L4",
            Self::GpioA1 => "N3",
            Self::GpioA2 => "N4",
            Self::GpioA3 => "H4",
            Self::GpioA4 => "G4",
            Self::GpioA5 => "T17",
            Self::Sck => "R17",
            Self::Mosi => "N16",
            Self::Miso => "N15",
            Self::Gpio0 => "N17",
            Self::Gpio1 => "M18",
            Self::Gnd1 => "GND",
            Self::Vbat => "VBAT",
            Self::Enable => "EN",
            Self::Vusb => "VUSB",
            Self::Gpio13 => "J2",
            Self::Gpio12 => "H2",
            Self::Gpio11 => "A8",
            Self::Gpio10 => "B8",
            Self::Gpio9 => "C8",
            Self::Gpio6 => "B9",
            Self::Gpio5 => "B10",
            Self::Scl => "C9",
            Self::Sda => "C10",
            Self::Clock => "A9",
            Self::LedRed => "K4",
            Self::LedGreen => "M3",
            Self::LedBlue => "J3",
            Self::Btn => "J17",
            Self::Rstn => "V17",
            Self::UsbP => "N1",
            Self::UsbN => "M2",
            Self::UsbPu => "N2",
            Self::QspiCsn => "U17",
            Self::QspiClock => "U16",
            Self::QspiDq0 => "U18",
            Self::QspiDq1 => "T18",
            Self::QspiDq2 => "R18",
            Self::QspiDq3 => "N18",
            Self::SdDat0 => "J1",
            Self::SdDat1 => "K3",
            Self::SdDat2 => "L3",
            Self::SdDat3 => "M1",
            Self::SdCmd => "K2",
            Self::SdClk => "K1",
            Self::SdCd => "L1",
            Self::AnaMux0 => "F4",
            Self::AnaMux1 => "F3",
            Self::AnaMux2 => "F2",
            Self::AnaMux3 => "H1",
            Self::AnaSenseL => "G3",
            Self::AnaSenseH => "H3",
            Self::AnaCtrl0 => "G1",
            Self::AnaCtrl1 => "F1",
        }
    }

    /// Gets the [Signal] for the clock.
    pub fn clock() -> Signal<In, Clock> {
        let mut x = Signal::<In, _>::default();
        x.add_location(0, Self::Clock.pad());
        x.connect();
        x
    }

    /// Gets the [Signal] for the LEDs.
    pub fn leds() -> Signal<Out, Bits<3>> {
        let mut x = Signal::<Out, _>::default();

        [Self::LedRed, Self::LedGreen, Self::LedBlue]
            .into_iter()
            .map(|l| l.pad())
            .enumerate()
            .for_each(|(ndx, pad)| x.add_location(ndx, pad));

        x
    }
}

impl Default for Pin {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: add DDR3 pins
