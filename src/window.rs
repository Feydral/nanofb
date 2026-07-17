pub(crate) mod color;
pub(crate) mod error;
mod event;

pub use color::Color32;
pub use error::{CursorGrabError, PresentError, WindowError};
pub use event::{CursorGrabMode, Event, Key, MouseButton};

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
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{
    CursorGrabMode as WinitCursorGrabMode, Fullscreen as WinitFullscreen, Window as WinitWindow,
    WindowAttributes, WindowId, WindowLevel,
};

use crate::renderer::{FilterMode, Renderer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FullscreenMode {
    #[default]
    Windowed,
    Borderless,
    Exclusive,
}

#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub transparent: bool,
    pub maximized: bool,
    pub fullscreen: FullscreenMode,
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub position: Option<(i32, i32)>,
    pub visible: bool,
    pub filter_mode: FilterMode,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "nanofb".to_string(),
            width: 854,
            height: 480,
            resizable: true,
            decorations: true,
            always_on_top: false,
            transparent: false,
            maximized: false,
            fullscreen: FullscreenMode::Windowed,
            min_size: None,
            max_size: None,
            position: None,
            visible: true,
            filter_mode: FilterMode::Nearest,
        }
    }
}

struct AppHandler {
    attributes: Option<WindowAttributes>,
    fullscreen: FullscreenMode,
    window: Option<Arc<WinitWindow>>,
    events: Vec<Event>,
    should_close: bool,
    focused: bool,
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut attributes = self.attributes.take().unwrap_or_default();

        if self.fullscreen != FullscreenMode::Windowed {
            let monitor = event_loop.primary_monitor();
            attributes.fullscreen = match self.fullscreen {
                FullscreenMode::Borderless => Some(WinitFullscreen::Borderless(monitor)),
                FullscreenMode::Exclusive => monitor
                    .and_then(|m| m.video_modes().next())
                    .map(WinitFullscreen::Exclusive),
                FullscreenMode::Windowed => None,
            };
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
                self.events.push(Event::Focused(f));
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    let key = map_key(code);
                    self.events.push(match key_event.state {
                        ElementState::Pressed => Event::KeyDown(key),
                        ElementState::Released => Event::KeyUp(key),
                    });
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.events.push(Event::MouseMoved {
                    x: position.x as f32,
                    y: position.y as f32,
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = map_mouse_button(button);
                self.events.push(match state {
                    ElementState::Pressed => Event::MouseButtonDown(button),
                    ElementState::Released => Event::MouseButtonUp(button),
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.events.push(Event::Scroll { dx, dy });
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
            self.events.push(Event::MouseDelta {
                dx: delta.0 as f32,
                dy: delta.1 as f32,
            });
        }
    }
}

pub struct Window {
    event_loop: EventLoop<()>,
    app: AppHandler,
    renderer: Renderer,
}

impl Window {
    pub fn new(options: WindowOptions) -> Result<Self, WindowError> {
        let mut event_loop =
            EventLoop::new().map_err(|e| WindowError::WindowCreationFailed(e.to_string()))?;

        let mut attributes = WinitWindow::default_attributes()
            .with_title(options.title.clone())
            .with_inner_size(PhysicalSize::new(options.width, options.height))
            .with_resizable(options.resizable)
            .with_decorations(options.decorations)
            .with_transparent(options.transparent)
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
        let renderer = Renderer::new(winit_window, options.filter_mode, options.transparent)?;

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

    pub fn poll_events(&mut self) -> Vec<Event> {
        let _ = self
            .event_loop
            .pump_app_events(Some(Duration::ZERO), &mut self.app);
        std::mem::take(&mut self.app.events)
    }

    pub fn present(&mut self, buffer: &[Color32]) -> Result<(), PresentError> {
        let size = self.winit_window().inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let expected = (width * height) as usize;

        if buffer.len() != expected {
            return Err(PresentError::BufferSizeMismatch {
                expected,
                got: buffer.len(),
            });
        }

        self.renderer.resize(width, height);
        self.renderer.present(buffer)
    }

    pub fn width(&self) -> u32 {
        self.winit_window().inner_size().width.max(1)
    }

    pub fn height(&self) -> u32 {
        self.winit_window().inner_size().height.max(1)
    }

    pub fn is_focused(&self) -> bool {
        self.app.focused
    }

    pub fn set_title(&mut self, title: &str) {
        self.winit_window().set_title(title);
    }

    pub fn should_close(&self) -> bool {
        self.app.should_close
    }

    pub fn request_close(&mut self) {
        self.app.should_close = true;
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.winit_window().set_cursor_visible(visible);
    }

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
