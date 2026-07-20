//! The [`Color`] type used for buffers passed to [`Window::present`].

/// An opaque 32-bit RGB color, packed for zero-cost upload to the GPU.
///
/// The bytes are laid out as `[r, g, b, 0]` in memory on little-endian
/// platforms, matching `wgpu::TextureFormat::Rgba8Unorm`'s byte order. The
/// top byte is always zero and unused (windows are always opaque), which
/// means a `&[Color]` buffer can be uploaded to the GPU with a plain
/// byte-cast, no per-pixel conversion needed.
///
/// `Color` is `#[repr(transparent)]` over `u32`, so it has identical
/// size, alignment, and bit layout to `u32`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color(u32);

impl Color {
    /// Builds a color from red, green, and blue components.
    #[inline]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r as u32 | (g as u32) << 8 | (b as u32) << 16)
    }

    /// The red component.
    #[inline]
    pub const fn r(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// The green component.
    #[inline]
    pub const fn g(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    /// The blue component.
    #[inline]
    pub const fn b(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    /// The raw packed `u32` value, laid out as `0x00BBGGRR`.
    #[inline]
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Builds a color from a raw packed `u32` value, laid out as `0x00BBGGRR`.
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        Self(value)
    }

    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);
    pub const RED: Self = Self::from_rgb(255, 0, 0);
    pub const GREEN: Self = Self::from_rgb(0, 255, 0);
    pub const BLUE: Self = Self::from_rgb(0, 0, 255);
}

impl From<u32> for Color {
    /// Same as [`Color::from_u32`].
    #[inline]
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<Color> for u32 {
    /// Same as [`Color::to_u32`].
    #[inline]
    fn from(color: Color) -> Self {
        color.to_u32()
    }
}

impl From<(u8, u8, u8)> for Color {
    /// Same as [`Color::from_rgb`].
    #[inline]
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::from_rgb(r, g, b)
    }
}
