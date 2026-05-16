use std::sync::{Arc, Mutex};

type Color = u32;

fn make_color(red: u8, green: u8, blue: u8) -> Color {
    0xFF000000 | ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
}

const TRANSPARENT: Color = 0x00000000;
const BLACK: Color = 0xFF000000;
const WHITE: Color = 0xFFFFFFFF;

const TAU: f64 = 6.283185307179586476925286766559;

fn radians(degrees: f64) -> f64 {
    degrees * TAU / 360.0
}

fn degrees(radians: f64) -> f64 {
    radians * 360.0 / TAU
}

struct Application {
    width: u32,
    height: u32,
    pixel_scale: u32,
    quit: Arc<Mutex<bool>>,
}

impl Application {
    fn new(width: u32, height: u32, pixel_scale: u32) -> Self {
        Self {
            width,
            height,
            pixel_scale,
            quit: Arc::new(Mutex::new(false)),
        }
    }

    fn run(&self) {
        let quit = Arc::clone(&self.quit);
        let _handle = std::thread::spawn(move || {
            // Call user run function here
            run();
            *quit.lock().unwrap() = true;
        });

        while !*self.quit.lock().unwrap() {
            // Main loop work
        }
    }

    fn close_window(&self) {
        *self.quit.lock().unwrap() = true;
    }
}

fn run() {
    // Example run function
    draw_pixel(10, 10, WHITE);
}

fn draw_pixel(_x: i32, _y: i32, _color: Color) {
    // Implementation for drawing a pixel
}

fn main() {
    let app = Application::new(160, 120, 5);
    app.run();
}