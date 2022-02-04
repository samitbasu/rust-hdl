#[macro_export]
macro_rules! i2c_begin_write {
    ($sim: ident, $clock: ident, $uut: ident, $addr: expr) => {
        $uut = $sim.watch(|x| !x.controller.busy.val(), $uut)?;
        wait_clock_true!($sim, $clock, $uut);
        $uut.controller.cmd.next = I2CControllerCmd::BeginWrite;
        $uut.controller.write_data_in.next = $addr.into();
        $uut.controller.run.next = true;
        wait_clock_cycle!($sim, $clock, $uut);
        $uut.controller.run.next = false;
        $uut = $sim.watch(|x| x.controller.nack.val() | x.controller.ack.val(), $uut)?;
    };
}

#[macro_export]
macro_rules! i2c_begin_read {
    ($sim: ident, $clock: ident, $uut: ident, $addr: expr) => {
        $uut = $sim.watch(|x| !x.controller.busy.val(), $uut)?;
        wait_clock_true!($sim, $clock, $uut);
        $uut.controller.cmd.next = I2CControllerCmd::BeginRead;
        $uut.controller.write_data_in.next = $addr.into();
        $uut.controller.run.next = true;
        wait_clock_cycle!($sim, $clock, $uut);
        $uut.controller.run.next = false;
        $uut = $sim.watch(|x| x.controller.nack.val() | x.controller.ack.val(), $uut)?;
    };
}

#[macro_export]
macro_rules! i2c_end_transmission {
    ($sim: ident, $clock: ident, $uut: ident) => {
        $uut = $sim.watch(|x| !x.controller.busy.val(), $uut)?;
        wait_clock_true!($sim, $clock, $uut);
        $uut.controller.cmd.next = I2CControllerCmd::EndTransmission;
        $uut.controller.run.next = true;
        wait_clock_cycle!($sim, $clock, $uut);
        $uut.controller.run.next = false;
        $uut = $sim.watch(|x| !x.controller.busy.val(), $uut)?;
        wait_clock_cycles!($sim, $clock, $uut, 10);
    };
}

#[macro_export]
macro_rules! i2c_write {
    ($sim: ident, $clock: ident, $uut: ident, $val: expr) => {
        $uut = $sim.watch(|x| !x.controller.busy.val(), $uut)?;
        wait_clock_true!($sim, $clock, $uut);
        $uut.controller.cmd.next = I2CControllerCmd::Write;
        $uut.controller.write_data_in.next = $val.into();
        $uut.controller.run.next = true;
        wait_clock_cycle!($sim, $clock, $uut);
        $uut.controller.run.next = false;
        $uut = $sim.watch(|x| x.controller.nack.val() | x.controller.ack.val(), $uut)?;
    };
}

#[macro_export]
macro_rules! i2c_read {
    ($sim: ident, $clock: ident, $uut: ident) => {{
        $uut = $sim.watch(|x| !x.controller.busy.val(), $uut)?;
        wait_clock_true!($sim, $clock, $uut);
        $uut.controller.cmd.next = I2CControllerCmd::Read;
        $uut.controller.run.next = true;
        wait_clock_cycle!($sim, $clock, $uut);
        $uut.controller.run.next = false;
        $uut = $sim.watch(|x| x.controller.read_valid.val(), $uut)?;
        $uut.controller.read_data_out.val()
    }};
}

#[macro_export]
macro_rules! i2c_read_last {
    ($sim: ident, $clock: ident, $uut: ident) => {{
        $uut = $sim.watch(|x| !x.controller.busy.val(), $uut)?;
        wait_clock_true!($sim, $clock, $uut);
        $uut.controller.cmd.next = I2CControllerCmd::ReadLast;
        $uut.controller.run.next = true;
        wait_clock_cycle!($sim, $clock, $uut);
        $uut.controller.run.next = false;
        $uut = $sim.watch(|x| x.controller.read_valid.val(), $uut)?;
        $uut.controller.read_data_out.val()
    }};
}
