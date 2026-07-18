use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Resized { width: u32, height: u32 },
    CloseRequested,
    Focused(bool),
}

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
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CursorGrabMode {
    #[default]
    None,
    Confined,
    Locked,
}

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
    pub fn is_key_down(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_key_up(&self, key: Key) -> bool {
        self.released_keys.contains(&key)
    }

    pub fn is_key_held(&self, key: Key) -> bool {
        self.held_keys.contains(&key)
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    pub fn is_mouse_button_up(&self, button: MouseButton) -> bool {
        self.released_mouse_buttons.contains(&button)
    }

    pub fn is_mouse_button_held(&self, button: MouseButton) -> bool {
        self.held_mouse_buttons.contains(&button)
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    pub fn scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }
}
