use hound;
use std::fs::File;
use std::io::BufWriter;

fn process_audio(input: &mut [f32], output: &mut [f32]) {
    let input_gain = 1.0;
    for sample in input.iter_mut() {
        *sample *= input_gain;
    }
    output.copy_from_slice(input);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open("test/di.wav")?;
    let spec = reader.spec();
    let mut samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    let mut processed_samples = vec![0.0; samples.len()];
    process_audio(&mut samples, &mut processed_samples);

    let output_file = File::create("target/output.wav")?;
    let writer = BufWriter::new(output_file);
    let mut wav_writer = hound::WavWriter::new(writer, spec)?;

    for &sample in &processed_samples {
        wav_writer.write_sample((sample * i16::MAX as f32) as i16)?;
    }

    wav_writer.finalize()?;
    Ok(())
}
