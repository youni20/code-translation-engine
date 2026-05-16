const IMGTYPE_RAW: i32 = 0;
const IMGTYPE_BMP: i32 = 1;
const IMGTYPE_PPM: i32 = 2;
const IMGTYPE_PGM: i32 = 3;
const IMGTYPE_UNSUPPORTED: i32 = 999;

struct Image; // Assume Image struct is defined elsewhere.

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
    fn load_image(filename: &str, image: &mut Image) {
        let image_type = Self::get_image_type(filename);
        Self::load_image_with_type(filename, image, image_type);
    }

    // Loads the image from a file named `filename` into `image`,
    // using the format given as `image_type`.
    fn load_image_with_type(filename: &str, image: &mut Image, image_type: i32) {
        match image_type {
            IMGTYPE_BMP => Self::load_image_bmp(filename, image),
            IMGTYPE_PPM => Self::load_image_ppm(filename, image),
            IMGTYPE_PGM => Self::load_image_pgm(filename, image),
            IMGTYPE_RAW => Self::load_image_raw(filename, image, 0, 0),
            _ => panic!("Unsupported image type"),
        }
    }

    // Saves the image into file named `filename`.
    // This method automatically determines the image format.
    fn save_image(filename: &str, image: &Image) {
        let image_type = Self::get_image_type(filename);
        Self::save_image_with_type(filename, image, image_type);
    }

    // Saves the image into file named `filename`,
    // using the format given as `image_type`.
    fn save_image_with_type(filename: &str, image: &Image, image_type: i32) {
        match image_type {
            IMGTYPE_BMP => Self::save_image_bmp(filename, image),
            IMGTYPE_PPM => Self::save_image_ppm(filename, image),
            IMGTYPE_PGM => Self::save_image_pgm(filename, image),
            IMGTYPE_RAW => Self::save_image_raw(filename, image),
            _ => panic!("Unsupported image type"),
        }
    }

    // Loads the uncompressed BMP image from `filename` into `image`.
    fn load_image_bmp(_filename: &str, _image: &mut Image) {
        // Implementation goes here
    }

    // Saves the image into file named `filename` in uncompressed BMP format.
    fn save_image_bmp(_filename: &str, _image: &Image) {
        // Implementation goes here
    }

    // Loads the PPM image from `filename` into `image`.
    fn load_image_ppm(_filename: &str, _image: &mut Image) {
        // Implementation goes here
    }

    // Saves the image into file named `filename` in PPM format.
    fn save_image_ppm(_filename: &str, _image: &Image) {
        // Implementation goes here
    }

    // Loads the PGM image from `filename` into `image`.
    fn load_image_pgm(_filename: &str, _image: &mut Image) {
        // Implementation goes here
    }

    // Saves the image into file named `filename` in PGM format.
    fn save_image_pgm(_filename: &str, _image: &Image) {
        // Implementation goes here
    }

    // Loads the image from the file named `filename`.
    // The file is assumed to be structured so that it only contains
    // an array of raw (gray) pixel values, as the file does not contain
    // the image width and height, those are passed as parameters to the
    // function. If width and height are 0, the image is assumed to be
    // square and the width and height are computed based on the file size.
    fn load_image_raw(_filename: &str, _image: &mut Image, _width: u32, _height: u32) {
        // Implementation goes here
    }

    // Saves the image to a file named `filename`.
    // Only the array of raw (gray) pixel values are stored,
    // without additional information such as image size.
    fn save_image_raw(_filename: &str, _image: &Image) {
        // Implementation goes here
    }
}

fn main() {
    // Entry point of the program
}