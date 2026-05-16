pub struct Image {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

pub struct Pixel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl Image {
    pub fn new() -> Self {
        Image {
            data: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn with_dimensions(width: usize, height: usize) -> Self {
        Image {
            data: vec![0; width * height * 3],
            width,
            height,
        }
    }

    pub fn init(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.data = vec![0; width * height * 3];
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_pixel_gray(&self, x: usize, y: usize) -> u8 {
        let index = 3 * ((y * self.width) + x);
        ((self.data[index] as usize + self.data[index + 1] as usize + self.data[index + 2] as usize) / 3) as u8
    }

    pub fn get_pixel_red(&self, x: usize, y: usize) -> u8 {
        let index = 3 * ((y * self.width) + x);
        self.data[index]
    }

    pub fn get_pixel_green(&self, x: usize, y: usize) -> u8 {
        let index = 3 * ((y * self.width) + x) + 1;
        self.data[index]
    }

    pub fn get_pixel_blue(&self, x: usize, y: usize) -> u8 {
        let index = 3 * ((y * self.width) + x) + 2;
        self.data[index]
    }

    pub fn set_pixel_red(&mut self, x: usize, y: usize, c: u8) {
        let index = 3 * ((y * self.width) + x);
        self.data[index] = c;
    }

    pub fn set_pixel_green(&mut self, x: usize, y: usize, c: u8) {
        let index = 3 * ((y * self.width) + x) + 1;
        self.data[index] = c;
    }

    pub fn set_pixel_blue(&mut self, x: usize, y: usize, c: u8) {
        let index = 3 * ((y * self.width) + x) + 2;
        self.data[index] = c;
    }

    pub fn get_pixel_color(&self, x: usize, y: usize) -> Pixel {
        let index = 3 * ((y * self.width) + x);
        Pixel {
            b: self.data[index],
            g: self.data[index + 1],
            r: self.data[index + 2],
        }
    }

    pub fn set_pixel_gray(&mut self, x: usize, y: usize, c: u8) {
        let index = 3 * ((y * self.width) + x);
        self.data[index] = c;
        self.data[index + 1] = c;
        self.data[index + 2] = c;
    }

    pub fn set_pixel_color(&mut self, x: usize, y: usize, rgb: Pixel) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = 3 * ((y * self.width) + x);
        self.data[index] = rgb.b;
        self.data[index + 1] = rgb.g;
        self.data[index + 2] = rgb.r;
    }
}

fn main() {
    // Example usage of the Image struct
    let mut img = Image::with_dimensions(10, 10);
    img.set_pixel_red(0, 0, 255);
    let red = img.get_pixel_red(0, 0);
    println!("Red value at (0, 0): {}", red);
}