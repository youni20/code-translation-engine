const IMGTYPE_RAW: i32 = 0;
const IMGTYPE_BMP: i32 = 1;
const IMGTYPE_PPM: i32 = 2;
const IMGTYPE_PGM: i32 = 3;
const IMGTYPE_UNSUPPORTED: i32 = 999;

// Struct to represent an Image. The implementation details would be completed as needed.
struct Image;

struct ImageIO;

impl ImageIO {
    // Gets the file extension from the file name.
    fn get_file_extension(filename: &str) -> Option<&str> {
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
        let image_type = ImageIO::get_image_type(filename);
        self.load_image_with_type(filename, image, image_type);
    }

    // Loads the image from a file named `filename` into `image`,
    // using the format given as `image_type`.
    fn load_image_with_type(&self, filename: &str, image: &mut Image, image_type: i32) {
        match image_type {
            IMGTYPE_BMP => self.load_image_bmp(filename, image),
            IMGTYPE_PPM => self.load_image_ppm(filename, image),
            IMGTYPE_PGM => self.load_image_pgm(filename, image),
            IMGTYPE_RAW => self.load_image_raw(filename, image, 0, 0),
            _ => (), // Unsupported image type
        }
    }

    // Saves the image into file named `filename`.
    // This method automatically determines the image format.
    fn save_image(&self, filename: &str, image: &Image) {
        let image_type = ImageIO::get_image_type(filename);
        self.save_image_with_type(filename, image, image_type);
    }

    // Saves the image into file named `filename`,
    // using the format given as `image_type`.
    fn save_image_with_type(&self, filename: &str, image: &Image, image_type: i32) {
        match image_type {
            IMGTYPE_BMP => self.save_image_bmp(filename, image),
            IMGTYPE_PPM => self.save_image_ppm(filename, image),
            IMGTYPE_PGM => self.save_image_pgm(filename, image),
            IMGTYPE_RAW => self.save_image_raw(filename, image),
            _ => (), // Unsupported image type
        }
    }

    // Loads the uncompressed BMP image from `filename` into `image`.
    fn load_image_bmp(&self, _filename: &str, _image: &mut Image) {
        // Implementation goes here
    }

    // Saves the image into file named `filename` in uncompressed BMP format.
    fn save_image_bmp(&self, _filename: &str, _image: &Image) {
        // Implementation goes here
    }

    // Loads the PPM image from `filename` into `image`.
    fn load_image_ppm(&self, _filename: &str, _image: &mut Image) {
        // Implementation goes here
    }
    
    // Saves the image into file named `filename` in PPM format.
    fn save_image_ppm(&self, _filename: &str, _image: &Image) {
        // Implementation goes here
    }

    // Loads the PGM image from `filename` into `image`.
    fn load_image_pgm(&self, _filename: &str, _image: &mut Image) {
        // Implementation goes here
    }

    // Saves the image into file named `filename` in PGM format.
    fn save_image_pgm(&self, _filename: &str, _image: &Image) {
        // Implementation goes here
    }

    // Loads the image from the file named `filename`.
    // The file is assumed to be structured so that it only contains
    // an array of raw (gray) pixel values, as the file does not contain
    // the image width and height, those are passed as parameters to the
    // function. If width and height are 0, the image is assumed to be
    // square and the width and height are computed based on the file size.
    fn load_image_raw(&self, _filename: &str, _image: &mut Image, _width: usize, _height: usize) {
        // Implementation goes here
    }

    // Saves the image to a file named `filename`.
    // Only the array of raw (gray) pixel values are stored,
    // without additional information such as image size.
    fn save_image_raw(&self, _filename: &str, _image: &Image) {
        // Implementation goes here
    }
}

fn main() {
    // Example usage
    let mut image = Image;
    let image_io = ImageIO;
    image_io.load_image("example.bmp", &mut image);
    image_io.save_image("example_saved.bmp", &image);
}