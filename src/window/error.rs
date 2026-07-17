use std::fmt;

#[derive(Debug)]
pub enum WindowError {
    WindowCreationFailed(String),
    NoSuitableAdapter(String),
    DeviceCreationFailed(String),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowCreationFailed(msg) => write!(f, "failed to create window: {msg}"),
            Self::NoSuitableAdapter(msg) => {
                write!(f, "no suitable graphics adapter was found: {msg}")
            }
            Self::DeviceCreationFailed(msg) => write!(f, "failed to create graphics device: {msg}"),
        }
    }
}

impl std::error::Error for WindowError {}

#[derive(Debug)]
pub enum PresentError {
    BufferSizeMismatch { expected: usize, got: usize },
    Fatal(String),
}

impl fmt::Display for PresentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferSizeMismatch { expected, got } => write!(
                f,
                "buffer size mismatch: expected {expected} pixels, got {got}"
            ),
            Self::Fatal(msg) => write!(f, "fatal graphics error: {msg}"),
        }
    }
}

impl std::error::Error for PresentError {}

#[derive(Debug)]
pub struct CursorGrabError(pub(crate) String);

impl fmt::Display for CursorGrabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to set cursor grab mode: {}", self.0)
    }
}

impl std::error::Error for CursorGrabError {}
