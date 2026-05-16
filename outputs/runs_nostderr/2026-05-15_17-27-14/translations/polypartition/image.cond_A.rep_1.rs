// A simple image class with some (very basic) image processing operations.

#[derive(Copy, Clone)]
pub struct Pixel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

pub struct Image {
    data: Vec<u8>,
    width: i64,
    height: i64,
}

impl Image {
    // Constructors and destructor.
    pub fn new() -> Image {
        Image {
            data: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn with_dimensions(width: i64, height: i64) -> Image {
        let data = vec![0; (width * height * 3) as usize];
        Image { data, width, height }
    }

    pub fn init(&mut self, width: i64, height: i64) {
        self.width = width;
        self.height = height;
        self.data = vec![0; (width * height * 3) as usize];
    }

    pub fn get_width(&self) -> i64 {
        self.width
    }

    pub fn get_height(&self) -> i64 {
        self.height
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_pixel_gray(&self, x: i64, y: i64) -> u8 {
        let index = 3 * ((y * self.width) + x);
        ((self.data[index as usize] as i64 + self.data[index as usize + 1] as i64 + self.data[index as usize + 2] as i64) / 3) as u8
    }

    pub fn get_pixel_red(&self, x: i64, y: i64) -> u8 {
        let index = 3 * ((y * self.width) + x);
        self.data[index as usize]
    }

    pub fn get_pixel_green(&self, x: i64, y: i64) -> u8 {
        let index = 3 * ((y * self.width) + x) + 1;
        self.data[index as usize]
    }

    pub fn get_pixel_blue(&self, x: i64, y: i64) -> u8 {
        let index = 3 * ((y * self.width) + x) + 2;
        self.data[index as usize]
    }

    pub fn set_pixel_red(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * ((y * self.width) + x);
        self.data[index as usize] = c;
    }

    pub fn set_pixel_green(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * ((y * self.width) + x) + 1;
        self.data[index as usize] = c;
    }

    pub fn set_pixel_blue(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * ((y * self.width) + x) + 2;
        self.data[index as usize] = c;
    }

    pub fn get_pixel_color(&self, x: i64, y: i64) -> Pixel {
        let index = 3 * ((y * self.width) + x);
        Pixel {
            b: self.data[index as usize],
            g: self.data[index as usize + 1],
            r: self.data[index as usize + 2],
        }
    }

    pub fn set_pixel_gray(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * ((y * self.width) + x);
        self.data[index as usize] = c;
        self.data[index as usize + 1] = c;
        self.data[index as usize + 2] = c;
    }

    pub fn set_pixel_color(&mut self, x: i64, y: i64, rgb: Pixel) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let index = 3 * ((y * self.width) + x);
        self.data[index as usize] = rgb.b;
        self.data[index as usize + 1] = rgb.g;
        self.data[index as usize + 2] = rgb.r;
    }

    // Other methods like `get_pixel_bilinear`, `get_mean_gray`, etc., would follow the same pattern.
}

fn main() {
    // Example usage of the Image struct.
    let mut img = Image::with_dimensions(10, 10);
    img.set_pixel_red(1, 1, 255);
    println!("Pixel red value at (1, 1): {}", img.get_pixel_red(1, 1));
}