pub struct Image {
    data: Vec<u8>,
    width: i64,
    height: i64,
}

#[derive(Clone, Copy)]
pub struct Pixel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl Image {
    // Constructors & destructor equivalent
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn with_dimensions(width: i64, height: i64) -> Self {
        let data = vec![0; (3 * width * height) as usize];
        Self { data, width, height }
    }

    // Equivalent to Image(const Image &src) Copy constructor
    pub fn from_image(src: &Image) -> Self {
        Self {
            data: src.data.clone(),
            width: src.width,
            height: src.height,
        }
    }

    // Equivalent to Image &operator=(const Image &src)
    pub fn assign_from(&mut self, src: &Image) -> &mut Self {
        self.data = src.data.clone();
        self.width = src.width;
        self.height = src.height;
        self
    }

    // Initializes the image of specified width and height. All pixels black.
    pub fn init(&mut self, width: i64, height: i64) {
        self.data = vec![0; (3 * width * height) as usize];
        self.width = width;
        self.height = height;
    }

    // Property getters
    pub fn width(&self) -> i64 {
        self.width
    }

    pub fn height(&self) -> i64 {
        self.height
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    // Pixel getters and setters
    pub fn get_pixel_gray(&self, x: i64, y: i64) -> u8 {
        let index = 3 * (y * self.width + x);
        ((self.data[index as usize] as i64
          + self.data[index as usize + 1] as i64
          + self.data[index as usize + 2] as i64) / 3) as u8
    }

    pub fn get_pixel_red(&self, x: i64, y: i64) -> u8 {
        let index = 3 * (y * self.width + x);
        self.data[index as usize]
    }

    pub fn get_pixel_green(&self, x: i64, y: i64) -> u8 {
        let index = 3 * (y * self.width + x) + 1;
        self.data[index as usize]
    }

    pub fn get_pixel_blue(&self, x: i64, y: i64) -> u8 {
        let index = 3 * (y * self.width + x) + 2;
        self.data[index as usize]
    }

    pub fn set_pixel_red(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * (y * self.width + x);
        self.data[index as usize] = c;
    }

    pub fn set_pixel_green(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * (y * self.width + x) + 1;
        self.data[index as usize] = c;
    }

    pub fn set_pixel_blue(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * (y * self.width + x) + 2;
        self.data[index as usize] = c;
    }

    pub fn get_pixel_color(&self, x: i64, y: i64) -> Pixel {
        let index = 3 * (y * self.width + x);
        Pixel {
            b: self.data[index as usize],
            g: self.data[index as usize + 1],
            r: self.data[index as usize + 2],
        }
    }

    pub fn set_pixel_gray(&mut self, x: i64, y: i64, c: u8) {
        let index = 3 * (y * self.width + x);
        self.data[index as usize] = c;
        self.data[index as usize + 1] = c;
        self.data[index as usize + 2] = c;
    }

    pub fn set_pixel_color(&mut self, x: i64, y: i64, rgb: Pixel) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let index = 3 * (y * self.width + x);
        self.data[index as usize] = rgb.b;
        self.data[index as usize + 1] = rgb.g;
        self.data[index as usize + 2] = rgb.r;
    }

    // Placeholder function to avoid errors, implementation would require further context
    pub fn get_pixel_bilinear(&self, _x: f32, _y: f32) -> Pixel {
        Pixel { b: 0, g: 0, r: 0 }
    }

    pub fn get_mean_gray(&self) -> u8 {
        let total: i64 = self.data.chunks(3).map(|c| c.iter().map(|&v| v as i64).sum::<i64>() / 3).sum();
        (total / (self.width * self.height)) as u8
    }

    pub fn get_histogram_gray(&self, histogram: &mut [i64; 256]) {
        for i in 0..256 {
            histogram[i] = 0;
        }
        for y in 0..self.height {
            for x in 0..self.width {
                let gray = self.get_pixel_gray(x, y);
                histogram[gray as usize] += 1;
            }
        }
    }

