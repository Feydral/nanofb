#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Resized { width: u32, height: u32 },
    CloseRequested,
    Focused(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CursorGrabMode {
    #[default]
    None,
    Confined,
    Locked,
}
