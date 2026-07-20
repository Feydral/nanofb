# nanofb

A minimal pixel-buffer window library for Rust. Open a window, hand it a
`Vec<Color>` once per frame, and it shows up on screen.

## Overview

nanofb gives you the same mental model as libraries like `minifb`: you own
a flat pixel buffer, write to it however you like, and call `present()`
once per frame. Underneath, it's built on `winit` (windowing and input)
and `wgpu` (presentation), but neither of those crates is
ever exposed through the public API — nanofb has its own `Window`,
`Event`, `Key`, and `Icon` types, so you never need to know it's there.

The goal is to stay a thin, predictable wrapper: no game loop, no scene
graph, no asset system. Just a window and a buffer.

## Features

- **Simple presentation model:** Own a `Vec<Color>`, write to it, call
  `window.present(&buffer)`. Nothing else required.
- **Buffer resolution decoupled from window size:** The pixel buffer keeps
  a fixed resolution (`buffer_width`/`buffer_height`) independent of the
  actual window size, so resizing the window never requires resizing the
  buffer. How the buffer fills the window is controlled by `AspectMode`
  (`Stretch`, `AspectFit`, `Center`), with a configurable background color
  for any letterboxed or bordered area.
- **Configurable sampling:** `FilterMode::Nearest` for crisp pixel art, or
  `FilterMode::Linear` for smoothed scaling.
- **Full window control:** Resizable, fullscreen (windowed, borderless, or
  exclusive), maximized, always-on-top, decorations, min/max size, initial
  position — all configurable at creation, with runtime setters for the
  ones that commonly change during a session (fullscreen, maximized,
  title, icon).
- **Events and input:** `poll_events()` drains window-level events (resize,
  close, focus). `input_snapshot()` returns a full keyboard/mouse state
  snapshot, distinguishing keys/buttons that were just pressed, just
  released, or are currently held, plus mouse position, mouse delta, and
  scroll delta.
- **Cursor control:** Show/hide the cursor, or confine/lock it to the
  window for FPS-style camera controls.
- **Window icons:** Build one from raw RGBA pixels, or load it directly
  from an image file (PNG, JPEG, GIF, BMP, and more, via the `image`
  crate).

## How it works

A `Window` owns the platform window and the GPU resources needed to
display a pixel buffer; it does not own the buffer itself. Each frame,
you:

1. Call `window.poll_events()` and `window.input_snapshot()` to pick up
   what happened since the last frame.
2. Write whatever you want into your own `Vec<Color>`.
3. Call `window.present(&buffer)` to upload and display it.

The buffer must always have exactly `buffer_width() * buffer_height()`
elements. Resizing the window doesn't change that requirement — the
buffer is scaled to fit according to the window's `AspectMode`. If you
want the buffer itself to track the window size (e.g. for a fully dynamic
resolution), call `window.set_buffer_size(...)` when you see a
`Event::Resized`.

## Installation

nanofb isn't published on crates.io yet. Until then, depend on it directly
via git or a local path:

```toml
[dependencies]
nanofb = { git = "https://github.com/Feydral/nanofb" }
```

```toml
[dependencies]
nanofb = { path = "../nanofb" }
```

It will be published soon — then it'll simplify to:

```toml
[dependencies]
nanofb = "0.1"
```

## Example

```rust
use nanofb::prelude::*;

fn main() {
    let mut window = Window::new(WindowOptions {
        title: "nanofb example".to_string(),
        width: 800,
        height: 600,
        resizable: true,
        ..Default::default()
    })
    .unwrap();

    let mut buffer =
        vec![Color::BLACK; (window.buffer_width() * window.buffer_height()) as usize];

    while !window.should_close() {
        for event in window.poll_events() {
            if let Event::CloseRequested = event {
                window.close();
            }
        }

        let input = window.input_snapshot();
        if input.is_key_pressed(Key::Escape) {
            window.close();
        }

        for (i, pixel) in buffer.iter_mut().enumerate() {
            let x = (i as u32 % window.buffer_width()) as u8;
            let y = (i as u32 / window.buffer_width()) as u8;
            *pixel = Color::from_rgb(x, y, 128);
        }

        window.present(&buffer).unwrap();
    }
}
```

More examples live in [`examples/`](examples/).

## Project structure

```
src/
  lib.rs          crate root, public re-exports
  window.rs       window creation, the event loop, the Window API
  renderer.rs     wgpu pipeline: uploads the buffer, blits it to the surface
  window/
    color.rs      the Color pixel type
    event.rs      Event, CursorGrabMode
    input.rs      InputSnapshot, Key, MouseButton
    icon.rs       Icon (window icon loading)
    error.rs      error types
```

`window.rs` and `renderer.rs` are deliberately separate: `window.rs` only
knows about `winit` (platform window, input events), `renderer.rs` only
knows about `wgpu` (GPU resources, the blit pipeline). Neither leaks into
the other's concerns.

## Platform notes

As with most windowing libraries, `Window::new` and every method on
`Window` must be called from your program's main thread.

## License

MIT