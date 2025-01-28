pub struct NoiseGate {
    threshold: f32,
    attack_time: f32,
    release_time: f32,
    sample_rate: f32,
    envelope: f32,
}

impl NoiseGate {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: -60.0,
            attack_time: 0.1,
            release_time: 0.2,
            sample_rate,
            envelope: 0.0,
        }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    pub fn process(&mut self, input: &mut [f32]) {
        let attack_coef = (-1.0 / (self.attack_time * self.sample_rate)).exp();
        let release_coef = (-1.0 / (self.release_time * self.sample_rate)).exp();

        for sample in input.iter_mut() {
            // Convert to dB
            let input_level = if *sample != 0.0 {
                20.0 * sample.abs().log10()
            } else {
                -100.0
            };

            // Envelope follower
            if input_level > self.envelope {
                self.envelope = input_level * (1.0 - attack_coef) + self.envelope * attack_coef;
            } else {
                self.envelope = input_level * (1.0 - release_coef) + self.envelope * release_coef;
            }

            // Apply gate
            let gain = if self.envelope < self.threshold {
                0.0
            } else {
                1.0
            };

            *sample *= gain;
        }
    }
}
