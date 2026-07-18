use nanofb::prelude::*;

const BUFFER_SIZE: u32 = 128;

fn main() {
    let window_opts = WindowOptions {
        title: "Resizable Example".to_string(),
        width: 1600,
        height: 900,
        resizable: true,
        filter_mode: FilterMode::Nearest,
        buffer_width: BUFFER_SIZE,
        buffer_height: BUFFER_SIZE,
        aspect_mode: AspectMode::AspectFit,
        fullscreen: FullscreenMode::Windowed,
        maximized: true,
        ..Default::default()
    };

    let mut window = Window::new(window_opts).unwrap();
    let mut buffer = vec![Color32::BLACK; (BUFFER_SIZE * BUFFER_SIZE) as usize];

    while !window.should_close() {
        for event in window.poll_events() {
            match event {
                Event::CloseRequested => window.request_close(),
                _ => {}
            }
        }

        for (idx, pixel) in buffer.iter_mut().enumerate() {
            let x = (idx as u32 % BUFFER_SIZE) as u8;
            let y = (idx as u32 / BUFFER_SIZE) as u8;
            *pixel = Color32::from_rgb(x.wrapping_mul(2), y.wrapping_mul(2), 128);
        }

        window.present(&buffer).unwrap();
    }
}
