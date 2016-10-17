extern crate rand;
extern crate vorbis;

use std::io::Write;

const LEAST_RATE: u64 = 8000;

fn channels_mixer(data: &Vec<i16>, channels: u16) -> Vec<i16> {
    let final_size = (data.len() as u64 / channels as u64) as usize;
    let mut short = vec![0i16; final_size];
    let mut index = 0u64;
    for i in 0..final_size {
        let mut bit = 0i64;
        for _ in 0..channels {
            bit += data[index as usize] as i64;
            index += 1;
        }
        bit /= channels as i64;
        short[i] = bit as i16;
    }
    return short;
}

fn rate_reducer(data: &Vec<i16>, rate: u64) -> Vec<i16> {
    let final_size = ((data.len() as u64 * LEAST_RATE) / rate as u64) as usize;
    let steps = rate as f64 / LEAST_RATE as f64;
    let mut short = vec![0i16; final_size];
    let mut prindex = 0u64;
    for i in 1..final_size {
        let mut bit = data[prindex as usize] as i64;
        let nexindex = (i as f64 * steps) as u64;
        for index in (prindex+1)..nexindex {
            bit += data[index as usize] as i64;
        }
        let num = nexindex as i64 - prindex as i64;
        let num = if num > 0 {
            num
        } else {
            1
        };
        bit /= num;
        short[i] = bit as i16;
        prindex = nexindex;
    }
    return short;
}

fn main() {
    use std::fs::File;

    let f = File::open("/home/thany/Dropbox/Projects/Start/Music/back-1.ogg").unwrap();
    let mut decoder = vorbis::Decoder::new(f).unwrap();
    let packets = decoder.packets();
    let mut shortened = Vec::new();
    let mut channels = 0;
    let mut rate = 0;
    for p in packets {
        match p {
            Ok(mut packet) => {
                channels = packet.channels;
                rate = packet.rate;
                shortened.append(&mut packet.data);
            },
            _ => {
                panic!("Unexpected behavior, it is likely a bug in vorbis.")
            }
        }
    }
    println!("Original PCM size: {}, number of channels: {}, rate: {} ", shortened.len(), channels, rate);
    let shortened = channels_mixer(&shortened, channels);
    let shortened = rate_reducer(&shortened, rate);
    let mut encoder = vorbis::Encoder::new(1, LEAST_RATE, vorbis::VorbisQuality::VeryHighPerformance)
        .expect("Unable to create encoder!");
    let mut shortened = encoder.encode(&shortened).expect("Unable to encode to vorbis!");
    shortened.append(&mut encoder.flush().expect("Unable to flush!"));
    let mut file = File::create("/home/thany/1.ogg").expect("Unable to create output file!");
    file.write(&shortened[..]).expect("Unable to write to output file!");
}
