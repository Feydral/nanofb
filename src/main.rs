use nanofb::prelude::*;

fn main() {
    let window_opts = WindowOptions {
        title: "Example".to_string(),
        width: 1600,
        height: 900,
        resizable: false,
        filter_mode: FilterMode::Linear,
        ..Default::default()
    };
    let mut window = Window::new(window_opts).unwrap();

    let mut buffer = vec![Color32::BLACK; 1600 * 900];

    while !window.should_close() {
        for event in window.poll_events() {
            if let Event::CloseRequested = event {
                window.request_close();
            }
        }

        buffer
            .iter_mut()
            .enumerate()
            .for_each(|(idx, pixel)| *pixel = Color32::from_rgb(idx as u8, idx as u8, idx as u8));

        window.present(&buffer).unwrap();
    }
}
