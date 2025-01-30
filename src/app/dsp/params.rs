use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AudioParams {
    pub input_volume: f32,
    pub output_volume: f32,
    pub overdrive_enabled: bool,
    pub overdrive_threshold: f32,
    pub overdrive_gain: f32,
    pub eq_enabled: bool,
    pub eq_low: f32,
    pub eq_mid: f32,
    pub eq_high: f32,
    pub gate_enabled: bool,
    pub gate_threshold: f32,
    pub cabinet_enabled: bool,
}

pub type SharedParams = Arc<Mutex<AudioParams>>;

impl AudioParams {
    pub fn new(
        input_volume: f32,
        output_volume: f32,
        overdrive_enabled: bool,
        threshold: f32,
        gain: f32,
    ) -> Self {
        Self {
            input_volume,
            output_volume,
            overdrive_enabled,
            overdrive_threshold: threshold,
            overdrive_gain: gain,
            eq_enabled: false,
            cabinet_enabled: false,
            eq_low: 1.0,
            eq_mid: 1.0,
            eq_high: 1.0,
            gate_enabled: false,
            gate_threshold: -40.0,
        }
    }
}
