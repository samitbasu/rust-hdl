#[macro_export]
macro_rules! hls_fifo_write_lazy {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo:ident).+, $data: expr) => {
        wait_clock_true!($sim, $clock, $uut);
        for val in $data {
            // Wait for the FIFO to not be full
            $uut = $sim.watch(|x| !x.$($fifo).+.full.val(), $uut)?;
            // Set the data lines and pulse the write signal
            $uut.$($fifo).+.data.next = (*val).to_bits();
            $uut.$($fifo).+.write.next = true;
            wait_clock_cycle!($sim, $clock, $uut);
            $uut.$($fifo).+.write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.2 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!($sim, $clock, $uut);
                }
            }
        }
    }
}

#[macro_export]
macro_rules! hls_fifo_write {
    ($sim: ident, $clock:ident, $uut: ident, $($fifo:ident).+, $data: expr) => {
        wait_clock_true!($sim, $clock, $uut);
        for val in $data {
            // Wait for the FIFO to not be full
            $uut = $sim.watch(|x| !x.$($fifo).+.full.val(), $uut)?;
            // Set the data lines and pulse the write signal
            $uut.$($fifo).+.data.next = val.to_bits();
            $uut.$($fifo).+.write.next = true;
            wait_clock_cycle!($sim, $clock, $uut);
            $uut.$($fifo).+.write.next = false;
        }
    }
}

#[macro_export]
macro_rules! hls_fifo_read {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo:ident).*, $data: expr) => {
        wait_clock_true!($sim, $clock, $uut);
        for val in $data {
            $uut = $sim.watch(|x| !x.$($fifo).+.empty.val(), $uut)?;
            sim_assert_eq!($sim, $uut.$($fifo).+.data.val(), *val, $uut);
            $uut.$($fifo).+.read.next = true;
            wait_clock_cycle!($sim, $clock, $uut);
            $uut.$($fifo).+.read.next = false;
        }
    }
}

#[macro_export]
macro_rules! hls_fifo_read_lazy {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo:ident).*, $data: expr) => {
        wait_clock_true!($sim, $clock, $uut);
        for val in $data {
            $uut = $sim.watch(|x| !x.$($fifo).+.empty.val(), $uut)?;
            sim_assert_eq!($sim, $uut.$($fifo).+.data.val(), *val, $uut);
            $uut.$($fifo).+.read.next = true;
            wait_clock_cycle!($sim, $clock, $uut);
            $uut.$($fifo).+.read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.2 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!($sim, $clock, $uut);
                }
            }
        }
    }
}

#[macro_export]
macro_rules! hls_host_get_word {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo:ident).*) => {
        {
        wait_clock_true!($sim, $clock, $uut);
        let mut ret = 0x0_u16;
        $uut = $sim.watch(|x| !x.$($fifo).+.bus_read.empty.val(), $uut)?;
        ret = $uut.$($fifo).+.bus_read.data.val().to_u16();
        $uut.$($fifo).+.bus_read.read.next = true;
        wait_clock_cycle!($sim, $clock, $uut);
        $uut.$($fifo).+.bus_read.read.next = false;
        $uut = $sim.watch(|x| !x.$($fifo).+.bus_read.empty.val(), $uut)?;
        let byte: u8 = $uut.$($fifo).+.bus_read.data.val().to_u8();
        ret = (ret << 8) | (byte as u16);
        $uut.$($fifo).+.bus_read.read.next = true;
        wait_clock_cycle!($sim, $clock, $uut);
        $uut.$($fifo).+.bus_read.read.next = false;
        ret
    }}
}

#[macro_export]
macro_rules! hls_host_get_words {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo: ident).+, $count: expr) => {
        {
            let mut ret = vec![];
            for _ in 0..$count {
                ret.push(hls_host_get_word!($sim, $clock, $uut, $($fifo).+));
            }
            ret
        }
    }
}

#[macro_export]
macro_rules! hls_host_put_word {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo:ident).*, $val: expr) => {
        hls_fifo_write!($sim, $clock, $uut, $($fifo).+.bus_write, [($val >> 8) as u8, ($val & 0xFF) as u8]);
    }
}

#[macro_export]
macro_rules! hls_host_ping {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo:ident).+, $data: expr) => {
        hls_host_put_word!($sim, $clock, $uut, $($fifo).+, 0x0100_u16 | ($data as u16));
    }
}

#[macro_export]
macro_rules! hls_host_noop {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo: ident).+) => {
        hls_host_put_word!($sim, $clock, $uut, $($fifo).+, 0x0000_u16);
    }
}

#[macro_export]
macro_rules! hls_host_write {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo: ident).+, $addr: expr, $data: expr) => {
        hls_host_put_word!($sim, $clock, $uut, $($fifo).+, 0x0300_u16 | ($addr as u16));
        hls_host_put_word!($sim, $clock, $uut, $($fifo).+, $data.len() as u16);
        for word in $data {
            hls_host_put_word!($sim, $clock, $uut, $($fifo).+, word as u16);
        }
    }
}

#[macro_export]
macro_rules! hls_host_issue_read {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo: ident).+, $addr: expr, $count: expr) => {
        {
            hls_host_put_word!($sim, $clock, $uut, $($fifo).+, 0x0200_u16 | ($addr as u16));
            hls_host_put_word!($sim, $clock, $uut, $($fifo).+, $count as u16);
        }
    }
}

#[macro_export]
macro_rules! hls_host_drain {
    ($sim: ident, $clock: ident, $uut: ident, $($fifo: ident).+) => {
        {
            $uut.$($fifo).+.bus_read.read.next = false;
            while (!$uut.$($fifo).+.bus_read.empty.val()) {
                $uut.$($fifo).+.bus_read.read.next = true;
                wait_clock_cycle!($sim, $clock, $uut);
                $uut.$($fifo).+.bus_read.read.next = false;
            }
        }
    }
}

#[macro_export]
macro_rules! bus_address_strobe {
    ($sim: ident, $uut: ident, $field: ident, $addr: expr) => {{
        wait_clock_true!($sim, $field.clock, $uut);
        $uut.$field.address.next = ($addr as u32).to_bits();
        $uut.$field.address_strobe.next = true;
        wait_clock_cycle!($sim, $field.clock, $uut);
        $uut.$field.address_strobe.next = false;
        $uut.$field.address.next = 0.into();
        $uut = $sim.watch(|x| x.$field.ready.val(), $uut)?;
    }};
}

#[macro_export]
macro_rules! bus_write_strobe {
    ($sim: ident,$uut: ident, $field: ident, $val: expr) => {{
        wait_clock_true!($sim, $field.clock, $uut);
        $uut.$field.from_controller.next = ($val).to_bits();
        $uut.$field.strobe.next = true;
        wait_clock_cycle!($sim, $field.clock, $uut);
        $uut.$field.strobe.next = false;
    }};
}
