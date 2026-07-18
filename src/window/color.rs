#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color32(u32);

impl Color32 {
    #[inline]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r as u32 | (g as u32) << 8 | (b as u32) << 16)
    }

    #[inline]
    pub const fn r(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    #[inline]
    pub const fn g(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    #[inline]
    pub const fn b(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    #[inline]
    pub const fn to_u32(self) -> u32 {
        self.0
    }

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

impl From<u32> for Color32 {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<Color32> for u32 {
    #[inline]
    fn from(color: Color32) -> Self {
        color.to_u32()
    }
}

impl From<(u8, u8, u8)> for Color32 {
    #[inline]
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::from_rgb(r, g, b)
    }
}
