use std::thread::sleep;
use std::time::Duration;

const FONT: [u32; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0x10000000, 0x10000017, 0x30000C03, 0x50AFABEA, 0x509AFEB2, 0x30004C99, 0x400A26AA, 0x10000003, 0x2000022E, 0x200001D1, 0x30001445, 0x300011C4, 0x10000018, 0x30001084, 0x10000010, 0x30000C98,
    0x30003A2E, 0x300043F2, 0x30004AB9, 0x30006EB1, 0x30007C87, 0x300026B7, 0x300076BF, 0x30007C21, 0x30006EBB, 0x30007EB7, 0x1000000A, 0x1000001A, 0x30004544, 0x4005294A, 0x30001151, 0x30000AA1,
    0x506ADE2E, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E, 0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
    0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F, 0x30006C9B, 0x30000F83, 0x30004EB9, 0x2000023F, 0x30006083, 0x200003F1, 0x30000822, 0x30004210,
    0x20000041, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E, 0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
    0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F, 0x30006C9B, 0x30000F83, 0x30004EB9, 0x30004764, 0x1000001F, 0x30001371, 0x50441044, 0x00000000,
];

struct Color(u8, u8, u8);

// Dummy functions to represent immediate mode 2D graphics operations.
fn draw_pixel(_x: i32, _y: i32, _color: &Color) {
    // Implementation of drawing a pixel at position (x, y) with a particular color
}

fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: Color) {
    // Implementation of drawing a rectangle starting at (x, y) with the given width, height, and color
}

fn last_buffered_key() -> Option<char> {
    // Dummy function to get the last buffered key, returns None if no key was pressed
    Some('A') // This is a placeholder for illustration
}

fn wait(milliseconds: u64) {
    sleep(Duration::from_millis(milliseconds));
}

fn measure_string(s: &str) -> i32 {
    let mut result = 0;
    for c in s.chars() {
        let glyph = FONT[c as usize];
        let width = (glyph >> 28) as i32;

        if width > 0 {
            result += width + 1;
        }
    }

    if result > 0 {
        result -= 1;
    }

    result
}

fn draw_character(left: i32, top: i32, c: char, color: &Color) -> i32 {
    let glyph = FONT[c as usize];
    let width = (glyph >> 28) as i32;

    let mut glyph = glyph;
    for x in left..left + width {
        for y in top..top + 5 {
            if (glyph & 1) == 1 {
                draw_pixel(x, y, color);
            }
            glyph >>= 1;
        }
    }

    width
}

fn draw_string(x: i32, y: i32, s: &str, color: &Color) {
    let mut x = x;
    for c in s.chars() {
        x += draw_character(x, y, c, color) + 1;
    }
}

fn run() {
    draw_string(1, 1, "Simple Text Editor", &Color(192, 192, 192)); // LightGray

    let mut cursor_x = 1;
    let cursor_y = 7;

    loop {
        wait(1);

        if let Some(key) = last_buffered_key() {
            draw_rectangle(cursor_x, cursor_y, 3, 5, Color(0, 0, 0)); // Black
            let width = draw_character(cursor_x, cursor_y, key, &Color(144, 238, 144)); // LightGreen
            if width > 0 {
                cursor_x += width + 1;
            }
            draw_rectangle(cursor_x, cursor_y, 3, 5, Color(0, 128, 0)); // Green
        }
    }
}

fn main() {
    run();
}