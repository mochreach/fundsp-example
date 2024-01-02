use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, SampleFormat, SizedSample, StreamConfig};
use fundsp::hacker::{
    hammond_hz, multipass, reverb_stereo, sine, sine_hz, soft_saw_hz, square_hz, wave64, Wave64,
};
use fundsp::prelude::AudioUnit64;
use std::sync::Arc;

/// This is the main function that is the entry point when we launch the
/// binary, either directly or with `cargo run`.
fn main() {
    // Change the `create_sine_440` function to any of the functions
    // that create a `Box<dyn AudioUnit64>` below, to change the
    // sound that's generated.
    let audio_graph = create_sine_440();

    // This function starts the thread that creates the audio and sends
    // it to CPAL so that we can hear it.
    run_output(audio_graph);

    // The audio is being played on a thread, and will run infinitely.
    // As soon as the main function exits, the sound will stop, so we
    // can sleep the main thread for a while so we can hear it.
    // Change the duration to play the sound for more or less time.
    let duration = 5;
    std::thread::sleep(std::time::Duration::from_secs(duration));
}

/// This function determines the sample format, which depends on your system,
/// then starts the synth, passing along the audio graph that will generate
/// the sound to be played.
fn run_output(audio_graph: Box<dyn AudioUnit64>) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    match config.sample_format() {
        SampleFormat::F32 => run_synth::<f32>(audio_graph, device, config.into()),
        SampleFormat::I16 => run_synth::<i16>(audio_graph, device, config.into()),
        SampleFormat::U16 => run_synth::<u16>(audio_graph, device, config.into()),

        _ => panic!("Unsupported format"),
    }
}

/// This function takes an audio graph as an input, along with some the audio
/// device and config, and starts a thread that will play the audio. The thread
/// will loop infinitely until the programme exits.
fn run_synth<T: SizedSample + FromSample<f64>>(
    mut audio_graph: Box<dyn AudioUnit64>,
    device: Device,
    config: StreamConfig,
) {
    std::thread::spawn(move || {
        let sample_rate = config.sample_rate.0 as f64;
        audio_graph.set_sample_rate(sample_rate);

        // This is a function that is used to get the next audio sample. It is
        // written using the closure syntax, so looks a bit different from
        // normal function definition.
        let mut next_value = move || audio_graph.get_stereo();

        let channels = config.channels as usize;
        let err_fn = |err| eprintln!("an error occurred on stream: {err}");
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut next_value)
                },
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
}

/// This function is passed to the output audio stream and is used to generate
/// audio samples and send them to the audio device.
fn write_data<T: SizedSample + FromSample<f64>>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> (f64, f64),
) {
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = T::from_sample(sample.0);
        let right: T = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            *sample = if channel & 1 == 0 { left } else { right };
        }
    }
}

// ------------------------------------------------------------------
// You can use any of the functions in this section to make the audio
// graph. Just replace the function call in `main` at the top.

/// Simple sine wave at 440 Hz which is standard tuning for A4
fn create_sine_440() -> Box<dyn AudioUnit64> {
    let synth = sine_hz(440.0);

    Box::new(synth)
}

/// C major chord created by summing waves! Sine by default, but try uncommenting
/// the other wave types.
fn create_c_major() -> Box<dyn AudioUnit64> {
    // let synth = sine_hz(261.6) + sine_hz(329.628) + sine_hz(391.995);
    // let synth = square_hz(261.6) + square_hz(329.628) + square_hz(391.995);
    // let synth = soft_saw_hz(261.6) + soft_saw_hz(329.628) + soft_saw_hz(391.995);
    let synth = hammond_hz(261.6) + hammond_hz(329.628) + hammond_hz(391.995);

    Box::new(synth)
}

/// Load and play a sample
fn create_sample() -> Box<dyn AudioUnit64> {
    let wave =
        Arc::new(Wave64::load("samples/closed_high_hat.wav").expect("Could not find sample file."));
    let left = wave64(&wave, 0, None);
    let right = wave64(&wave, 1, None);
    let synth = left | right;

    Box::new(synth)
}

/// Load and play a sample, but this time we add reverb
fn create_sample_with_reverb() -> Box<dyn AudioUnit64> {
    let wave =
        Arc::new(Wave64::load("samples/closed_high_hat.wav").expect("Could not find sample file."));
    let left = wave64(&wave, 0, None);
    let right = wave64(&wave, 1, None);
    let synth = (left | right) >> (multipass() & (0.2 * reverb_stereo(10.0, 3.0)));

    Box::new(synth)
}

// Simple FM synthesiser taken from the FunDSP docs
fn create_simple_fm() -> Box<dyn AudioUnit64> {
    // Frequency
    let f = 440.0;
    // Modulation index
    let m = 5.0;
    let synth = (sine_hz(f) * f * m + f) >> sine();

    Box::new(synth)
}
