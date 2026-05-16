#[allow(dead_code)]
mod immediate2d {
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    pub type Color = u32;

    pub const TRANSPARENT: Color = 0;
    pub const BLACK: Color = 0xff000000;
    const BLUE: Color = 0xff0000aa;
    const GREEN: Color = 0xff00aa00;
    const CYAN: Color = 0xff00aaaa;
    const RED: Color = 0xffaa0000;
    const MAGENTA: Color = 0xffaa00aa;
    const BROWN: Color = 0xffaa5500;
    const LIGHT_GRAY: Color = 0xffaaaaaa;
    const DARK_GRAY: Color = 0xff555555;
    const LIGHT_BLUE: Color = 0xff5555aa;
    const LIGHT_GREEN: Color = 0xff55ff55;
    const LIGHT_CYAN: Color = 0xff55ffff;
    const LIGHT_RED: Color = 0xffff5555;
    const LIGHT_MAGENTA: Color = 0xffff55ff;
    const YELLOW: Color = 0xffffff55;
    const WHITE: Color = 0xffffffff;

    pub const WIDTH: usize = 160;
    pub const HEIGHT: usize = 120;
    pub const PIXEL_SCALE: usize = 5;

    pub fn make_color(r: u8, g: u8, b: u8) -> Color {
        0xff000000 | ((r as Color) << 16) | ((g as Color) << 8) | (b as Color)
    }

    pub const TAU: f64 = 6.283185307179586476925286766559;

    pub fn radians(degrees: f64) -> f64 {
        degrees * TAU / 360.0
    }

    pub fn degrees(radians: f64) -> f64 {
        radians * 360.0 / TAU
    }

    pub struct Immediate2D {
        dirty: bool,
        double_buffered: bool,
        graphics: Arc<Mutex<Option<Graphics>>>,
        input_buffer: Arc<Mutex<VecDeque<char>>>,
        run_duration: Arc<Mutex<Duration>>,
        mouse_state: Arc<Mutex<MouseState>>,
    }

    struct Graphics {
        color: Color,
        font_cache: HashMap<(String, i32), Font>,
    }

    #[derive(Clone)]
    struct Font {
        name: String,
        size: i32,
    }

    struct MouseState {
        x: i32,
        y: i32,
        left_pressed: bool,
        right_pressed: bool,
        middle_pressed: bool,
    }

    impl Immediate2D {
        pub fn new() -> Self {
            Self {
                dirty: false,
                double_buffered: false,
                graphics: Arc::new(Mutex::new(Some(Graphics {
                    color: BLACK,
                    font_cache: HashMap::new(),
                }))),
                input_buffer: Arc::new(Mutex::new(VecDeque::new())),
                run_duration: Arc::new(Mutex::new(Duration::from_millis(0))),
                mouse_state: Arc::new(Mutex::new(MouseState { x: -1, y: -1, left_pressed: false, right_pressed: false, middle_pressed: false })),
            }
        }

        pub fn draw_pixel(&mut self, x: i32, y: i32, c: Color) {
            let mut graphics = self.graphics.lock().unwrap();
            if graphics.is_none() || x < 0 || x >= WIDTH as i32 || y < 0 || y >= HEIGHT as i32 {
                return;
            }
            graphics.as_mut().unwrap().color = c;
            self.dirty = true;
        }

        pub fn use_double_buffering(&mut self, enabled: bool) {
            self.double_buffered = enabled;
            self.dirty = true;
        }

        pub fn clear(&mut self, c: Color) {
            let mut graphics = self.graphics.lock().unwrap();
            if graphics.is_none() {
                return;
            }
            graphics.as_mut().unwrap().color = c;
            self.dirty = true;
        }

        pub fn present(&mut self) {
            self.dirty = true;
        }

        pub fn last_key(&mut self) -> char {
            let mut input_buffer = self.input_buffer.lock().unwrap();
            input_buffer.pop_front().unwrap_or('\0')
        }
        
        pub fn left_mouse_pressed(&self) -> bool {
            let mouse_state = self.mouse_state.lock().unwrap();
            mouse_state.left_pressed
        }

        pub fn right_mouse_pressed(&self) -> bool {
            let mouse_state = self.mouse_state.lock().unwrap();
            mouse_state.right_pressed
        }

        pub fn middle_mouse_pressed(&self) -> bool {
            let mouse_state = self.mouse_state.lock().unwrap();
            mouse_state.middle_pressed
        }

        pub fn mouse_x(&self) -> i32 {
            let mouse_state = self.mouse_state.lock().unwrap();
            mouse_state.x
        }

        pub fn mouse_y(&self) -> i32 {
            let mouse_state = self.mouse_state.lock().unwrap();
            mouse_state.y
        }

        pub fn wait(&self, milliseconds: u64) {
            std::thread::sleep(Duration::from_millis(milliseconds));
        }
    }
}

fn main() {
    let mut engine = immediate2d::Immediate2D::new();
    engine.use_double_buffering(true);
    engine.clear(immediate2d::BLACK);
    engine.present();
}