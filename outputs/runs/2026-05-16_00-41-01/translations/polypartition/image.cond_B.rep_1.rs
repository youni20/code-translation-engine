// A simple image class with some (very basic) image processing operations.

pub struct Image {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Pixel {
    b: u8,
    g: u8,
    r: u8,
}

impl Image {
    // Constructors and destructor.
    pub fn new() -> Self {
        Image {
            data: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn with_dimensions(width: usize, height: usize) -> Self {
        let size = 3 * width * height;
        Image {
            data: vec![0; size],
            width,
            height,
        }
    }

    pub fn init(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.data = vec![0; 3 * width * height];
    }

    // Property getters.
    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    // Pixel getters and setters.
    pub fn get_pixel_gray(&self, x: usize, y: usize) -> u8 {
        let index = 3 * (y * self.width + x);
        ((self.data[index] as usize + self.data[index + 1] as usize + self.data[index + 2] as usize) / 3) as u8
    }

    pub fn get_pixel_red(&self, x: usize, y: usize) -> u8 {
        let index = 3 * (y * self.width + x);
        self.data[index]
    }

    pub fn get_pixel_green(&self, x: usize, y: usize) -> u8 {
        let index = 3 * (y * self.width + x) + 1;
        self.data[index]
    }

    pub fn get_pixel_blue(&self, x: usize, y: usize) -> u8 {
        let index = 3 * (y * self.width + x) + 2;
        self.data[index]
    }

    pub fn set_pixel_red(&mut self, x: usize, y: usize, c: u8) {
        let index = 3 * (y * self.width + x);
        self.data[index] = c;
    }

    pub fn set_pixel_green(&mut self, x: usize, y: usize, c: u8) {
        let index = 3 * (y * self.width + x) + 1;
        self.data[index] = c;
    }

    pub fn set_pixel_blue(&mut self, x: usize, y: usize, c: u8) {
        let index = 3 * (y * self.width + x) + 2;
        self.data[index] = c;
    }

    pub fn get_pixel_color(&self, x: usize, y: usize) -> Pixel {
        let index = 3 * (y * self.width + x);
        Pixel {
            b: self.data[index],
            g: self.data[index + 1],
            r: self.data[index + 2],
        }
    }

    pub fn set_pixel_gray(&mut self, x: usize, y: usize, c: u8) {
        let index = 3 * (y * self.width + x);
        self.data[index] = c;
        self.data[index + 1] = c;
        self.data[index + 2] = c;
    }

    pub fn set_pixel_color(&mut self, x: isize, y: isize, rgb: Pixel) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let index = 3 * (y as usize * self.width + x as usize);
        self.data[index] = rgb.b;
        self.data[index + 1] = rgb.g;
        self.data[index + 2] = rgb.r;
    }

    // Other methods omitted for brevity.
}

fn main() {
    let img = Image::new();
    println!("Image dimensions: {}x{}", img.get_width(), img.get_height());
    // Further usage of Image can be added here.
}