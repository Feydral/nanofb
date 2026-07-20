//! nanofb is a minimal pixel-buffer window library: open a window, hand it
//! a `Vec<Color>` once per frame, and it shows up on screen. Built on
//! `winit` and `wgpu`, but neither is exposed through the public API.
//!
//! # Example
//!
//! ```no_run
//! use nanofb::prelude::*;
//!
//! fn main() {
//!     let mut window = Window::new(WindowOptions::default()).unwrap();
//!     let mut buffer =
//!         vec![Color::BLACK; (window.buffer_width() * window.buffer_height()) as usize];
//!
//!     while !window.should_close() {
//!         for event in window.poll_events() {
//!             if let Event::CloseRequested = event {
//!                 window.close();
//!             }
//!         }
//!
//!         // ... write to `buffer` ...
//!
//!         window.present(&buffer).unwrap();
//!     }
//! }
//! ```
//!
//! # Design
//!
//! - The pixel buffer is owned by the caller, not by [`Window`]: keep a
//!   `Vec<Color>` and hand a reference to it to [`Window::present`]
//!   every frame.
//! - [`Window::present`] requires the buffer to have exactly
//!   `buffer_width() * buffer_height()` elements. This is independent of
//!   the window's actual size: the buffer is scaled to fit according to
//!   [`AspectMode`], so resizing the window never requires resizing the
//!   buffer. Change the buffer resolution itself with
//!   [`Window::set_buffer_size`].
//! - [`Color`] pixels are packed as `0x00BBGGRR` in memory (bytes
//!   `[r, g, b, 0]` on little-endian platforms), matching
//!   `wgpu::TextureFormat::Rgba8Unorm`'s byte order exactly, so buffers
//!   upload with a plain byte-cast, no per-pixel conversion.
//! - Input is polled, not pushed: [`Window::poll_events`] drains
//!   window-level events (resize, close, focus), and
//!   [`Window::input_snapshot`] returns current keyboard/mouse state.
//!   Neither blocks.
//!
//! # Platform notes
//!
//! As with most windowing libraries, [`Window::new`] and every method on
//! [`Window`] must be called from your program's main thread.
//!
//! # Module layout
//!
//! The public API is re-exported at the crate root (and via [`prelude`]),
//! but internally is organized by concern:
//! - `color` - the [`Color`] pixel type.
//! - `event` - [`Event`], [`CursorGrabMode`].
//! - `input` - [`InputSnapshot`], [`Key`], [`MouseButton`].
//! - `icon` - [`Icon`].
//! - `error` - error types returned by this crate.
//! - `renderer` - the wgpu-specific rendering backend (private, not part
//!   of the public API beyond [`AspectMode`]/[`FilterMode`]).
//! - the top-level `window` module - window creation, the event loop, and
//!   the [`Window`] type itself.

mod renderer;
mod window;

pub use window::{
    Color, CursorGrabError, CursorGrabMode, Event, FullscreenMode, Icon, IconError,
    InputSnapshot, Key, MouseButton, PresentError, Window, WindowError, WindowOptions,
};

pub use renderer::{AspectMode, FilterMode};

/// Re-exports every public type in one place.
///
/// ```
/// use nanofb::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        AspectMode, Color, CursorGrabError, CursorGrabMode, Event, FilterMode, FullscreenMode,
        Icon, IconError, InputSnapshot, Key, MouseButton, PresentError, Window, WindowError,
        WindowOptions,
    };
}