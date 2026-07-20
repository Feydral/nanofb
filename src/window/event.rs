//! Window-level events, as reported by [`Window::poll_events`].

/// A window-level event, as reported by [`Window::poll_events`].
///
/// For keyboard and mouse state, use [`Window::input_snapshot`]
/// instead; this enum only covers things that happen to the window itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// The window was resized. The new size is given in physical pixels.
    Resized { width: u32, height: u32 },
    /// The user requested the window be closed (e.g. clicked the close
    /// button). This doesn't close the window by itself; call
    /// [`Window::close`] to request that your loop exit, or check
    /// [`Window::should_close`].
    CloseRequested,
    /// The window gained or lost keyboard focus.
    Focused(bool),
}

/// Controls how the cursor is confined or locked to the window. See
/// [`Window::set_cursor_grab`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CursorGrabMode {
    /// The cursor is free to move anywhere, as normal.
    #[default]
    None,
    /// The cursor is kept within the window's bounds, but still
    /// moves and reports its absolute position normally.
    Confined,
    /// The cursor's position is fixed and its absolute position no
    /// longer changes; use [`InputSnapshot::mouse_delta`] for movement
    /// instead. Useful for FPS-style camera controls.
    Locked,
}
