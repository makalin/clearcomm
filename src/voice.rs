use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use opus::{Decoder, Encoder};
use std::sync::Arc;

pub struct VoiceHandler {
    encoder: Encoder,
    decoder: Decoder,
    stream: Option<cpal::Stream>,
}

impl VoiceHandler {
    pub fn new() -> Result<Self> {
        let encoder = Encoder::new(48000, opus::Channels::Mono, opus::Application::Voip)?;
        let decoder = Decoder::new(48000, opus::Channels::Mono)?;
        
        Ok(Self {
            encoder,
            decoder,
            stream: None,
        })
    }

    pub fn start_recording(&mut self, tx: mpsc::UnboundedSender<Vec<u8>>) -> Result<()> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;

        let config = device.default_input_config()?;
        let encoder = Arc::new(Mutex::new(self.encoder.clone()));

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                let mut encoder = encoder.lock().unwrap();
                let mut opus_data = vec![0; 1024];
                if let Ok(encoded) = encoder.encode_float(data, &mut opus_data) {
                    let _ = tx.unbounded_send(opus_data[..encoded].to_vec());
                }
            },
            |err| eprintln!("Error in audio stream: {}", err),
        )?;

        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn decode_voice_data(&mut self, data: &[u8]) -> Result<Vec<f32>> {
        let mut pcm_data = vec![0.0; 1024];
        let decoded = self.decoder.decode_float(Some(data), &mut pcm_data, false)?;
        Ok(pcm_data[..decoded].to_vec())
    }
}
