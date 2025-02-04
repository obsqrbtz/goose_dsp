pub struct CabinetSim {
    coefficients: Vec<f32>,
}

impl CabinetSim {
    pub fn new() -> Self {
        let mut coeffs = vec![
            1.0, 0.9, 0.7, 0.5, 0.3, 0.2, 0.1, 0.05, 0.025, 0.0125, 0.00625, 0.003125, 0.001563,
            0.000781, 0.000391, 0.000195, 0.0001,
        ];

        let sum: f32 = coeffs.iter().sum();
        for coeff in coeffs.iter_mut() {
            *coeff /= sum;
        }

        CabinetSim {
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
