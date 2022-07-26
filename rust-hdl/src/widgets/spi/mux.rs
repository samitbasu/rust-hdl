use crate::core::prelude::*;
use crate::widgets::prelude::*;

// Mux N SPI slaves onto a bus
#[derive(LogicBlock)]
pub struct MuxSlaves<const N: usize, const A: usize> {
    pub from_master: SPIWiresSlave,
    pub to_slaves: [SPIWiresMaster; N],
    pub sel: Signal<In, Bits<A>>,
}

impl<const N: usize, const A: usize> Default for MuxSlaves<N, A> {
    fn default() -> Self {
        assert!((1 << A) >= N);
        Self {
            from_master: Default::default(),
            to_slaves: array_init::array_init(|_| Default::default()),
            sel: Default::default(),
        }
    }
}

impl<const N: usize, const A: usize> Logic for MuxSlaves<N, A> {
    #[hdl_gen]
    fn update(&mut self) {
        self.from_master.miso.next = false;
        for i in 0..N {
            self.to_slaves[i].mclk.next = true;
            self.to_slaves[i].msel.next = true;
            self.to_slaves[i].mosi.next = true;
            if self.sel.val().index() == i {
                self.to_slaves[i].mclk.next = self.from_master.mclk.val();
                self.to_slaves[i].msel.next = self.from_master.msel.val();
                self.to_slaves[i].mosi.next = self.from_master.mosi.val();
                self.from_master.miso.next = self.to_slaves[i].miso.val();
            }
        }
    }
}

#[test]
fn test_spi_mux_slaves_is_synthesizable() {
    let mut uut = MuxSlaves::<4, 2>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("spi_mux_slaves", &vlog).unwrap();
}

// Mux N SPI masters onto a bus
#[derive(LogicBlock)]
pub struct MuxMasters<const N: usize, const A: usize> {
    pub to_bus: SPIWiresMaster,
    pub from_masters: [SPIWiresSlave; N],
    pub sel: Signal<In, Bits<A>>,
}

impl<const N: usize, const A: usize> Default for MuxMasters<N, A> {
    fn default() -> Self {
        assert!((1 << A) >= N);
        Self {
            to_bus: Default::default(),
            from_masters: array_init::array_init(|_| Default::default()),
            sel: Default::default(),
        }
    }
}

impl<const N: usize, const A: usize> Logic for MuxMasters<N, A> {
    #[hdl_gen]
    fn update(&mut self) {
        // Latch prevention
        self.to_bus.mosi.next = true;
        self.to_bus.msel.next = true;
        self.to_bus.mclk.next = true;
        for i in 0..N {
            self.from_masters[i].miso.next = true;
            if self.sel.val().index() == i {
                self.to_bus.mosi.next = self.from_masters[i].mosi.val();
                self.to_bus.msel.next = self.from_masters[i].msel.val();
                self.to_bus.mclk.next = self.from_masters[i].mclk.val();
                self.from_masters[i].miso.next = self.to_bus.miso.val();
            }
        }
    }
}

#[test]
fn test_spi_mux_is_synthesizable() {
    let mut uut = MuxMasters::<4, 2>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("spi_mux", &vlog).unwrap();
}
