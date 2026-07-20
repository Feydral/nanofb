//! Window creation, the event loop, and the public [`Window`] API. This is
//! the only module users of the crate interact with directly.

mod color;
mod error;
mod event;
mod icon;
mod input;

pub use color::Color;
pub use error::{CursorGrabError, IconError, PresentError, WindowError};
pub use event::{CursorGrabMode, Event};
pub use icon::Icon;
pub use input::{InputSnapshot, Key, MouseButton};

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    DeviceEvent, DeviceId, ElementState, MouseButton as WinitMouseButton, MouseScrollDelta,
    WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::monitor::MonitorHandle;
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{
    CursorGrabMode as WinitCursorGrabMode, Fullscreen as WinitFullscreen, Window as WinitWindow,
    WindowAttributes, WindowId, WindowLevel,
};

use crate::renderer::Renderer;
use crate::renderer::{AspectMode, FilterMode};

/// How the window should occupy the screen. See
/// [`Window::set_fullscreen`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FullscreenMode {
    /// A normal, windowed presentation.
    #[default]
    Windowed,
    /// Fullscreen without a video mode change; the window simply expands
    /// to cover the screen. Works well on all platforms and switches
    /// quickly.
    Borderless,
    /// Exclusive fullscreen, taking over the display's video mode
    /// directly. Slower to enter/exit and less portable than
    /// `Borderless`, but may offer lower input latency.
    Exclusive,
}

/// Configuration used to create a [`Window`].
///
/// Construct with [`WindowOptions::default`] and override only the fields
/// you care about:
///
/// ```no_run
/// use nanofb::prelude::*;
///
/// let options = WindowOptions {
///     title: "My Game".to_string(),
///     width: 800,
///     height: 600,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct WindowOptions {
    /// Text shown in the window's title bar.
    pub title: String,
    /// Initial window width, in physical pixels.
    pub width: u32,
    /// Initial window height, in physical pixels.
    pub height: u32,
    /// Whether the user can resize the window by dragging its edges.
    pub resizable: bool,
    /// Whether the window has a title bar and border.
    pub decorations: bool,
    /// Whether the window should stay above other windows.
    pub always_on_top: bool,
    /// Whether the window starts maximized.
    pub maximized: bool,
    /// Whether, and how, the window starts in fullscreen. See
    /// [`FullscreenMode`].
    pub fullscreen: FullscreenMode,
    /// Smallest size the user is allowed to resize the window to, in
    /// physical pixels. `None` means no minimum.
    pub min_size: Option<(u32, u32)>,
    /// Largest size the user is allowed to resize the window to, in
    /// physical pixels. `None` means no maximum.
    pub max_size: Option<(u32, u32)>,
    /// Initial position of the window's top-left corner, in physical
    /// screen pixels. `None` lets the platform choose.
    pub position: Option<(i32, i32)>,
    /// Whether the window is shown immediately on creation.
    pub visible: bool,
    /// Filtering used when presenting the pixel buffer. Defaults to
    /// [`FilterMode::Nearest`] for crisp pixels.
    pub filter_mode: FilterMode,
    /// Initial pixel buffer width. Independent of `width`: the window can
    /// be resized freely without needing a matching buffer resize, since
    /// the buffer is scaled to the window according to `aspect_mode`.
    /// Change it later with [`Window::set_buffer_size`].
    pub buffer_width: u32,
    /// Initial pixel buffer height. See `buffer_width`.
    pub buffer_height: u32,
    /// How the pixel buffer is scaled to fill the window when their sizes
    /// or aspect ratios don't match. Change it later with
    /// [`Window::set_aspect_mode`].
    pub aspect_mode: AspectMode,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "nanofb".to_string(),
            width: 640,
            height: 360,
            resizable: true,
            decorations: true,
            always_on_top: false,
            maximized: false,
            fullscreen: FullscreenMode::Windowed,
            min_size: None,
            max_size: None,
            position: None,
            visible: true,
            filter_mode: FilterMode::Nearest,
            buffer_width: 640,
            buffer_height: 360,
            aspect_mode: AspectMode::default(),
        }
    }
}

fn build_fullscreen(
    mode: FullscreenMode,
    monitor: Option<MonitorHandle>,
) -> Option<WinitFullscreen> {
    match mode {
        FullscreenMode::Windowed => None,
        FullscreenMode::Borderless => Some(WinitFullscreen::Borderless(monitor)),
        FullscreenMode::Exclusive => monitor
            .and_then(|m| m.video_modes().next())
            .map(WinitFullscreen::Exclusive),
    }
}

