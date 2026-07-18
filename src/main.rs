use nanofb::prelude::*;

fn main() {
    let window_opts = WindowOptions {
        title: "Resizable Example".to_string(),
        width: 1600,
        height: 900,
        resizable: true,
        filter_mode: FilterMode::Linear,
        buffer_width: 1600,
        buffer_height: 900,
        aspect_mode: AspectMode::AspectFit,
        fullscreen: FullscreenMode::Windowed,
        ..Default::default()
    };

    let mut window = Window::new(window_opts).unwrap();
    let mut buffer = vec![Color32::BLACK; (1600 * 900) as usize];

    while !window.should_close() {
        for event in window.poll_events() {
            match event {
                Event::CloseRequested => window.request_close(),
                Event::Resized { width, height } => {
                    buffer.resize((width * height) as usize, Color32::BLACK);
                    window.set_buffer_size(width, height);
                }
                _ => {}
            }
        }

        for (idx, pixel) in buffer.iter_mut().enumerate() {
            let x = (idx as u32 % 256) as u8;
            let y = (idx as u32 / 256) as u8;
            *pixel = Color32::from_rgb(x.wrapping_mul(2), y.wrapping_mul(2), 128);
        }

        window.present(&buffer).unwrap();
    }
}
