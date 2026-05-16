pub struct Image {
    data: Vec<u8>,
    width: i64,
    height: i64,
}

pub struct Pixel {
    b: u8,
    g: u8,
    r: u8,
}

impl Image {
    pub fn new() -> Self {
        Image {
            data: vec![],
            width: 0,
            height: 0,
        }
    }

    pub fn with_size(width: i64, height: i64) -> Self {
        let size = (width * height * 3) as usize;
        Image {
            data: vec![0; size],
            width,
            height,
        }
    }

    pub fn init(&mut self, width: i64, height: i64) {
        self.width = width;
        self.height = height;
        self.data.resize((width * height * 3) as usize, 0);
    }

    fn round(x: f32) -> i64 {
        x.round() as i64
    }

    fn interpolate(x1: f32, x2: f32, x3: f32, x4: f32, dx: f32, dy: f32) -> f32 {
        x1 * (1.0 - dx) * (1.0 - dy) +
        x2 * dx * (1.0 - dy) +
        x3 * (1.0 - dx) * dy +
        x4 * dx * dy
    }

    pub fn get_width(&self) -> i64 {
        self.width
    }

    pub fn get_height(&self) -> i64 {
        self.height
    }

    pub fn get_data(&self) -> &[u8] { // returning a slice is safer in Rust
        &self.data
    }

    pub fn get_pixel_gray(&self, x: i64, y: i64) -> u8 {
        let index = (3 * (y * self.width + x)) as usize;
        ((self.data[index] as i64 + self.data[index + 1] as i64 + self.data[index + 2] as i64) / 3) as u8
    }

    pub fn get_pixel_red(&self, x: i64, y: i64) -> u8 {
        let index = (3 * (y * self.width + x)) as usize;
        self.data[index]
    }

    pub fn get_pixel_green(&self, x: i64, y: i64) -> u8 {
        let index = (3 * (y * self.width + x) + 1) as usize;
        self.data[index]
    }

    pub fn get_pixel_blue(&self, x: i64, y: i64) -> u8 {
        let index = (3 * (y * self.width + x) + 2) as usize;
        self.data[index]
    }

    pub fn set_pixel_red(&mut self, x: i64, y: i64, c: u8) {
        let index = (3 * (y * self.width + x)) as usize;
        self.data[index] = c;
    }

    pub fn set_pixel_green(&mut self, x: i64, y: i64, c: u8) {
        let index = (3 * (y * self.width + x) + 1) as usize;
        self.data[index] = c;
    }

    pub fn set_pixel_blue(&mut self, x: i64, y: i64, c: u8) {
        let index = (3 * (y * self.width + x) + 2) as usize;
        self.data[index] = c;
    }

    pub fn get_pixel_color(&self, x: i64, y: i64) -> Pixel {
        let index = (3 * (y * self.width + x)) as usize;
        Pixel {
            b: self.data[index],
            g: self.data[index + 1],
            r: self.data[index + 2],
        }
    }

    pub fn set_pixel_gray(&mut self, x: i64, y: i64, c: u8) {
        let index = (3 * (y * self.width + x)) as usize;
        self.data[index] = c;
        self.data[index + 1] = c;
        self.data[index + 2] = c;
    }

    pub fn set_pixel_color(&mut self, x: i64, y: i64, rgb: Pixel) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let index = (3 * (y * self.width + x)) as usize;
        self.data[index] = rgb.b;
        self.data[index + 1] = rgb.g;
        self.data[index + 2] = rgb.r;
    }
}

fn main() {
    // Main function added to avoid compiler errors related to missing entry point.
}