struct AppHandler {
    attributes: Option<WindowAttributes>,
    fullscreen: FullscreenMode,
    window: Option<Arc<WinitWindow>>,
    events: Vec<Event>,
    should_close: bool,
    focused: bool,

    held_keys: HashSet<Key>,
    pressed_keys: HashSet<Key>,
    released_keys: HashSet<Key>,
    held_mouse_buttons: HashSet<MouseButton>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    released_mouse_buttons: HashSet<MouseButton>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    scroll_delta: (f32, f32),
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut attributes = self.attributes.take().unwrap_or_default();

        if self.fullscreen != FullscreenMode::Windowed {
            attributes.fullscreen = build_fullscreen(self.fullscreen, event_loop.primary_monitor());
        }

        let window = event_loop
            .create_window(attributes)
            .expect("nanofb: failed to create window");
        self.window = Some(Arc::new(window));
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.should_close = true;
                self.events.push(Event::CloseRequested);
            }
            WindowEvent::Resized(size) => {
                self.events.push(Event::Resized {
                    width: size.width,
                    height: size.height,
                });
            }
            WindowEvent::Focused(f) => {
                self.focused = f;
                if !f {
                    self.held_keys.clear();
                    self.held_mouse_buttons.clear();
                }
                self.events.push(Event::Focused(f));
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    let key = map_key(code);
                    match key_event.state {
                        ElementState::Pressed => {
                            if !key_event.repeat {
                                self.pressed_keys.insert(key);
                            }
                            self.held_keys.insert(key);
                        }
                        ElementState::Released => {
                            self.held_keys.remove(&key);
                            self.released_keys.insert(key);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);
                self.mouse_position = (x, y);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = map_mouse_button(button);
                match state {
                    ElementState::Pressed => {
                        self.held_mouse_buttons.insert(button);
                        self.pressed_mouse_buttons.insert(button);
                    }
                    ElementState::Released => {
                        self.held_mouse_buttons.remove(&button);
                        self.released_mouse_buttons.insert(button);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.scroll_delta.0 += dx;
                self.scroll_delta.1 += dy;
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let (dx, dy) = (delta.0 as f32, delta.1 as f32);
            self.mouse_delta.0 += dx;
            self.mouse_delta.1 += dy;
        }
    }
}

/// A window that a raw RGB pixel buffer can be presented to.
///
/// `Window` owns the platform window and drives the renderer; it does not
/// own the pixel buffer itself. Create your own `Vec<Color>`, write to
/// it however you like, and hand a reference to [`Window::present`] once
/// per frame.
pub struct Window {
    event_loop: EventLoop<()>,
    app: AppHandler,
    renderer: Renderer,
}

impl Window {
    /// Creates a new window and initializes the graphics context. Must be
    /// called from your program's main thread.
    pub fn new(options: WindowOptions) -> Result<Self, WindowError> {
        let mut event_loop =
            EventLoop::new().map_err(|e| WindowError::WindowCreationFailed(e.to_string()))?;

        let mut attributes = WinitWindow::default_attributes()
            .with_title(options.title.clone())
            .with_inner_size(PhysicalSize::new(options.width, options.height))
            .with_resizable(options.resizable)
            .with_decorations(options.decorations)
            .with_maximized(options.maximized)
            .with_visible(options.visible);

        if options.always_on_top {
            attributes = attributes.with_window_level(WindowLevel::AlwaysOnTop);
        }
        if let Some((w, h)) = options.min_size {
            attributes = attributes.with_min_inner_size(PhysicalSize::new(w, h));
        }
        if let Some((w, h)) = options.max_size {
            attributes = attributes.with_max_inner_size(PhysicalSize::new(w, h));
        }
        if let Some((x, y)) = options.position {
            attributes = attributes.with_position(PhysicalPosition::new(x, y));
        }

        let mut app = AppHandler {
            attributes: Some(attributes),
            fullscreen: options.fullscreen,
            window: None,
            events: Vec::new(),
            should_close: false,
            focused: false,
            held_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),
            held_mouse_buttons: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            released_mouse_buttons: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            scroll_delta: (0.0, 0.0),
        };

        loop {
            let status = event_loop.pump_app_events(Some(Duration::from_millis(50)), &mut app);
            if app.window.is_some() {
                break;
            }
            if let PumpStatus::Exit(_) = status {
                return Err(WindowError::WindowCreationFailed(
                    "event loop exited before the window was created".to_string(),
                ));
            }
        }
        app.events.clear();

        let winit_window = app.window.clone().expect("window created above");
        let renderer = Renderer::new(
            winit_window,
            options.filter_mode,
            options.buffer_width,
            options.buffer_height,
            options.aspect_mode,
        )?;

        Ok(Self {
            event_loop,
            app,
            renderer,
        })
    }

    fn winit_window(&self) -> &Arc<WinitWindow> {
        self.app
            .window
            .as_ref()
            .expect("window is always present after Window::new")
    }

    fn pump(&mut self) {
        let _ = self
            .event_loop
            .pump_app_events(Some(Duration::ZERO), &mut self.app);
    }

    /// Polls and drains all pending window events since the last call.
    ///
    /// Never blocks. Call it once per frame, typically at the top of your
    /// loop.
    pub fn poll_events(&mut self) -> Vec<Event> {
        self.pump();
        std::mem::take(&mut self.app.events)
    }

    /// Takes a snapshot of the current keyboard and mouse state.
    ///
    /// Call this once per frame, typically alongside [`Window::poll_events`].
    /// See [`InputSnapshot`] for what it captures and how
    /// pressed/released/held differ.
    pub fn input_snapshot(&mut self) -> InputSnapshot {
        self.pump();
        InputSnapshot {
            held_keys: self.app.held_keys.clone(),
            pressed_keys: std::mem::take(&mut self.app.pressed_keys),
            released_keys: std::mem::take(&mut self.app.released_keys),
            held_mouse_buttons: self.app.held_mouse_buttons.clone(),
            pressed_mouse_buttons: std::mem::take(&mut self.app.pressed_mouse_buttons),
            released_mouse_buttons: std::mem::take(&mut self.app.released_mouse_buttons),
            mouse_position: self.app.mouse_position,
            mouse_delta: std::mem::take(&mut self.app.mouse_delta),
            scroll_delta: std::mem::take(&mut self.app.scroll_delta),
        }
    }

    /// Uploads `buffer` and presents it to the window.
    ///
    /// `buffer` must have exactly `buffer_width() * buffer_height()`
    /// elements. This is independent of the window's actual size: resizing
    /// the window does not require resizing `buffer`, since it's scaled to
    /// fit according to [`Window::set_aspect_mode`].
    ///
    /// Transient, recoverable surface errors (e.g. a surface briefly
    /// outdated during a resize) are retried internally; only errors that
    /// require the caller's attention are returned.
    pub fn present(&mut self, buffer: &[Color]) -> Result<(), PresentError> {
        let size = self.winit_window().inner_size();
        self.renderer.resize(size.width, size.height);
        self.renderer.present(buffer)
    }

    /// The pixel buffer width [`Window::present`] currently expects.
    pub fn buffer_width(&self) -> u32 {
        self.renderer.buffer_width()
    }

    /// The pixel buffer height [`Window::present`] currently expects.
    pub fn buffer_height(&self) -> u32 {
        self.renderer.buffer_height()
    }

    /// Changes the pixel buffer resolution. Takes effect on the next call
    /// to [`Window::present`], which will then expect a buffer of the new
    /// size.
    pub fn set_buffer_size(&mut self, width: u32, height: u32) {
        self.renderer.set_buffer_size(width, height);
    }

    /// Changes how the pixel buffer is scaled to fill the window.
    pub fn set_aspect_mode(&mut self, mode: AspectMode) {
        self.renderer.set_aspect_mode(mode);
    }

    /// Changes the color used to fill the area around the pixel buffer
    /// when it doesn't cover the whole window (letterboxing in
    /// [`AspectMode::AspectFit`], or the border in [`AspectMode::Center`]).
    /// Defaults to opaque black.
    pub fn set_background_color(&mut self, color: Color) {
        self.renderer.set_background_color(color);
    }

    /// The window's current width in physical pixels.
    pub fn width(&self) -> u32 {
        self.winit_window().inner_size().width.max(1)
    }

    /// The window's current height in physical pixels.
    pub fn height(&self) -> u32 {
        self.winit_window().inner_size().height.max(1)
    }

    /// Whether the window currently has keyboard focus.
    pub fn is_focused(&self) -> bool {
        self.app.focused
    }

    /// Changes the window's title.
    pub fn set_title(&mut self, title: &str) {
        self.winit_window().set_title(title);
    }

    /// The window's current fullscreen state.
    pub fn fullscreen_mode(&self) -> FullscreenMode {
        match self.winit_window().fullscreen() {
            None => FullscreenMode::Windowed,
            Some(WinitFullscreen::Borderless(_)) => FullscreenMode::Borderless,
            Some(WinitFullscreen::Exclusive(_)) => FullscreenMode::Exclusive,
        }
    }

    /// Switches the window between windowed and fullscreen presentation.
    /// See [`FullscreenMode`] for the available modes.
    pub fn set_fullscreen(&mut self, mode: FullscreenMode) {
        let monitor = self.winit_window().current_monitor();
        let fullscreen = build_fullscreen(mode, monitor);
        self.winit_window().set_fullscreen(fullscreen);
    }

    /// Maximizes or restores the window.
    pub fn set_maximized(&mut self, maximized: bool) {
        self.winit_window().set_maximized(maximized);
    }

    /// Sets the window's title bar / taskbar icon. Not supported on all
    /// platforms (e.g. ignored on macOS, which uses the app bundle icon).
    pub fn set_icon(&mut self, icon: Icon) -> Result<(), IconError> {
        let winit_icon = winit::window::Icon::from_rgba(icon.rgba, icon.width, icon.height)
            .map_err(|e| IconError(e.to_string()))?;
        self.winit_window().set_window_icon(Some(winit_icon));
        Ok(())
    }

    /// Removes the window icon set by [`Window::set_icon`], reverting to
    /// the platform default.
    pub fn clear_icon(&mut self) {
        self.winit_window().set_window_icon(None);
    }

    /// Whether the window has been requested to close, either by the user
    /// (e.g. clicking the close button) or by a call to [`Window::close`].
    pub fn should_close(&self) -> bool {
        self.app.should_close
    }

    /// Requests that the window close: sets a flag checked by
    /// [`Window::should_close`], which your loop is expected to act on
    /// (e.g. `break`). This alone doesn't destroy the window or free its
    /// graphics resources; that happens automatically once `Window` is
    /// dropped, typically when it goes out of scope at the end of `main`.
    pub fn close(&mut self) {
        self.app.should_close = true;
    }

    /// Shows or hides the mouse cursor.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.winit_window().set_cursor_visible(visible);
    }

    /// Confines or locks the cursor to the window. See [`CursorGrabMode`]
    /// for the difference between confining and locking.
    pub fn set_cursor_grab(&mut self, mode: CursorGrabMode) -> Result<(), CursorGrabError> {
        let winit_mode = match mode {
            CursorGrabMode::None => WinitCursorGrabMode::None,
            CursorGrabMode::Confined => WinitCursorGrabMode::Confined,
            CursorGrabMode::Locked => WinitCursorGrabMode::Locked,
        };
        self.winit_window()
            .set_cursor_grab(winit_mode)
            .map_err(|e| CursorGrabError(e.to_string()))
    }
}

