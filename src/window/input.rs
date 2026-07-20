//! Keyboard and mouse input state, as reported by [`Window::input_snapshot`].

use std::collections::HashSet;

/// A snapshot of keyboard and mouse state, as returned by
/// [`Window::input_snapshot`].
///
/// Distinguishes three kinds of key/button state, each useful for
/// different things:
/// - "pressed": true only on the frame the key/button went down.
/// - "released": true only on the frame the key/button went up.
/// - "held": true for as long as the key/button stays down.
///
/// Taking a new snapshot resets "pressed"/"released" to whatever changed
/// since the previous snapshot, and resets `mouse_delta`/`scroll_delta`
/// back to zero.
#[derive(Debug, Clone)]
pub struct InputSnapshot {
    pub(crate) held_keys: HashSet<Key>,
    pub(crate) pressed_keys: HashSet<Key>,
    pub(crate) released_keys: HashSet<Key>,
    pub(crate) held_mouse_buttons: HashSet<MouseButton>,
    pub(crate) pressed_mouse_buttons: HashSet<MouseButton>,
    pub(crate) released_mouse_buttons: HashSet<MouseButton>,
    pub(crate) mouse_position: (f32, f32),
    pub(crate) mouse_delta: (f32, f32),
    pub(crate) scroll_delta: (f32, f32),
}

impl InputSnapshot {
    /// Whether `key` was pressed down on this frame. Fires exactly once
    /// per press, regardless of how long the key is then held.
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Whether `key` was released on this frame. Fires exactly once per
    /// release.
    pub fn is_key_released(&self, key: Key) -> bool {
        self.released_keys.contains(&key)
    }

    /// Whether `key` is currently held down, regardless of when it was
    /// first pressed.
    pub fn is_key_held(&self, key: Key) -> bool {
        self.held_keys.contains(&key)
    }

    /// Whether `button` was pressed down on this frame.
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    /// Whether `button` was released on this frame.
    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.released_mouse_buttons.contains(&button)
    }

    /// Whether `button` is currently held down.
    pub fn is_mouse_button_held(&self, button: MouseButton) -> bool {
        self.held_mouse_buttons.contains(&button)
    }

    /// The cursor's current position, in physical pixels relative to the
    /// window's top-left corner.
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    /// Raw, unaccelerated mouse motion accumulated since the previous
    /// snapshot. Keeps working even while the cursor is locked via
    /// [`CursorGrabMode::Locked`], unlike `mouse_position`.
    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    /// Scroll wheel movement accumulated since the previous snapshot.
    pub fn scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }
}

/// A keyboard key, identified by its physical position on the keyboard
/// (layout-independent, e.g. `Key::W` is always the key labeled W on a
/// QWERTY keyboard, whatever letter it's labeled with elsewhere).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[rustfmt::skip]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Escape,
    Space,
    Enter,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
    LeftSuper,
    RightSuper,
    CapsLock,
    Comma,
    Period,
    Slash,
    Semicolon,
    Apostrophe,
    LeftBracket,
    RightBracket,
    Backslash,
    Minus,
    Equals,
    Grave,
    /// A key not covered by any other variant.
    Unknown,
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    /// A platform-specific extra button, identified by its raw id.
    Other(u16),
}
