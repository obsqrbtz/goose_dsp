use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

fn main() {
    #[cfg(target_os = "windows")]
    let host = cpal::host_from_id(cpal::HostId::Asio).expect("failed to initialize ASIO host");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("no input device available");
    println!("Using input device: {}", device.name().unwrap());

    println!("\nSupported input configurations:");
    for config in device.supported_input_configs().unwrap() {
        println!(
            "  Sample rate range: {:?} - {:?} Hz",
            config.min_sample_rate().0,
            config.max_sample_rate().0
        );
        println!("  Channels: {}", config.channels());
        println!("  Buffer size: {:?}", config.buffer_size());
        println!("  Sample format: {:?}\n", config.sample_format());
    }

    let supported_config = device
        .supported_input_configs()
        .unwrap()
        .find(|config| {
            config.channels() == 2
                && config.sample_format() == cpal::SampleFormat::I32
                && config.min_sample_rate().0 == 44100
                && config.max_sample_rate().0 == 44100
                && device
                    .supported_output_configs()
                    .unwrap()
                    .any(|output_config| {
                        output_config.channels() == config.channels()
                            && output_config.sample_format() == config.sample_format()
                            && output_config.min_sample_rate().0 == 44100
                            && output_config.max_sample_rate().0 == 44100
                    })
        })
        .expect("no supported config");

    let config = supported_config
        .with_sample_rate(cpal::SampleRate(44100))
        .config();

    println!("Using config:");
    println!("  Channels: {}", config.channels);
    println!("  Sample Rate: {} Hz", config.sample_rate.0);
    println!("  Buffer Size: {:?}", config.buffer_size);

    let processed_audio = Arc::new(Mutex::new(Vec::new()));
    let processed_audio_clone = Arc::clone(&processed_audio);

    let input_stream = device
        .build_input_stream(
            &config,
            move |data: &[i32], _: &cpal::InputCallbackInfo| {
                *processed_audio.lock().unwrap() = data.to_vec();
            },
            |err| eprintln!("Input error: {}", err),
            None,
        )
        .expect("failed to build input stream");

    let output_stream = device
        .build_output_stream(
            &config,
            move |data: &mut [i32], _: &cpal::OutputCallbackInfo| {
                let processed = processed_audio_clone.lock().unwrap();
                if !processed.is_empty() {
                    data.copy_from_slice(&processed);
                }
            },
            |err| eprintln!("Output error: {}", err),
            None,
        )
        .expect("failed to build output stream");

    input_stream.play().expect("failed to start input stream");
    output_stream.play().expect("failed to start output stream");

    println!("Echo started. Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