    pub fn binarize(&mut self, threshold: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                let gray = self.get_pixel_gray(x, y);
                let color = if gray < threshold { 0 } else { 255 };
                self.set_pixel_gray(x, y, color);
            }
        }
    }

    pub fn flip_horizontal(&mut self) {
        let half_width = self.width / 2;
        for y in 0..self.height {
            for x in 0..half_width {
                let opposite_x = self.width - x - 1;
                let index1 = 3 * (y * self.width + x);
                let index2 = 3 * (y * self.width + opposite_x);
                for offset in 0..3 {
                    self.data.swap(index1 as usize + offset, index2 as usize + offset);
                }
            }
        }
    }

    pub fn flip_vertical(&mut self) {
        let half_height = self.height / 2;
        for y in 0..half_height {
            for x in 0..self.width {
                let opposite_y = self.height - y - 1;
                let index1 = 3 * (y * self.width + x);
                let index2 = 3 * (opposite_y * self.width + x);
                for offset in 0..3 {
                    self.data.swap(index1 as usize + offset, index2 as usize + offset);
                }
            }
        }
    }

    pub fn invert(&mut self) {
        for c in self.data.iter_mut() {
            *c = 255 - *c;
        }
    }

    pub fn crop(&self, posx: i32, posy: i32, width: i32, height: i32) -> Image {
        let mut cropped_image = Image::with_dimensions(width as i64, height as i64);
        for y in 0..height {
            for x in 0..width {
                if posx + x < self.width as i32 && posy + y < self.height as i32 {
                    let color = self.get_pixel_color((posx + x) as i64, (posy + y) as i64);
                    cropped_image.set_pixel_color(x as i64, y as i64, color);
                }
            }
        }
        cropped_image
    }

    pub fn resize(&self, factor: i32) -> Image {
        if factor <= 0 {
            return Image::new();
        }
        let new_width = self.width / factor as i64;
        let new_height = self.height / factor as i64;
        let mut resized_image = Image::with_dimensions(new_width, new_height);
        for y in 0..new_height {
            for x in 0..new_width {
                let mut sum_b = 0;
                let mut sum_g = 0;
                let mut sum_r = 0;
                let count = factor * factor;
                for j in 0..factor {
                    for i in 0..factor {
                        let color = self.get_pixel_color(x * factor as i64 + i as i64, y * factor as i64 + j as i64);
                        sum_b += color.b as i32;
                        sum_g += color.g as i32;
                        sum_r += color.r as i32;
                    }
                }
                resized_image.set_pixel_color(
                    x,
                    y,
                    Pixel {
                        b: (sum_b / count) as u8,
                        g: (sum_g / count) as u8,
                        r: (sum_r / count) as u8,
                    },
                );
            }
        }
        resized_image
    }

    pub fn filter(&self, filter: &[f32], filterwidth: i64, filterheight: i64) -> Image {
        let mut result_image = Image::with_dimensions(self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let mut sum_b = 0.0;
                let mut sum_g = 0.0;
                let mut sum_r = 0.0;
                for j in 0..filterheight {
                    for i in 0..filterwidth {
                        let offset_x = i - filterwidth / 2;
                        let offset_y = j - filterheight / 2;
                        let xx = (x + offset_x).clamp(0, self.width - 1);
                        let yy = (y + offset_y).clamp(0, self.height - 1);
                        let color = self.get_pixel_color(xx, yy);
                        let coef = filter[(j * filterwidth + i) as usize];
                        sum_b += color.b as f32 * coef;
                        sum_g += color.g as f32 * coef;
                        sum_r += color.r as f32 * coef;
                    }
                }
                result_image.set_pixel_color(
                    x,
                    y,
                    Pixel {
                        b: sum_b.clamp(0.0, 255.0) as u8,
                        g: sum_g.clamp(0.0, 255.0) as u8,
                        r: sum_r.clamp(0.0, 255.0) as u8,
                    },
                );
            }
        }
        result_image
    }

    pub fn gauss_blur(&self, sigma: f32, masksize: i64) -> Image {
        let mut size = masksize;
        if size == 0 {
            size = (sigma * 2.0 * 2.0 + 1.0).ceil() as i64;
        }
        let mut filter = vec![0.0; (size * size) as usize];
        let mut sum = 0.0;
        for y in 0..size {
            for x in 0..size {
                let r = (((x as f32 - size as f32 / 2.0).powi(2) + (y as f32 - size as f32 / 2.0).powi(2))
                    / (2.0 * sigma * sigma))
                .exp();
                filter[(y * size + x) as usize] = r;
                sum += r;
            }
        }
        for f in filter.iter_mut() {
            *f /= sum;
        }
        self.filter(&filter, size, size)
    }

    pub fn clear(&mut self, color: Pixel) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel_color(x, y, color);
            }
        }
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Pixel) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            if x >= 0 && y >= 0 && (x as i64) < self.width && (y as i64) < self.height {
                self.set_pixel_color(x as i64, y as i64, color);
            }
            if x == x2 && y == y2 {
                break;
            }
            let err2 = 2 * err;
            if err2 > -dy {
                err -= dy;
                x += sx;
            }
            if err2 < dx {
                err += dx;
                y += sy;
            }
        }
    }
}

fn main() {
    // Example usage of Image
    let mut image = Image::with_dimensions(100, 100);
    image.set_pixel_color(10, 10, Pixel { r: 255, g: 0, b: 0 });
}