fn map_mouse_button(button: WinitMouseButton) -> MouseButton {
    match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Other(u16::MAX - 1),
        WinitMouseButton::Forward => MouseButton::Other(u16::MAX),
        WinitMouseButton::Other(id) => MouseButton::Other(id),
    }
}

fn map_key(code: KeyCode) -> Key {
    match code {
        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,
        KeyCode::Digit0 => Key::Key0,
        KeyCode::Digit1 => Key::Key1,
        KeyCode::Digit2 => Key::Key2,
        KeyCode::Digit3 => Key::Key3,
        KeyCode::Digit4 => Key::Key4,
        KeyCode::Digit5 => Key::Key5,
        KeyCode::Digit6 => Key::Key6,
        KeyCode::Digit7 => Key::Key7,
        KeyCode::Digit8 => Key::Key8,
        KeyCode::Digit9 => Key::Key9,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::Escape => Key::Escape,
        KeyCode::Space => Key::Space,
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Insert => Key::Insert,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::ShiftLeft => Key::LeftShift,
        KeyCode::ShiftRight => Key::RightShift,
        KeyCode::ControlLeft => Key::LeftControl,
        KeyCode::ControlRight => Key::RightControl,
        KeyCode::AltLeft => Key::LeftAlt,
        KeyCode::AltRight => Key::RightAlt,
        KeyCode::SuperLeft => Key::LeftSuper,
        KeyCode::SuperRight => Key::RightSuper,
        KeyCode::CapsLock => Key::CapsLock,
        KeyCode::Comma => Key::Comma,
        KeyCode::Period => Key::Period,
        KeyCode::Slash => Key::Slash,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Quote => Key::Apostrophe,
        KeyCode::BracketLeft => Key::LeftBracket,
        KeyCode::BracketRight => Key::RightBracket,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::Minus => Key::Minus,
        KeyCode::Equal => Key::Equals,
        KeyCode::Backquote => Key::Grave,
        _ => Key::Unknown,
    }
}
