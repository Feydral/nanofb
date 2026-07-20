mod renderer;
mod window;

pub use window::{
    Color32, CursorGrabError, CursorGrabMode, Event, FullscreenMode, Icon, IconError,
    InputSnapshot, Key, MouseButton, PresentError, Window, WindowError, WindowOptions,
};

pub use renderer::{AspectMode, FilterMode};

pub mod prelude {
    pub use crate::{
        AspectMode, Color32, CursorGrabError, CursorGrabMode, Event, FilterMode, FullscreenMode,
        Icon, IconError, InputSnapshot, Key, MouseButton, PresentError, Window, WindowError,
        WindowOptions,
    };
}
