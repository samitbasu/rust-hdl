use crate::core::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug, LogicState)]
pub enum SDRAMCommand {
    LoadModeRegister, // LLL
    AutoRefresh,      // LLH
    Precharge,        // LHL
    BurstTerminate,   // HHL
    Write,            // HLL
    Read,             // HLH
    Active,           // LHH
    NOP,              // HHH
}

#[derive(LogicBlock, Default)]
pub struct SDRAMCommandEncoder {
    pub ras_not: Signal<Out, Bit>,
    pub cas_not: Signal<Out, Bit>,
    pub we_not: Signal<Out, Bit>,
    pub cs_not: Signal<Out, Bit>,
    pub cmd: Signal<In, SDRAMCommand>,
}

impl Logic for SDRAMCommandEncoder {
    #[hdl_gen]
    fn update(&mut self) {
        self.cs_not.next = false;
        match self.cmd.val() {
            SDRAMCommand::LoadModeRegister => {
                self.ras_not.next = false;
                self.cas_not.next = false;
                self.we_not.next = false;
            }
            SDRAMCommand::AutoRefresh => {
                self.ras_not.next = false;
                self.cas_not.next = false;
                self.we_not.next = true;
            }
            SDRAMCommand::Precharge => {
                self.ras_not.next = false;
                self.cas_not.next = true;
                self.we_not.next = false;
            }
            SDRAMCommand::BurstTerminate => {
                self.ras_not.next = true;
                self.cas_not.next = true;
                self.we_not.next = false;
            }
            SDRAMCommand::Write => {
                self.ras_not.next = true;
                self.cas_not.next = false;
                self.we_not.next = false;
            }
            SDRAMCommand::Read => {
                self.ras_not.next = true;
                self.cas_not.next = false;
                self.we_not.next = true;
            }
            SDRAMCommand::Active => {
                self.ras_not.next = false;
                self.cas_not.next = true;
                self.we_not.next = true;
            }
            SDRAMCommand::NOP => {
                self.ras_not.next = true;
                self.cas_not.next = true;
                self.we_not.next = true;
            }
        }
    }
}

#[derive(LogicBlock, Default)]
pub struct SDRAMCommandDecoder {
    pub ras_not: Signal<In, Bit>,
    pub cas_not: Signal<In, Bit>,
    pub we_not: Signal<In, Bit>,
    pub cs_not: Signal<In, Bit>,
    pub cmd: Signal<Out, SDRAMCommand>,
}

impl Logic for SDRAMCommandDecoder {
    #[hdl_gen]
    fn update(&mut self) {
        self.cmd.next = SDRAMCommand::NOP;
        // Decode the SDRAM command
        if !self.cs_not.val() {
            if self.ras_not.val() & self.cas_not.val() & self.we_not.val() {
                self.cmd.next = SDRAMCommand::NOP;
            } else if self.ras_not.val() & self.cas_not.val() & !self.we_not.val() {
                self.cmd.next = SDRAMCommand::BurstTerminate;
            } else if self.ras_not.val() & !self.cas_not.val() & self.we_not.val() {
                self.cmd.next = SDRAMCommand::Read;
            } else if self.ras_not.val() & !self.cas_not.val() & !self.we_not.val() {
                self.cmd.next = SDRAMCommand::Write;
            } else if !self.ras_not.val() & self.cas_not.val() & self.we_not.val() {
                self.cmd.next = SDRAMCommand::Active;
            } else if !self.ras_not.val() & self.cas_not.val() & !self.we_not.val() {
                self.cmd.next = SDRAMCommand::Precharge;
            } else if !self.ras_not.val() & !self.cas_not.val() & self.we_not.val() {
                self.cmd.next = SDRAMCommand::AutoRefresh;
            } else if !self.ras_not.val() & !self.cas_not.val() & !self.we_not.val() {
                self.cmd.next = SDRAMCommand::LoadModeRegister;
            }
        }
    }
}
