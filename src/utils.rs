#[derive(Copy, Clone)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub fn contains(&self, x: f64) -> bool {
        self.min < x && self.max > x
    }
}
