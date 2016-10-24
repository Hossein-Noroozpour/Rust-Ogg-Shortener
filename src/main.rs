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
    struct AppData {
        l_in_file: gtk::Label,
        l_out_file: gtk::Label,
        window: gtk::Window,
    }

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
    window.set_title("Rust vorbis shortener");
    window.set_position(gtk::WindowPosition::Center);
    window.add(&grid);
    window.connect_delete_event(|_, _| {
       gtk::main_quit();
       Inhibit(false)
    });
    window.set_resizable(false);
    window.show_all();

    let data = std::sync::Arc::new(std::sync::RwLock::new(AppData{
        l_in_file: l_in_file,
        l_out_file: l_out_file,
        window: window
    }));

    let b_in_data = data.clone();
    b_in.connect_clicked(move |_| {
        let data = b_in_data.write().unwrap();
        let dialog = gtk::FileChooserDialog::new(
            Some("Choose a ogg file"), Some(&data.window), gtk::FileChooserAction::Open);
        dialog.add_buttons(&[
            ("Open", gtk::ResponseType::Ok.into()),
            ("Cancel", gtk::ResponseType::Cancel.into())
        ]);
        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.ogg");

        dialog.set_filter(&filter);

        dialog.set_select_multiple(false);
        if dialog.run() == gtk::ResponseType::Ok.into() {
            data.l_in_file.set_text(
                dialog.get_filename().expect("Unexpected behavior!")
                .to_str().expect("Unexpected behavior!"));
        }
        dialog.destroy();
    });

    let b_out_data = data.clone();
    b_out.connect_clicked(move |_| {
        let data = b_out_data.write().unwrap();
        let dialog = gtk::FileChooserDialog::new(
            Some("Create a ogg file"), Some(&data.window), gtk::FileChooserAction::Save);
        dialog.add_buttons(&[
            ("Save", gtk::ResponseType::Ok.into()),
            ("Cancel", gtk::ResponseType::Cancel.into())
        ]);
        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.ogg");

        dialog.set_filter(&filter);

        dialog.set_select_multiple(false);
        if dialog.run() == gtk::ResponseType::Ok.into() {
            data.l_out_file.set_text(
                dialog.get_filename().expect("Unexpected behavior!")
                .to_str().expect("Unexpected behavior!"));
        }
        dialog.destroy();
    });

    b_run.connect_clicked(move |_| {
        let data = data.read().unwrap();
        let in_file = data.l_in_file.get_text().expect("Unexpected behavior");
        let out_file = data.l_out_file.get_text().expect("Unexpected behavior");
        if in_file.len() == 0 {
            let dialog = gtk::MessageDialog::new(
                Some(&data.window),
                gtk::DIALOG_MODAL,
                gtk::MessageType::Error,
                gtk::ButtonsType::Close,
                "Please specify an OGG input file!");
            dialog.run();
            dialog.destroy();
            return;
        }
        if out_file.len() == 0 {
            let dialog = gtk::MessageDialog::new(
                Some(&data.window),
                gtk::DIALOG_MODAL,
                gtk::MessageType::Error,
                gtk::ButtonsType::Close,
                "Please specify an OGG output file!");
            dialog.run();
            dialog.destroy();
            return;
        }
        if !in_file.ends_with(".ogg") {
            let dialog = gtk::MessageDialog::new(
                Some(&data.window),
                gtk::DIALOG_MODAL,
                gtk::MessageType::Error,
                gtk::ButtonsType::Close,
                "Please specify an OGG formated file for input file!");
            dialog.run();
            dialog.destroy();
            return;
        }
        if !out_file.ends_with(".ogg") {
            let dialog = gtk::MessageDialog::new(
                Some(&data.window),
                gtk::DIALOG_MODAL,
                gtk::MessageType::Error,
                gtk::ButtonsType::Close,
                "Please specify an OGG formated file for output file!");
            dialog.run();
            dialog.destroy();
            return;
        }
        shortener(&in_file, &out_file);
    });

    gtk::main();
}
