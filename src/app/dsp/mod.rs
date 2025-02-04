pub mod cabinet;
pub mod eq;
pub mod gate;
pub mod overdrive;
pub mod params;
use cpal::StreamConfig;

use self::cabinet::CabinetSim;
use self::eq::EQ;
use self::gate::NoiseGate;

pub fn process_audio(
    data: &[i32],
    _config: &Option<StreamConfig>,
    audio_params: &params::SharedParams,
) -> Vec<i32> {
    let params = audio_params.lock().unwrap();

    let mut float_data: Vec<f32> = data
        .iter()
        .map(|&x| (x as f32 / i32::MAX as f32) * 1.5)
        .collect();

    // Apply noise gate first
    if params.gate_enabled {
        let mut gate = NoiseGate::new(44100.0);
        gate.set_threshold(params.gate_threshold);
        gate.process(&mut float_data);
    }

    for sample in &mut float_data {
        *sample *= params.input_volume * 2.0;
    }

    // Apply EQ before overdrive if enabled
    if params.eq_enabled {
        let mut eq = EQ::new(44100.0);
        eq.set_gains(params.eq_low, params.eq_mid, params.eq_high);
        eq.process(&mut float_data);
    }

    if params.overdrive_enabled {
        overdrive::apply_overdrive(
            &mut float_data,
            params.overdrive_threshold,
            params.overdrive_gain,
        );
        if params.cabinet_enabled {
            let cabinet = CabinetSim::new();
            cabinet.process(&mut float_data);
        }
    }

    float_data
        .iter()
        .map(|&x| {
            let boosted = x * params.output_volume * 3.0;
            let limited = boosted.tanh();
            (limited * i32::MAX as f32) as i32
        })
        .collect()
}
