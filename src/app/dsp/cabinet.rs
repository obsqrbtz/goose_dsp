pub struct CabinetSim {
    sample_rate: f32,
    coefficients: Vec<f32>,
}

impl CabinetSim {
    pub fn new(sample_rate: f32) -> Self {
        // Simple IR coefficients simulating a 4x12 cabinet
        let mut coeffs = vec![
            1.0, 0.8, 0.6, 0.4, 0.2, 0.1, 0.05, 0.025, 0.0125, 0.00625, 0.003125, 0.001563,
            0.000781, 0.000391,
        ];

        // Normalize coefficients
        let sum: f32 = coeffs.iter().sum();
        for coeff in coeffs.iter_mut() {
            *coeff /= sum;
        }

        CabinetSim {
            sample_rate,
            coefficients: coeffs,
        }
    }

    pub fn process(&self, input: &mut [f32]) {
        let mut buffer = vec![0.0; input.len()];

        for (i, sample) in input.iter().enumerate() {
            for (j, coeff) in self.coefficients.iter().enumerate() {
                if i + j < buffer.len() {
                    buffer[i + j] += sample * coeff;
                }
            }
        }

        input.copy_from_slice(&buffer[..input.len()]);
    }
}
