use crate::core::prelude::*;
use crate::widgets::prelude::*;
use crate::widgets::sdram::*;

#[derive(LogicBlock, Clone, Default)]
pub struct SDRAMOnChipBuffer<const D: usize> {
    pub buf_in: SDRAMDevice<D>,
    pub buf_out: SDRAMDriver<D>,
    we_not_flop: DFF<Bit>,
    cas_not_flop: DFF<Bit>,
    ras_not_flop: DFF<Bit>,
    cs_not_flop: DFF<Bit>,
    bank_flop: DFF<Bits<2>>,
    address_flop: DFF<Bits<13>>,
    write_flop: DFF<Bits<D>>,
    read_flop: DFF<Bits<D>>,
}

impl<const D: usize> Logic for SDRAMOnChipBuffer<D> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect up the flop inputs
        self.we_not_flop.d.next = self.buf_in.we_not.val();
        self.cas_not_flop.d.next = self.buf_in.cas_not.val();
        self.ras_not_flop.d.next = self.buf_in.ras_not.val();
        self.cs_not_flop.d.next = self.buf_in.cs_not.val();
        self.bank_flop.d.next = self.buf_in.bank.val();
        self.address_flop.d.next = self.buf_in.address.val();
        self.write_flop.d.next = self.buf_in.write_data.val();
        self.read_flop.d.next = self.buf_out.read_data.val();
        // Connect up the flop outputs
        self.buf_out.we_not.next = self.we_not_flop.q.val();
        self.buf_out.cas_not.next = self.cas_not_flop.q.val();
        self.buf_out.ras_not.next = self.ras_not_flop.q.val();
        self.buf_out.cs_not.next = self.cs_not_flop.q.val();
        self.buf_out.bank.next = self.bank_flop.q.val();
        self.buf_out.address.next = self.address_flop.q.val();
        self.buf_out.write_enable.next = self.buf_in.write_enable.val();
        self.buf_in.read_data.next = self.read_flop.q.val();
        self.buf_out.write_data.next = self.write_flop.q.val();
        // Clock the flops
        self.we_not_flop.clock.next = self.buf_in.clk.val();
        self.cas_not_flop.clock.next = self.buf_in.clk.val();
        self.ras_not_flop.clock.next = self.buf_in.clk.val();
        self.cs_not_flop.clock.next = self.buf_in.clk.val();
        self.bank_flop.clock.next = self.buf_in.clk.val();
        self.address_flop.clock.next = self.buf_in.clk.val();
        self.write_flop.clock.next = self.buf_in.clk.val();
        self.read_flop.clock.next = self.buf_in.clk.val();
        // Forward the clock
        self.buf_out.clk.next = !self.buf_in.clk.val();
    }
}

#[test]
fn test_buffer_synthesizes() {
    let mut uut = TopWrap::new(SDRAMOnChipBuffer::<16>::default());
    uut.uut.buf_in.link_connect_dest();
    uut.uut.buf_out.link_connect_dest();
    uut.connect_all();
    yosys_validate("sdram_buffer", &generate_verilog(&uut)).unwrap();
}
