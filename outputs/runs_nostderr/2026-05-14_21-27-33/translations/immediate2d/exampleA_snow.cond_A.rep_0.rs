use std::time::Duration;
use std::thread::sleep;
use std::convert::TryInto;

// Immediate2D replacement functions (stubs for actual implementation)
fn use_double_buffering(_enabled: bool) {}
fn load_image(_base64: &str) -> Vec<u8> { vec![] }  // Placeholder for image
fn draw_pixel(_x: i32, _y: i32, _color: Color) {}
fn clear(_color: Color) {}
fn draw_line(_x: i32, _y: i32, _x2: i32, _y2: i32, _thickness: i32, _color: Color) {}
fn draw_image(_x: i32, _y: i32, _image: &[u8]) {}
fn draw_string(_x: i32, _y: i32, _text: &str, _font: &str, _size: i32, _color: Color, _centered: bool) {}
fn read_pixel(_x: i32, _y: i32) -> Color { Color(0, 0, 0) } // returns Black
fn present() {}
fn last_key() -> Option<char> { None }
fn close_window() {}
fn wait(milliseconds: i32) { sleep(Duration::from_millis(milliseconds as u64)); }

#[derive(PartialEq, Clone, Copy)]
struct Color(u8, u8, u8);

impl Color {
    const BLACK: Color = Color(0, 0, 0);
    const WHITE: Color = Color(255, 255, 255);
    const LIGHT_BLUE: Color = Color(173, 216, 230);

    fn new(r: u8, g: u8, b: u8) -> Self {
        Color(r, g, b)
    }
}

struct Flake {
    c: Color,
    x: i32,
    y: i32,
    speed: i32,
}

impl Flake {
    fn draw(&self) {
        draw_pixel(self.x, self.y, self.c);
    }
}

fn random_gray() -> Color {
    let albedo: u8 = (205..=255).into_iter().next().unwrap_or(205);
    Color::new(albedo, albedo, albedo)
}

fn random_int(min: i32, max: i32) -> i32 {
    min + (max - min) / 2 // simple random replacement
}

fn random_double() -> f64 {
    0.5 // simple random replacement
}

fn random_bool() -> bool {
    true // simple random replacement
}

fn run() {
    use_double_buffering(true);
    let house = load_image(HOUSE_PNG);
    let tree = load_image(TREE_GIF);

    const MAX_FLAKES: usize = 300;

    let mut flakes = Vec::with_capacity(MAX_FLAKES);

    let mut show_text = true;
    let mut delay = 30;
    loop {
        flakes.clear();

        clear(Color::BLACK);
        draw_line(0, HEIGHT.try_into().unwrap(), WIDTH.try_into().unwrap(), HEIGHT.try_into().unwrap(), 6, random_gray());
        draw_image(<u32 as TryInto<i32>>::try_into(WIDTH / 2).unwrap() - random_int(0, house.len() as i32), HEIGHT as i32 - house.len() as i32 - random_int(1, 3), &house);
        draw_image(<u32 as TryInto<i32>>::try_into(1 * WIDTH / 4).unwrap() + random_int(-30, 0), HEIGHT as i32 - tree.len() as i32 - random_int(1, 4), &tree);
        draw_image(<u32 as TryInto<i32>>::try_into(3 * WIDTH / 4).unwrap() + random_int(-5, 20), HEIGHT as i32 - tree.len() as i32 - random_int(1, 4), &tree);

        if show_text {
            draw_string(<u32 as TryInto<i32>>::try_into(WIDTH / 2).unwrap(), 20, "Happy\nHolidays!", "Arial", 18, Color::LIGHT_BLUE, true);
        }

        for y in 0..(HEIGHT as i32 - 5) {
            for x in 0..WIDTH as i32 {
                if read_pixel(x, y) == Color::WHITE {
                    draw_pixel(x, y, random_gray());
                }
            }
        }

        let mut running = true;
        while running {
            present();

            if let Some(key) = last_key() {
                match key {
                    '+' => delay += 25,
                    '-' => delay -= 25,
                    't' | 'T' | 'r' | 'R' => {
                        show_text = !show_text;
                        running = false;
                    }
                    '\u{1B}' => {  // Esc key
                        close_window();
                        return;
                    }
                    _ => {}
                }
            }

            delay = delay.clamp(5, 200);
            wait(delay);

            const CUTOFF_Y: i32 = 15;

            let mut lit_pixels = 0;
            for x in 0..WIDTH as i32 {
                if read_pixel(x, CUTOFF_Y) == Color::BLACK { break; }
                else { lit_pixels += 1; }
            }

            if lit_pixels == WIDTH as i32 { break; }

            for _ in 0..(MAX_FLAKES - flakes.len()) {
                const SPAWN_CHANCE: f64 = 0.01;
                if random_double() > SPAWN_CHANCE { continue; }

                let x = random_int(0, WIDTH as i32);
                let speed = random_int(1, 4);
                flakes.push(Flake { c: random_gray(), x, y: 0, speed });
            }

            for i in (0..flakes.len()).rev() {
                let mut f = &mut flakes[i];
                draw_pixel(f.x, f.y, Color::BLACK);

                let mut landed = false;
                for _ in 0..f.speed {
                    let left = read_pixel(f.x - 1, f.y + 1) == Color::BLACK && f.x > 0;
                    let down = read_pixel(f.x, f.y + 1) == Color::BLACK;
                    let right = read_pixel(f.x + 1, f.y + 1) == Color::BLACK && f.x < WIDTH as i32 - 1;

                    if down || left || right { f.y += 1; } else {
                        landed = true;
                        break;
                    }

                    if !down {
                        if left && right { f.x += if random_bool() { 1 } else { -1 }; }
                        else if left { f.x -= 1; }
                        else if right { f.x += 1; }
                    }
                }
                f.draw();

                if landed {
                    flakes[i] = flakes.pop().unwrap(); // replace with the last element
                }
            }
        }
    }
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

static HOUSE_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAACEAAAAeCAMAAACVFoclAAAAM1BMVEX/\
/////+yJwigPADrplWfMBdnNRBoOhf/r1C5fzrz4d/R0dFVMBWinJezs7OVlZXAurXSwoknAAAAAXRSTlMAQObYZgAAA\
LdJREFUeF6V0FkOgzAMBFCPnQ3odv/TtpmIglGtqCPxYfHiCcg1kEkAzMCNJDwMlBISVSUgiQRQe24FiEQZIYlETCjYER\
ZBNbMiKgJydqJeCFBzRq3nHkeAPhcXR/xLX7RXuLiieAN7SELAHhahxuEW2bZ7C7KkJC8ReUgKsrQmT5HP04J0wRzCmF/\
CTFXNdrGP/R4jyWzV1SwN8R3nO5zgoUNw/O8ee4Zw/4MBJl8LDAKcBNAFwRuJgw4oY9H/hQAAAABJRU5ErkJggg==";

static TREE_GIF: &str = "R0lGODlhDgAcAKIAAACNAP///6HFoaA8AA1/Df///wAAAAAAACH5BAUAA\
AUALAAAAAAOABwAAANVWLoV/AxAGOSM9iqQNe/T51wVIIAYQKDLx12ccALjwxErDd25brufR8nkqhVcBBmtNuTlFk2gz6\
UENmICHJYz1H5wT6l0MhhoFuWzIq1mn92kgPyRAAA7";

fn main() {
    run();
}