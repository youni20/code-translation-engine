use std::sync::{atomic::{AtomicBool, AtomicI32, Ordering}, Mutex};
use std::collections::{HashMap, VecDeque};

type Color = u32;
const Transparent: Color = 0;

const fn MakeColor(r: i32, g: i32, b: i32) -> Color {
    0xFF000000 | ((r as Color & 0xFF) << 16) | ((g as Color & 0xFF) << 8) | (b as Color & 0xFF)
}

const Black: Color = MakeColor(0, 0, 0);
const Blue: Color = MakeColor(0, 0, 170);
const Green: Color = MakeColor(0, 170, 0);
const Cyan: Color = MakeColor(0, 170, 170);
const Red: Color = MakeColor(170, 0, 0);
const Magenta: Color = MakeColor(170, 0, 170);
const Brown: Color = MakeColor(170, 85, 0);
const LightGray: Color = MakeColor(170, 170, 170);
const DarkGray: Color = MakeColor(85, 85, 85);
const LightBlue: Color = MakeColor(85, 85, 170);
const LightGreen: Color = MakeColor(85, 255, 85);
const LightCyan: Color = MakeColor(85, 255, 255);
const LightRed: Color = MakeColor(255, 85, 85);
const LightMagenta: Color = MakeColor(255, 85, 255);
const Yellow: Color = MakeColor(255, 255, 85);
const White: Color = MakeColor(255, 255, 255);

const Width: i32 = 160;
const Height: i32 = 120;
const PixelScale: i32 = 5;
const Tau: f64 = 6.28318530717958647693;

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

struct Immediate2D {
    dirty: bool,
    double_buffer: bool,
    key: AtomicI32,
    quitting: AtomicBool,
    mouse_down: [AtomicBool; 3],
    mouse_x: AtomicI32,
    mouse_y: AtomicI32,
    input_buffer: Mutex<VecDeque<u8>>,
    images: Mutex<HashMap<String, Image>>,
}

struct Image; // Placeholder for Image type

impl Immediate2D {
    fn new() -> Self {
        Immediate2D {
            dirty: true,
            double_buffer: false,
            key: AtomicI32::new(0),
            quitting: AtomicBool::new(false),
            mouse_down: Default::default(),
            mouse_x: AtomicI32::new(-1),
            mouse_y: AtomicI32::new(-1),
            input_buffer: Mutex::new(VecDeque::new()),
            images: Mutex::new(HashMap::new()),
        }
    }

    fn present(&mut self) {
        if self.double_buffer {
            // Swapping buffers if double buffering is enabled
            println!("Swapping buffers");
        }
        self.dirty = true;
    }

    fn close_window(&self) {
        self.quitting.store(true, Ordering::SeqCst);
    }

    fn last_key(&self) -> char {
        let key = self.key.swap(0, Ordering::SeqCst);
        if key != 0 {
            key as u8 as char
        } else {
            '\0'
        }
    }

    fn clear_input_buffer(&self) {
        if let Ok(mut buffer) = self.input_buffer.lock() {
            buffer.clear();
        }
    }

    fn last_buffered_key(&self) -> char {
        let mut input_buffer = self.input_buffer.lock().unwrap();
        if let Some(&key) = input_buffer.front() {
            input_buffer.pop_front();
            key as char
        } else {
            '\0'
        }
    }

    fn handle_key_event(&self, key: u8) {
        // Adding key to the input buffer
        let mut input_buffer = self.input_buffer.lock().unwrap();
        if input_buffer.len() < 100 {
            input_buffer.push_back(key);
        }
    }
}

fn main() {
    let mut imm2d = Immediate2D::new();
    imm2d.handle_key_event(65);
    println!("Last buffered key: {}", imm2d.last_buffered_key());
    imm2d.present();
}