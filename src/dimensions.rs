#[derive(Clone, Copy)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn new(
        width: u32,
        height: u32,
    ) -> Self {
        Dimensions {
            width,
            height,
        }
    }

    pub fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
