static FONT: [u32; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0x10000000, 0x10000017, 0x30000C03, 0x50AFABEA, 0x509AFEB2, 0x30004C99, 0x400A26AA, 0x10000003, 0x2000022E, 0x200001D1, 0x30001445, 0x300011C4, 0x10000018, 0x30001084, 0x10000010, 0x30000C98,
    0x30003A2E, 0x300043F2, 0x30004AB9, 0x30006EB1, 0x30007C87, 0x300026B7, 0x300076BF, 0x30007C21, 0x30006EBB, 0x30007EB7, 0x1000000A, 0x1000001A, 0x30004544, 0x4005294A, 0x30001151, 0x30000AA1,
    0x506ADE2E, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E, 0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
    0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F, 0x30006C9B, 0x30000F83, 0x30004EB9, 0x2000023F, 0x30006083, 0x200003F1, 0x30000822, 0x30004210,
    0x20000041, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E, 0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
    0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F, 0x30006C9B, 0x30000F83, 0x30004EB9, 0x30004764, 0x1000001F, 0x30001371, 0x50441044, 0x00000000,
];

fn measure_string(s: &str) -> i32 {
    s.chars().filter_map(|c| {
        let glyph = FONT.get(c as usize)?;
        let width = glyph >> 28;
        if width == 0 {
            None
        } else {
            Some(width as i32 + 1)
        }
    }).sum::<i32>() - 1
}

fn draw_character(left: i32, top: i32, c: char, color: u32) -> i32 {
    let glyph = FONT[c as usize];
    let width = (glyph >> 28) as i32;
    let mut glyph_data = glyph;

    for x in left..left + width {
        for y in top..top + 5 {
            if glyph_data & 1 == 1 {
                draw_pixel(x, y, color);
            }
            glyph_data >>= 1;
        }
    }

    width
}

fn draw_string(mut x: i32, y: i32, s: &str, color: u32) {
    for c in s.chars() {
        x += draw_character(x, y, c, color) + 1;
    }
}

fn run() {
    draw_string(1, 1, "Simple Text Editor", 0xC0C0C0);

    let mut cursor_x = 1;
    let cursor_y = 7;

    loop {
        wait(1);

        if let Some(key) = last_buffered_key() {
            draw_rectangle(cursor_x, cursor_y, 3, 5, 0x000000);

            let width = draw_character(cursor_x, cursor_y, key, 0x90EE90);

            if width > 0 {
                cursor_x += width + 1;
            }

            draw_rectangle(cursor_x, cursor_y, 3, 5, 0x008000);
        }
    }
}

fn draw_pixel(_x: i32, _y: i32, _color: u32) {}
fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: u32) {}
fn wait(_ms: u64) {}
fn last_buffered_key() -> Option<char> { None }

fn main() {
    run();
}