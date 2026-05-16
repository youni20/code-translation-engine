#[derive(Debug)]
struct Pixel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

struct Image {
    data: Vec<u8>,
    width: i64,
    height: i64,
}

impl Image {
    pub fn new() -> Image {
        Image {
            data: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn with_dimensions(width: i64, height: i64) -> Image {
        let mut image = Image {
            data: vec![0; (width * height * 3) as usize],
            width,
            height,
        };
        image.init(width, height);
        image
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

    fn round(x: f32) -> i64 {
        x.round() as i64
    }

    fn interpolate(x1: f32, x2: f32, x3: f32, x4: f32, dx: f32, dy: f32) -> f32 {
        // Bilinear interpolation
        let inter1 = x1 + dx * (x2 - x1);
        let inter2 = x3 + dx * (x4 - x3);
        inter1 + dy * (inter2 - inter1)
    }

    pub fn get_pixel_gray(&self, x: i64, y: i64) -> u8 {
        let index = 3 * ((y * self.width + x) as usize);
        ((self.data[index] as u32 + self.data[index + 1] as u32 + self.data[index + 2] as u32) / 3) as u8
    }

    pub fn get_pixel_red(&self, x: i64, y: i64) -> u8 {
        let index = 3 * ((y * self.width + x) as usize);
        self.data[index]
    }

    pub fn get_pixel_green(&self, x: i64, y: i64) -> u8 {
        let index = 3 * ((y * self.width + x) as usize) + 1;
        self.data[index]
    }

    pub fn get_pixel_blue(&self, x: i64, y: i64) -> u8 {
        let index = 3 * ((y * self.width + x) as usize) + 2;
        self.data[index]
    }

    pub fn set_pixel_red(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * ((y * self.width + x) as usize);
        self.data[index] = c;
    }

    pub fn set_pixel_green(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * ((y * self.width + x) as usize) + 1;
        self.data[index] = c;
    }

    pub fn set_pixel_blue(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * ((y * self.width + x) as usize) + 2;
        self.data[index] = c;
    }

    pub fn get_pixel_color(&self, x: i64, y: i64) -> Option<Pixel> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        let index = 3 * ((y * self.width + x) as usize);
        Some(Pixel {
            b: self.data[index],
            g: self.data[index + 1],
            r: self.data[index + 2],
        })
    }

    pub fn set_pixel_gray(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * ((y * self.width + x) as usize);
        self.data[index] = c;
        self.data[index + 1] = c;
        self.data[index + 2] = c;
    }

    pub fn set_pixel_color(&mut self, x: i64, y: i64, rgb: Pixel) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let index = 3 * ((y * self.width + x) as usize);
        self.data[index] = rgb.b;
        self.data[index + 1] = rgb.g;
        self.data[index + 2] = rgb.r;
    }
}

fn main() {
    // Example usage
    let mut img = Image::with_dimensions(100, 100);
    img.set_pixel_color(10, 10, Pixel { r: 255, g: 0, b: 0 }); // set a red pixel
    println!("Pixel at (10, 10): {:?}", img.get_pixel_color(10, 10));
}