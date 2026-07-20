use nanofb::prelude::*;

const BUFFER_WIDTH: u32 = 320;
const BUFFER_HEIGHT: u32 = 180;

fn main() {
    let window_opts = WindowOptions {
        title: "Resizable Stretch Example".to_string(),
        width: 1600,
        height: 900,
        resizable: true,
        filter_mode: FilterMode::Nearest,
        buffer_width: BUFFER_WIDTH,
        buffer_height: BUFFER_HEIGHT,
        aspect_mode: AspectMode::Stretch,
        ..Default::default()
    };

    let icon = Icon::from_file("example/path/icon.png").unwrap();

    let mut window = Window::new(window_opts).unwrap();
    window.set_icon(icon).unwrap();
    window.set_background_color(Color32::from_rgb(128, 128, 255));
    window.set_cursor_visible(false);

    let mut buffer = vec![Color32::BLACK; (BUFFER_WIDTH * BUFFER_HEIGHT) as usize];

    while !window.should_close() {
        for event in window.poll_events() {
            if let Event::CloseRequested = event {
                window.close();
            }
        }

        for (idx, pixel) in buffer.iter_mut().enumerate() {
            let x = (idx as u32 % BUFFER_WIDTH) as u8;
            let y = (idx as u32 / BUFFER_WIDTH) as u8;
            *pixel = Color32::from_rgb(x.wrapping_mul(2), y.wrapping_mul(2), 128);
        }

        window.present(&buffer).unwrap();
    }
}
