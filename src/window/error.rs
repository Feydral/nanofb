//! Error types returned by this crate.

use std::fmt;

/// An error returned by [`Window::new`].
#[derive(Debug)]
pub enum WindowError {
    /// The underlying platform window could not be created.
    WindowCreationFailed(String),
    /// No suitable graphics adapter (GPU) was found.
    NoSuitableAdapter(String),
    /// The graphics device could not be created.
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

/// An error returned by [`Window::present`].
///
/// Transient, recoverable GPU surface errors (e.g. a surface becoming
/// briefly outdated during a resize) are handled internally and never
/// surfaced here; only errors that require the caller's attention are
/// returned.
#[derive(Debug)]
pub enum PresentError {
    /// The provided buffer's length didn't match
    /// `buffer_width() * buffer_height()`.
    BufferSizeMismatch { expected: usize, got: usize },
    /// An unrecoverable graphics error occurred (e.g. the GPU device was
    /// lost, or the system ran out of graphics memory).
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

/// An error returned by [`Window::set_cursor_grab`].
#[derive(Debug)]
pub struct CursorGrabError(pub(crate) String);

impl fmt::Display for CursorGrabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to set cursor grab mode: {}", self.0)
    }
}

impl std::error::Error for CursorGrabError {}

/// An error returned by [`Icon::from_rgba`], [`Icon::from_file`],
/// or [`Window::set_icon`].
#[derive(Debug)]
pub struct IconError(pub(crate) String);

impl fmt::Display for IconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to set window icon: {}", self.0)
    }
}

impl std::error::Error for IconError {}
