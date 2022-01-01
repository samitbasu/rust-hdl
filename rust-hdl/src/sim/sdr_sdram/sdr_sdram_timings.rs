#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MemoryTimings {
    pub initial_delay_in_nanoseconds: f64,
    pub t_rp_recharge_period_nanoseconds: f64,
    pub autorefresh_command_timing_nanoseconds: f64,
    pub load_mode_command_timing_clocks: u32,
    pub t_ras_row_active_min_time_nanoseconds: f64,
    pub t_rc_row_to_row_min_time_nanoseconds: f64,
    pub t_rcd_row_to_column_min_time_nanoseconds: f64,
}

impl MemoryTimings {
    pub fn mt48lc8m16a2() -> Self {
        Self {
            initial_delay_in_nanoseconds: 100.0e3,
            t_rp_recharge_period_nanoseconds: 20.0,
            autorefresh_command_timing_nanoseconds: 66.0,
            load_mode_command_timing_clocks: 2,
            t_ras_row_active_min_time_nanoseconds: 44.0,
            t_rc_row_to_row_min_time_nanoseconds: 66.0,
            t_rcd_row_to_column_min_time_nanoseconds: 20.0,
        }
    }
}

pub fn nanos_to_clocks(time_in_nanos: f64, clock_speed_hz: f64) -> u32 {
    let clock_period_in_nanos = 1.0e9 / clock_speed_hz;
    (time_in_nanos / clock_period_in_nanos).ceil() as u32
}
