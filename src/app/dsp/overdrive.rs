pub fn apply_overdrive(input: &mut [f32], threshold: f32, gain: f32) {
    let input_gain = 2.0;
    for sample in input.iter_mut() {
        *sample *= input_gain;
    }

    // Pre-emphasis filter to boost highs before distortion
    let alpha = 0.2;
    let mut previous_input = 0.0;
    for sample in input.iter_mut() {
        let current = *sample;
        *sample = current - alpha * previous_input;
        previous_input = current;
    }

    // Soft clipping with asymmetric response
    for sample in input.iter_mut() {
        let amplified = *sample * gain;

        if amplified > threshold {
            *sample = threshold
                + (1.0 - (-((amplified - threshold) / threshold))).tanh() * threshold * 0.5;
        } else if amplified < -threshold {
            *sample =
                -threshold - (1.0 - ((amplified + threshold) / threshold)).tanh() * threshold * 0.6;
        } else {
            *sample = amplified;
        }

        *sample *= 3.0;
    }

    // Post-EQ filtering
    let cutoff_freq = 4000.0;
    let sample_rate = 44100.0;
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    let mut previous_output = input[0];
    for sample in input.iter_mut() {
        let current = *sample;
        let filtered = previous_output + alpha * (current - previous_output);
        previous_output = filtered;
        *sample = filtered;
    }

    let output_gain = 1.5;
    for sample in input.iter_mut() {
        *sample *= output_gain;
        *sample = sample.tanh();
    }
}
