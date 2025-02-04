pub struct EQ {
    low_gain: f32,
    mid_gain: f32,
    high_gain: f32,
    low_freq: f32,
    high_freq: f32,
    sample_rate: f32,
    low_prev: f32,
    high_prev: f32,
}

impl EQ {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            low_gain: 1.0,
            mid_gain: 1.0,
            high_gain: 1.0,
            low_freq: 250.0,
            high_freq: 2500.0,
            sample_rate,
            low_prev: 0.0,
            high_prev: 0.0,
        }
    }

    pub fn process(&mut self, input: &mut [f32]) {
        let dt = 1.0 / self.sample_rate;
        let low_alpha = dt / (1.0 / (2.0 * std::f32::consts::PI * self.low_freq) + dt);
        let high_alpha = dt / (1.0 / (2.0 * std::f32::consts::PI * self.high_freq) + dt);

        for sample in input.iter_mut() {
            let low = self.low_prev + low_alpha * (*sample - self.low_prev);
            self.low_prev = low;

            let high = (*sample - self.high_prev) * high_alpha;
            self.high_prev = high;

            let mid = *sample - low - high;

            *sample = (low * self.low_gain) + (mid * self.mid_gain) + (high * self.high_gain);
        }
    }

    pub fn set_gains(&mut self, low: f32, mid: f32, high: f32) {
        self.low_gain = low;
        self.mid_gain = mid;
        self.high_gain = high;
    }
}
