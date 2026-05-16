const IMGTYPE_RAW: i32 = 0;
const IMGTYPE_BMP: i32 = 1;
const IMGTYPE_PPM: i32 = 2;
const IMGTYPE_PGM: i32 = 3;
const IMGTYPE_UNSUPPORTED: i32 = 999;

struct Image;

struct ImageIO;

impl ImageIO {
    // Gets the file extension from the file name.
    fn get_file_extension(filename: &str) -> Option<&str> {
        // Equivalent to C++'s `strrchr` on '/' and then '.'
        filename.rsplit('.').next()
    }

    // Determines the image format from the file name.
    fn get_image_type(filename: &str) -> i32 {
        match Self::get_file_extension(filename) {
            Some("raw") => IMGTYPE_RAW,
            Some("bmp") => IMGTYPE_BMP,
            Some("ppm") => IMGTYPE_PPM,
            Some("pgm") => IMGTYPE_PGM,
            _ => IMGTYPE_UNSUPPORTED,
        }
    }

    // Loads the image from `filename` into `image`.
    // This method automatically determines the image format.
    fn load_image(&self, filename: &str, image: &mut Image) {
        let image_type = Self::get_image_type(filename);
        self.load_image_specific(filename, image, image_type);
    }

    // Loads the image from a file named `filename` into `image`,
    // using the format given as `imageType`.
    fn load_image_specific(&self, filename: &str, image: &mut Image, image_type: i32) {
        match image_type {
            IMGTYPE_RAW => self.load_image_raw(filename, image, 0, 0),
            IMGTYPE_BMP => self.load_image_bmp(filename, image),
            IMGTYPE_PPM => self.load_image_ppm(filename, image),
            IMGTYPE_PGM => self.load_image_pgm(filename, image),
            _ => panic!("Unsupported image type"),
        }
    }

    // Saves the image into file named `filename`.
    // This method automatically determines the image format.
    fn save_image(&self, filename: &str, image: &Image) {
        let image_type = Self::get_image_type(filename);
        self.save_image_specific(filename, image, image_type);
    }

    // Saves the image into file named `filename`,
    // using the format given as `imageType`.
    fn save_image_specific(&self, filename: &str, image: &Image, image_type: i32) {
        match image_type {
            IMGTYPE_RAW => self.save_image_raw(filename, image),
            IMGTYPE_BMP => self.save_image_bmp(filename, image),
            IMGTYPE_PPM => self.save_image_ppm(filename, image),
            IMGTYPE_PGM => self.save_image_pgm(filename, image),
            _ => panic!("Unsupported image type"),
        }
    }

    // Loads the uncompressed BMP image from `filename` into `image`.
    fn load_image_bmp(&self, _filename: &str, _image: &mut Image) {
        // BMP loading logic
    }

    // Saves the image into file named `filename` in uncompressed BMP format.
    fn save_image_bmp(&self, _filename: &str, _image: &Image) {
        // BMP saving logic
    }

    // Loads the PPM image from `filename` into `image`.
    fn load_image_ppm(&self, _filename: &str, _image: &mut Image) {
        // PPM loading logic
    }

    // Saves the image into file named `filename` in PPM format.
    fn save_image_ppm(&self, _filename: &str, _image: &Image) {
        // PPM saving logic
    }

    // Loads the PGM image from `filename` into `image`.
    fn load_image_pgm(&self, _filename: &str, _image: &mut Image) {
        // PGM loading logic
    }

    // Saves the image into file named `filename` in PGM format.
    fn save_image_pgm(&self, _filename: &str, _image: &Image) {
        // PGM saving logic
    }

    // Loads the image from the file named `filename`.
    // The file is assumed to be structured so that it only contains
    // an array of raw (gray) pixel values, as the file does not contain
    // the image width and height, those are passed as parameters to the
    // function. If width and height are 0, the image is assumed to be
    // square and the width and height are computed based on the file size.
    fn load_image_raw(&self, _filename: &str, _image: &mut Image, _width: usize, _height: usize) {
        // RAW loading logic
    }

    // Saves the image to a file named `filename`.
    // Only the array of raw (gray) pixel values are stored,
    // without additional information such as image size.
    fn save_image_raw(&self, _filename: &str, _image: &Image) {
        // RAW saving logic
    }
}

// Add a main function to fix the error
fn main() {
    // Example usage (you can remove or modify this as needed)
    let image_io = ImageIO;
    let mut img = Image;
    image_io.load_image("example.bmp", &mut img);
    image_io.save_image("example.bmp", &img);
}