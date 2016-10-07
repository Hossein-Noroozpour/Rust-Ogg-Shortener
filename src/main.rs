#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;
extern crate rand;
extern crate vorbis;

use piston_window::{EventLoop, PistonWindow, UpdateEvent, WindowSettings};

widget_ids! {
    struct Ids {
        canvas,
        canvas_x_scrollbar,
        canvas_y_scrollbar,
        title,
        button,
        file_address_text_box
    }
}

fn shorten(data: &Vec<i16>, channels: u16, rate: u64) -> Vec<i16> {
    println!("origin size: {} ", data.len(), rate);
    let final_size = (data.len() as u64 * 8000) / (channels as u64 * rate);
    let step_size = rate as f64 / 8000.0;
    let mut short = vec![0i16; final_size as usize];
    let mut index = 0u64;
    let mut last_step = 0u64;
    for i in 0..final_size {
        let mut bit = 0i64;
        let cur_step = ((i + 1) as f64 * step_size) as u64;
        let steps = cur_step - last_step;
        last_step = cur_step;
        for _ in 0..steps {
            for _ in 0..channels {
                bit += data[index as usize] as i64;
                index += 1;
            }
        }
        bit /= steps as i64;
        short[i as usize] = bit as i16;
    }
    return short;
}

fn set_widgets(ui: &mut conrod::UiCell, app: &mut Application, ids: &mut Ids) {
    use conrod::{color, widget, Colorable, Borderable, Labelable, Positionable, Sizeable, Widget};
    let bg_color = color::rgb(0.2, 0.2, 0.2);
    let bt_color = color::rgb(0.3, 0.3, 0.3);
    let tb_color = color::rgb(0.4, 0.4, 0.4);
    // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
    widget::Canvas::new()
        .border(1.0)
        .pad(30.0)
        .color(bg_color)
        .scroll_kids()
        .set(ids.canvas, ui);

    widget::Scrollbar::x_axis(ids.canvas).auto_hide(true).set(ids.canvas_y_scrollbar, ui);
    widget::Scrollbar::y_axis(ids.canvas).auto_hide(true).set(ids.canvas_x_scrollbar, ui);

    // Text example.
    widget::Text::new("Ogg Reducer")
        .top_left_with_margins_on(ids.canvas, 0.0, 10.0)
        .font_size(32)
        .color(bg_color.plain_contrast())
        .set(ids.title, ui);

    for e in widget::TextBox::new(app.text.as_str())
        .w_h(600.0, 50.0)
        .right_from(ids.button, 10.0)
        .color(tb_color)
        .text_color(tb_color.plain_contrast())
        .set(ids.file_address_text_box, ui) {
        match e {
            widget::text_box::Event::Enter => println!("TextBox: {:?}", app.text),
            widget::text_box::Event::Update(string) => app.text = string,
        }
    }

    // Button widget example button.
    if widget::Button::new()
        .w_h(200.0, 50.0)
        .mid_left_of(ids.canvas)
        .down_from(ids.title, 45.0)
        .color(bt_color)
        .label_color(bt_color.plain_contrast())
        .border(1.0)
        .label("Reduce")
        .set(ids.button, ui)
        .was_clicked()
        {
//            use std::io::prelude::*;
            use std::fs::File;

            let f = File::open(&app.text).unwrap();
            let mut decoder = vorbis::Decoder::new(f).unwrap();
            let packets = decoder.packets();
            for p in packets {
                match p {
                    Ok(packet) => println!("{:?}", shorten(&packet.data, packet.channels, packet.rate).len()),
                    _ => {}
                }

            }
//            vorbis::vorbis_sys::
        }
}

struct Application {
    text: String,
}

impl Application {
    fn new() -> Self {
        Application {
            text: String::new()
        }
    }
}

fn main() {
    const WIDTH: u32 = 1100;
    const HEIGHT: u32 = 560;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = piston_window::OpenGL::V4_5;

    // Construct the window.
    let mut window: PistonWindow =
    WindowSettings::new("Ogg reducer", [WIDTH, HEIGHT])
        .opengl(opengl).exit_on_esc(true).vsync(true).build().unwrap();
    let mut ui = conrod::UiBuilder::new().build();
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();
    let mut ids = Ids::new(ui.widget_id_generator());
    let mut text_texture_cache =
    conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);
    let image_map = conrod::image::Map::new();
    window.set_ups(25);
    let mut app = Application::new();
    while let Some(event) = window.next() {
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }
        event.update(|_| {
            let mut ui = ui.set_widgets();
            set_widgets(&mut ui, &mut app, &mut ids);
        });
        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T {
                    img
                };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     &image_map,
                                                     texture_from_image);
            }
        });
    }
}