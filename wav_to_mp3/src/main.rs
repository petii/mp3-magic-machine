mod mp3_encode_i16 {
    use mp3lame_encoder as mp3;

    type SampleType = i16;

    fn build_mp3_encoder(sample_rate: u32, channels: u8) -> mp3::Encoder {
        let mut encoder = mp3::Builder::new().expect("creating builder");
        encoder
            .set_sample_rate(sample_rate)
            .expect("set sample_rate");
        encoder
            .set_num_channels(channels)
            .expect("set number of channels");
        encoder.build().expect("building builder")
    }

    fn encode_mono_mp3(sample_rate: u32, samples: &[SampleType]) -> Vec<u8> {
        let mut output_buffer = Vec::new();
        output_buffer.reserve(mp3::max_required_buffer_size(samples.len()));

        let encoded = build_mp3_encoder(sample_rate, 1)
            .encode_to_vec(mp3::MonoPcm { 0: samples }, &mut output_buffer)
            .expect("writing sample buffer");

        Vec::from(&output_buffer[..encoded])
    }

    fn encode_stereo_mp3(
        sample_rate: u32,
        left_samples: &[SampleType],
        right_samples: &[SampleType],
    ) -> Vec<u8> {
        let mut output_buffer = Vec::new();
        output_buffer.reserve(mp3::max_required_buffer_size(
            left_samples.len() + right_samples.len(),
        ));

        let encoded = build_mp3_encoder(sample_rate, 2)
            .encode_to_vec(
                mp3::DualPcm {
                    left: left_samples,
                    right: right_samples,
                },
                &mut output_buffer,
            )
            .expect("writing sample buffer");

        Vec::from(&output_buffer[..encoded])
    }

    pub fn from_reader<R: std::io::Read>(
        reader: hound::WavReader<R>,
        out_dir: &std::path::Path,
        base_name: &str,
    ) {
        use std::path::Path;

        let sample_rate = reader.spec().sample_rate;

        match reader.spec().channels {
            1 => unimplemented!(),
            2 => {
                let left_mp3 =
                    out_dir.join(Path::new(&format!("{base_name}-left")).with_extension("mp3"));
                let right_mp3 =
                    out_dir.join(Path::new(&format!("{base_name}-right")).with_extension("mp3"));

                let mut left_samples = Vec::new();
                let mut right_samples = Vec::new();

                let mut is_left = false;
                for s in reader.into_samples::<i16>() {
                    is_left = !is_left;
                    if s.is_err() {
                        continue;
                    }
                    let sample = s.unwrap();
                    if is_left {
                        left_samples.push(sample);
                    } else {
                        right_samples.push(sample);
                    }
                }

                let mp3_buffer = encode_mono_mp3(sample_rate, &left_samples);
                std::fs::write(left_mp3, mp3_buffer).unwrap();

                let mp3_buffer = encode_mono_mp3(sample_rate, &right_samples);
                std::fs::write(right_mp3, mp3_buffer).unwrap();

                let mp3_buffer = encode_stereo_mp3(sample_rate, &left_samples, &right_samples);
                std::fs::write(out_dir.join(format!("{base_name}.mp3")), mp3_buffer).unwrap();
            }
            _ => unimplemented!(),
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let src = std::fs::File::open(&path).unwrap();

    let base_name = std::path::Path::new(&path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let out = std::path::Path::new(&path).parent().unwrap();

    let wav_reader = hound::WavReader::new(std::io::BufReader::new(src)).unwrap();
    let spec = wav_reader.spec();
    dbg!(spec);
    match spec.bits_per_sample {
        16 => mp3_encode_i16::from_reader(wav_reader, out, base_name),
        _ => unimplemented!(),
    }
}
