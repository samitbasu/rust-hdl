#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MemoryTimings {
    pub initial_delay_in_nanoseconds: f64,
    pub t_rp_recharge_period_nanoseconds: f64,
    pub t_rfc_autorefresh_period_nanoseconds: f64,
    pub load_mode_command_timing_clocks: u32,
    pub t_ras_row_active_min_time_nanoseconds: f64,
    pub t_rc_row_to_row_min_time_nanoseconds: f64,
    pub t_rcd_row_to_column_min_time_nanoseconds: f64,
    pub t_rrd_bank_to_bank_activate_min_time_nanoseconds: f64,
    pub t_wr_write_recovery_time_nanoseconds: f64,
    pub t_refresh_max_interval_nanoseconds: f64,
    pub clock_speed_hz: f64,
    pub columns_per_bank: u32,
    pub rows_per_bank: u32,
    pub num_banks: u32,
}

impl MemoryTimings {
    pub fn mt48lc8m16a2(clock_speed_hz: f64) -> Self {
        Self {
            initial_delay_in_nanoseconds: 100.0e3,
            t_rp_recharge_period_nanoseconds: 20.0,
            t_rfc_autorefresh_period_nanoseconds: 66.0,
            load_mode_command_timing_clocks: 2,
            t_ras_row_active_min_time_nanoseconds: 44.0,
            t_rc_row_to_row_min_time_nanoseconds: 66.0,
            t_rcd_row_to_column_min_time_nanoseconds: 20.0,
            t_rrd_bank_to_bank_activate_min_time_nanoseconds: 15.0,
            t_wr_write_recovery_time_nanoseconds: 15.0,
            t_refresh_max_interval_nanoseconds: 16e6 / 4096.0,
            clock_speed_hz,
            columns_per_bank: 512,
            rows_per_bank: 4096,
            num_banks: 4,
        }
    }
    pub fn is42s16320f7(clock_speed_hz: f64) -> Self {
        Self {
            initial_delay_in_nanoseconds: 100.0e3,
            t_rp_recharge_period_nanoseconds: 15.0,
            t_rfc_autorefresh_period_nanoseconds: 60.0,
            load_mode_command_timing_clocks: 2,
            t_ras_row_active_min_time_nanoseconds: 37.0,
            t_rc_row_to_row_min_time_nanoseconds: 60.0,
            t_rcd_row_to_column_min_time_nanoseconds: 15.0,
            t_rrd_bank_to_bank_activate_min_time_nanoseconds: 14.0,
            t_wr_write_recovery_time_nanoseconds: 14.0,
            t_refresh_max_interval_nanoseconds: 16e6 / 4096.0,
            clock_speed_hz,
            columns_per_bank: 1024,
            rows_per_bank: 8192,
            num_banks: 4,
        }
    }
    pub fn fast_boot_sim(clock_speed_hz: f64) -> Self {
        Self {
            initial_delay_in_nanoseconds: 1000.0,
            t_rp_recharge_period_nanoseconds: 20.0,
            t_rfc_autorefresh_period_nanoseconds: 66.0,
            load_mode_command_timing_clocks: 2,
            t_ras_row_active_min_time_nanoseconds: 44.0,
            t_rc_row_to_row_min_time_nanoseconds: 66.0,
            t_rcd_row_to_column_min_time_nanoseconds: 20.0,
            t_rrd_bank_to_bank_activate_min_time_nanoseconds: 15.0,
            t_wr_write_recovery_time_nanoseconds: 14.0,
            t_refresh_max_interval_nanoseconds: 16e6 / 8192.0,
            clock_speed_hz,
            columns_per_bank: 32,
            rows_per_bank: 32,
            num_banks: 4,
        }
    }
    pub fn t_boot(&self) -> u16 {
        nanos_to_clocks(self.initial_delay_in_nanoseconds, self.clock_speed_hz)
    }
    pub fn t_rp(&self) -> u16 {
        nanos_to_clocks(self.t_rp_recharge_period_nanoseconds, self.clock_speed_hz)
    }
    pub fn t_rfc(&self) -> u16 {
        nanos_to_clocks(
            self.t_rfc_autorefresh_period_nanoseconds,
            self.clock_speed_hz,
        )
    }
    pub fn t_ras(&self) -> u16 {
        nanos_to_clocks(
            self.t_ras_row_active_min_time_nanoseconds,
            self.clock_speed_hz,
        )
    }
    pub fn t_rc(&self) -> u16 {
        nanos_to_clocks(
            self.t_rc_row_to_row_min_time_nanoseconds,
            self.clock_speed_hz,
        )
    }
    pub fn t_rcd(&self) -> u16 {
        nanos_to_clocks(
            self.t_rcd_row_to_column_min_time_nanoseconds,
            self.clock_speed_hz,
        )
    }
    pub fn t_rrd(&self) -> u16 {
        nanos_to_clocks(
            self.t_rrd_bank_to_bank_activate_min_time_nanoseconds,
            self.clock_speed_hz,
        )
    }
    pub fn t_wr(&self) -> u16 {
        nanos_to_clocks(
            self.t_wr_write_recovery_time_nanoseconds,
            self.clock_speed_hz,
        )
    }
    pub fn t_refresh_max(&self) -> u16 {
        nanos_to_clocks(self.t_refresh_max_interval_nanoseconds, self.clock_speed_hz)
    }
}

pub fn nanos_to_clocks(time_in_nanos: f64, clock_speed_hz: f64) -> u16 {
    let clock_period_in_nanos = 1.0e9 / clock_speed_hz;
    (time_in_nanos / clock_period_in_nanos).ceil() as u16
}
