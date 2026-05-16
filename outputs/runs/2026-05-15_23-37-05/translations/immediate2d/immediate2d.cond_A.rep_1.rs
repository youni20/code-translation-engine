use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering}};
use std::collections::{VecDeque};
use std::time::{Duration};
use std::thread;
use std::fmt;

const WIDTH: i32 = 160;
const HEIGHT: i32 = 120;
const PIXEL_SCALE: i32 = 5;

type Color = u32;
type Image = usize;

const TRANSPARENT: Color = 0x00000000;
const BLACK: Color = make_color(0, 0, 0);
const WHITE: Color = make_color(255, 255, 255);

#[derive(Debug)]
enum Keys {
    Left = 17,
    Up,
    Right,
    Down,
    Backspace = 8,
    Enter = 13,
    Esc = 27,
    Tab = 9,
}

#[derive(Debug, Default, Clone)]
struct Immediate2D {
    dirty: Arc<AtomicBool>,
    double_buffered: Arc<AtomicBool>,
    key: Arc<AtomicU8>,
    quitting: Arc<AtomicBool>,
    mouse_down: Vec<Arc<AtomicBool>>,
    mouse_x: Arc<AtomicI32>,
    mouse_y: Arc<AtomicI32>,
    run_duration: Arc<Mutex<Duration>>,
    media_lock: Arc<Mutex<()>>,
    bitmap: Arc<Mutex<Option<Vec<Color>>>>,
    input_buffer: Arc<Mutex<VecDeque<u8>>>,
}

impl Immediate2D {
    fn new() -> Self {
        Self {
            dirty: Arc::new(AtomicBool::new(true)),
            double_buffered: Arc::new(AtomicBool::new(false)),
            key: Arc::new(AtomicU8::new(0)),
            quitting: Arc::new(AtomicBool::new(false)),
            mouse_down: vec![Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false))],
            mouse_x: Arc::new(AtomicI32::new(-1)),
            mouse_y: Arc::new(AtomicI32::new(-1)),
            run_duration: Arc::new(Mutex::new(Duration::new(0, 0))),
            media_lock: Arc::new(Mutex::new(())),
            bitmap: Arc::new(Mutex::new(Some(vec![TRANSPARENT; (WIDTH * HEIGHT) as usize]))),
            input_buffer: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn draw_pixel(&self, x: i32, y: i32, c: Color) {
        if (0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y) {
            if let Ok(mut bitmap) = self.bitmap.lock() {
                if let Some(ref mut bitmap) = *bitmap {
                    bitmap[(y * WIDTH + x) as usize] = c;
                    self.dirty.store(true, Ordering::SeqCst);
                }
            }
        }
    }

    fn clear(&self, c: Color) {
        if let Ok(mut bitmap) = self.bitmap.lock() {
            if let Some(ref mut bitmap) = *bitmap {
                for pixel in bitmap.iter_mut() {
                    *pixel = c;
                }
                self.dirty.store(true, Ordering::SeqCst);
            }
        }
    }

    fn last_key(&self) -> Option<u8> {
        if let Some(key) = self.input_buffer.lock().unwrap().pop_front() {
            Some(key)
        } else {
            None
        }
    }

    fn last_buffered_key(&self) -> Option<u8> {
        self.last_key()
    }

    fn save_image(&self, _suffix: u32) {
        // Placeholder: replace with actual image saving functionality
        println!("SaveImage not implemented");
    }

    fn random_int(&self, low: i32, high: i32) -> i32 {
        // Placeholder for random integer generation
        low + (high - low) / 2  // Simple deterministic value for compilation
    }

    fn random_bool(&self) -> bool {
        self.random_int(0, 2) == 1
    }

    fn random_double(&self) -> f64 {
        // Placeholder for random double generation
        0.5 // Simple deterministic value for compilation
    }

    fn run(&self) {
        // Placeholder: replace with actual run implementation
        println!("Run function not implemented");
    }
}

const fn make_color(red: i32, green: i32, blue: i32) -> Color {
    (((255 & 0xFF) << 24) | ((red & 0xFF) << 16) | ((green & 0xFF) << 8) | ((blue & 0xFF) << 0)) as u32
}

impl fmt::Display for Immediate2D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Immediate2D State")
    }
}

fn main() {
    let immediate2d = Arc::new(Immediate2D::new());
    let immediate2d_clone = Arc::clone(&immediate2d);
    thread::spawn(move || {
        while !immediate2d_clone.quitting.load(Ordering::SeqCst) {
            immediate2d_clone.run();
            thread::sleep(Duration::from_millis(16));
        }
    });

    // Simulation of the application loop
    thread::sleep(Duration::from_secs(5));
    immediate2d.quitting.store(true, Ordering::SeqCst);
}