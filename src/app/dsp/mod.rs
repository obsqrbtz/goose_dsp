pub mod overdrive;
use cpal::StreamConfig;

pub fn process_audio(
    data: &[i32],
    _config: &Option<StreamConfig>,
    input_volume: f32,
    output_volume: f32,
    overdrive_enabled: bool,
    threshold: f32,
    gain: f32,
) -> Vec<i32> {
    // Convert i32 to float for processing
    let mut float_data: Vec<f32> = data.iter().map(|&x| x as f32 / i32::MAX as f32).collect();

    // Apply input volume
    for sample in &mut float_data {
        *sample *= input_volume;
    }

    // Apply overdrive if enabled
    if overdrive_enabled {
        overdrive::apply_overdrive(&mut float_data, threshold, gain);
    }

    // Apply output volume and convert back to i32
    float_data
        .iter()
        .map(|&x| (x * output_volume * i32::MAX as f32) as i32)
        .collect()
}
