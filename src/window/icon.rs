use std::path::Path;

use crate::window::error::IconError;

/// Window icon data (RGBA8, one byte per channel).
///
/// Unlike [`Color`], icons carry an alpha channel, since taskbar
/// and title bar icons are usually non-rectangular (e.g. a circular logo
/// on a transparent square canvas). Build one with [`Icon::from_rgba`] or
/// [`Icon::from_file`], then pass it to [`Window::set_icon`].
#[derive(Debug, Clone)]
pub struct Icon {
    pub(crate) rgba: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Icon {
    /// Builds an icon from raw RGBA8 pixels (4 bytes per pixel, `[r, g, b, a]`).
    /// `rgba.len()` must equal `width * height * 4`.
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, IconError> {
        let expected = (width as usize) * (height as usize) * 4;
        if rgba.len() != expected {
            return Err(IconError(format!(
                "expected {expected} bytes for a {width}x{height} RGBA icon, got {}",
                rgba.len()
            )));
        }
        Ok(Self {
            rgba,
            width,
            height,
        })
    }

    /// Loads an icon from an image file (PNG, JPEG, GIF, BMP, and more;
    /// format is guessed from the file's contents/extension).
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, IconError> {
        let image = image::open(path.as_ref())
            .map_err(|e| IconError(e.to_string()))?
            .into_rgba8();
        let (width, height) = image.dimensions();
        Ok(Self {
            rgba: image.into_raw(),
            width,
            height,
        })
    }

    /// The icon's width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The icon's height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}
