mod immediate2d {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use std::sync::Once;

    pub type Color = u32;

    pub const Transparent: Color = 0;
    pub const Black: Color = 0xFF000000;
    pub const Blue: Color = 0xFF0000AA;
    pub const Green: Color = 0xFF00AA00;
    pub const Cyan: Color = 0xFF00AAAA;
    pub const Red: Color = 0xFFAA0000;
    pub const Magenta: Color = 0xFFAA00AA;
    pub const Brown: Color = 0xFFAA5500;
    pub const LightGray: Color = 0xFFAAAAAA;
    pub const DarkGray: Color = 0xFF555555;
    pub const LightBlue: Color = 0xFF5555AA;
    pub const LightGreen: Color = 0xFF55FF55;
    pub const LightCyan: Color = 0xFF55FFFF;
    pub const LightRed: Color = 0xFFFF5555;
    pub const LightMagenta: Color = 0xFFFF55FF;
    pub const Yellow: Color = 0xFFFFFF55;
    pub const White: Color = 0xFFFFFFFF;

    #[allow(dead_code)]
    pub const Tau: f64 = 6.283185307179586476925286766559;

    #[allow(dead_code)]
    pub fn radians(degrees: f64) -> f64 {
        degrees * Tau / 360.0
    }

    #[allow(dead_code)]
    pub fn degrees(radians: f64) -> f64 {
        radians * 360.0 / Tau
    }

    #[allow(dead_code)]
    pub fn make_color(red: u8, green: u8, blue: u8) -> Color {
        (0xFF << 24) | ((red as Color) << 16) | ((green as Color) << 8) | (blue as Color)
    }

    pub fn draw_pixel(x: i32, y: i32, c: Color) {
        let mut state = APP_STATE.lock().unwrap();
        if x >= 0 && x < state.width && y >= 0 && y < state.height {
            let index = (y * state.width + x) as usize;
            state.screen[index] = c;
        }
    }

    pub fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, c: Color) {
        let mut state = APP_STATE.lock().unwrap();
        let mut x = x1;
        let mut y = y1;
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        while x != x2 || y != y2 {
            let index = (y * state.width + x) as usize;
            state.screen[index] = c;
            let e2 = err * 2;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn clear(c: Color) {
        let mut state = APP_STATE.lock().unwrap();
        state.screen.fill(c);
    }

    pub struct AppState {
        width: i32,
        height: i32,
        screen: Vec<Color>,
    }

    static mut APP_STATE: Option<Arc<Mutex<AppState>>> = None;
    static INIT: Once = Once::new();

    fn get_app_state() -> &'static Arc<Mutex<AppState>> {
        unsafe {
            INIT.call_once(|| {
                APP_STATE = Some(Arc::new(Mutex::new(AppState {
                    width: 160,
                    height: 120,
                    screen: vec![Black; 160 * 120],
                })));
            });
            APP_STATE.as_ref().unwrap()
        }
    }

    pub fn close_window() {}

    pub fn run_main_loop<F: FnMut() + Send + 'static>(mut draw: F) {
        let app_state = get_app_state().clone();
        thread::spawn(move || loop {
            {
                let _state = app_state.lock().unwrap();
                draw();
            }
            thread::sleep(Duration::from_millis(16)); // ~60 FPS
        });
    }
}

fn main() {
    immediate2d::run_main_loop(|| {
        immediate2d::clear(immediate2d::Black);
        immediate2d::draw_pixel(80, 60, immediate2d::White);
    });
}