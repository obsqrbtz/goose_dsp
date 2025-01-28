pub mod overdrive;
use cpal::StreamConfig;

pub fn process_audio(
    data: &[f32],
    _config: &Option<StreamConfig>, // remove underscore if will be used
    input_volume: f32,
    output_volume: f32,
    overdrive_enabled: bool,
    threshold: f32,
    gain: f32,
) -> Vec<f32> {
    let mut adjusted_data: Vec<f32> = data.iter().map(|&x| x * input_volume).collect();

    if overdrive_enabled {
        overdrive::apply_overdrive(&mut adjusted_data, threshold, gain);
    }

    let processed_data: Vec<f32> = adjusted_data.iter().map(|&x| x * output_volume).collect();
    processed_data
}
