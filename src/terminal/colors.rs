use image::Rgb;

#[derive(Clone, Copy)]
pub struct TermColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl TermColor {
    pub fn to_rgb(&self) -> Rgb<u8> {
        Rgb([self.r, self.g, self.b])
    }
}
