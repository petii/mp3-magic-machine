mod mp3_encode_i16;

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
