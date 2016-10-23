extern crate gtk;
extern crate vorbis;

use std::io::Write;
use gtk::prelude::*;

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

fn shortener(in_file: &str, out_file: &str) {
    use std::fs::File;

    let f = File::open(in_file).unwrap();
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
    let mut file = File::create(out_file).expect("Unable to create output file!");
    file.write(&shortened[..]).expect("Unable to write to output file!");
}



fn main() {
    if gtk::init().is_err() {
        panic!("Failed to initialize GTK.");
    }

    let l1 = gtk::Label::new(Some("Input file: "));
    l1.set_halign(gtk::Align::Start);
    let l_in_file = gtk::Label::new(Some(""));
    l_in_file.set_halign(gtk::Align::Start);
    let b_in = gtk::Button::new_with_label("Choose input file");

    let l2 = gtk::Label::new(Some("Output file: "));
    l2.set_halign(gtk::Align::Start);
    let l_out_file = gtk::Label::new(Some(""));
    l_out_file.set_halign(gtk::Align::Start);
    let b_out = gtk::Button::new_with_label("Choose output file");

    let b_run = gtk::Button::new_with_label("Shorten");

    let grid = gtk::Grid::new();
    grid.set_row_spacing(5);
    grid.set_column_spacing(5);
    grid.set_border_width(5);
    grid.attach(&l1, 0, 0, 1, 1);
    grid.attach(&l_in_file, 1, 0, 1, 1);
    grid.attach(&b_in, 2, 0, 1, 1);
    grid.attach(&l2, 0, 1, 1, 1);
    grid.attach(&l_out_file, 1, 1, 1, 1);
    grid.attach(&b_out, 2, 1, 1, 1);
    grid.attach(&b_run, 0, 2, 3, 1);

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Rust ogg shortener");
    window.set_position(gtk::WindowPosition::Center);
    window.add(&grid);
    window.connect_delete_event(|_, _| {
       gtk::main_quit();
       Inhibit(false)
    });
    window.show_all();
    window.set_resizable(false);
    gtk::main();
}
