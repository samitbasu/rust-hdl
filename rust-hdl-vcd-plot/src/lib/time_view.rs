use substring::Substring;
use crate::{Interval, TimedValue};
use crate::renderable::Renderable;

#[derive(Clone, Debug)]
pub struct TimeView {
    pub start_time: u64,
    pub end_time: u64,
    pub pixel_scale: f64,
}

impl TimeView {
    pub fn map(&self, time: u64) -> f64 {
        (self.start_time.max(time).min(self.end_time) - self.start_time) as f64 * self.pixel_scale
    }
    pub fn intervals<T: PartialEq + Clone + Renderable>(
        &self,
        vals: &[TimedValue<T>],
    ) -> Vec<Interval<T>> {
        vals.windows(2)
            .map(|x| {
                let end_x = self.map(x[1].time);
                let start_x = self.map(x[0].time);
                let label_max = ((end_x - start_x) / 6.0).round() as usize;
                let mut label = x[0].value.render();
                if label.len() > label_max {
                    if label_max <= 3 {
                        label = format!("!");
                    } else {
                        label = format!("{}+", label.substring(0, label_max - 1));
                    }
                }
                Interval {
                    start_time: x[0].time,
                    end_time: x[1].time,
                    value: x[0].value.clone(),
                    start_x,
                    end_x,
                    label,
                }
            })
            .collect()
    }
}
