pub fn apply_overdrive(input: &mut [f32], threshold: f32, gain: f32) {
    let cutoff_freq = 500.0;
    let sample_rate = 44100.0;
    for sample in &mut *input {
        let amplified = *sample * gain;
        *sample = if amplified > threshold {
            threshold + (amplified - threshold) * 0.3
        } else {
            amplified
        };
    }

    // TODO: move lowpass to separate FX slot
    let mut previous_input = 0.0;
    let mut previous_output = 0.0;
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    for sample in &mut *input {
        let raw = *sample - previous_input;
        previous_input = *sample;
        let filtered = previous_output + alpha * raw;
        previous_output = filtered;
        *sample = filtered;
    }

    for sample in input.iter_mut() {
        *sample = (*sample).tanh();
    }
}
