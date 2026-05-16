use std::cmp::min;
use std::thread;
use std::time::Duration;

const HOUSE_PNG: &str = concat!(
    "iVBORw0KGgoAAAANSUhEUgAAACEAAAAeCAMAAACVFoclAAAAM1BMVEX/",
    "//////+yJwigPADrplWfMBdnNRBoOhf/r1C5fzrz4d/R0dFVMBWinJezs7OVlZXAurXSwoknAAAAAXRSTlMAQObYZgAAA"
);

const TREE_GIF: &str = concat!(
    "R0lGODlhDgAcAKIAAACNAP///6HFoaA8AA1/Df///wAAAAAAACH5BAUAA",
    "AUALAAAAAAOABwAAANVWLoV/AxAGOSM9iqQNe/T51wVIIAYQKDLx12ccALjwxErDd25brufR8nkqhVcBBmtNuTlFk2gz6"
);

#[derive(PartialEq, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

const BLACK: Color = Color { r: 0, g: 0, b: 0 };
const WHITE: Color = Color { r: 255, g: 255, b: 255 };
const LIGHT_BLUE: Color = Color { r: 173, g: 216, b: 230 };

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
    let albedo = 205 + (rand() % 51);
    Color::rgb(albedo as u8, albedo as u8, albedo as u8)
}

fn clamp(val: i32, min_val: i32, max_val: i32) -> i32 {
    min(max_val, val.max(min_val))
}

fn random_double() -> f64 {
    rand() as f64 / u32::MAX as f64
}

fn random_int(min: i32, max: i32) -> i32 {
    min + (rand() % (max - min + 1))
}

fn random_bool() -> bool {
    rand() % 2 == 0
}

fn rand() -> i32 {
    // Placeholder for actual random number generation
    42
}

fn use_double_buffering(_enabled: bool) {
    // Placeholder function
}

fn load_image(_image_data: &str) -> i32 {
    // Placeholder function for image handle
    1
}

fn clear(_color: Color) {
    // Placeholder function
}

fn draw_pixel(_x: i32, _y: i32, _color: Color) {
    // Placeholder function
}

fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _thickness: i32, _color: Color) {
    // Placeholder function
}

fn draw_image(_x: i32, _y: i32, _image: i32) {
    // Placeholder function
}

fn draw_string(_x: i32, _y: i32, _text: &str, _font: &str, _size: i32, _color: Color, _centered: bool) {
    // Placeholder function
}

fn read_pixel(_x: i32, _y: i32) -> Color {
    // Placeholder function
    BLACK
}

fn present() {
    // Placeholder function
}

fn last_key() -> Option<char> {
    // Placeholder function
    None
}

fn close_window() {
    // Placeholder function
}

fn image_width(_image: &i32) -> i32 {
    // Placeholder function
    100
}

fn image_height(_image: &i32) -> i32 {
    // Placeholder function
    100
}

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fn run() {
    use_double_buffering(true);
    let house = load_image(HOUSE_PNG);
    let tree = load_image(TREE_GIF);

    const MAX_FLAKES: usize = 300;

    let mut flakes = Vec::with_capacity(MAX_FLAKES);

    let mut show_text = true;
    let mut delay: i32 = 30;

    loop {
        flakes.clear();

        clear(BLACK);
        draw_line(0, HEIGHT, WIDTH, HEIGHT, 6, random_gray());
        draw_image(WIDTH / 2 - random_int(0, image_width(&house)), HEIGHT - image_height(&house) - random_int(1, 3), house);
        draw_image(WIDTH / 4 + random_int(-30, 0), HEIGHT - image_height(&tree) - random_int(1, 4), tree);
        draw_image(3 * WIDTH / 4 + random_int(-5, 20), HEIGHT - image_height(&tree) - random_int(1, 4), tree);

        if show_text {
            draw_string(WIDTH / 2, 20, "Happy\nHolidays!", "Arial", 18, LIGHT_BLUE, true);
        }

        for y in 0..HEIGHT - 5 {
            for x in 0..WIDTH {
                if read_pixel(x, y) == WHITE {
                    draw_pixel(x, y, random_gray());
                }
            }
        }

        let mut running = true;
        while running {
            present();

            match last_key() {
                Some('+') => delay += 25,
                Some('-') => delay -= 25,
                Some('t') | Some('T') => {
                    show_text = !show_text;
                    running = false;
                }
                Some('r') | Some('R') => running = false,
                Some('\x1B') => {
                    close_window();
                    return;
                }
                _ => (),
            }

            delay = clamp(delay, 5, 200);
            thread::sleep(Duration::from_millis(delay as u64));

            const CUTOFF_Y: i32 = 15;

            let mut lit_pixels = 0;
            for i in 0..WIDTH {
                if read_pixel(i, CUTOFF_Y) == BLACK {
                    break;
                } else {
                    lit_pixels += 1;
                }
            }

            if lit_pixels == WIDTH {
                break;
            }

            for _ in 0..(MAX_FLAKES - flakes.len()) {
                const SPAWN_CHANCE: f64 = 0.01;
                if random_double() <= SPAWN_CHANCE {
                    let x = random_int(0, WIDTH);
                    let speed = random_int(1, 4);
                    flakes.push(Flake { c: random_gray(), x, y: 0, speed });
                }
            }

            flakes.retain_mut(|f| {
                draw_pixel(f.x, f.y, BLACK);

                let mut landed = false;
                for _ in 0..f.speed {
                    let left = read_pixel(f.x - 1, f.y + 1) == BLACK && f.x > 0;
                    let down = read_pixel(f.x, f.y + 1) == BLACK;
                    let right = read_pixel(f.x + 1, f.y + 1) == BLACK && f.x < WIDTH - 1;

                    if down || left || right {
                        f.y += 1;
                    } else {
                        landed = true;
                        break;
                    }

                    if !down {
                        if left && right {
                            f.x += if random_bool() { 1 } else { -1 };
                        } else if left {
                            f.x -= 1;
                        } else if right {
                            f.x += 1;
                        }
                    }
                }
                f.draw();

                !landed
            });
        }
    }
}

fn main() {
    run();
}