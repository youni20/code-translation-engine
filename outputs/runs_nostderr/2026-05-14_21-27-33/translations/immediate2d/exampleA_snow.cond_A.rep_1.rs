use std::thread::sleep;
use std::time::Duration;

// Constants directly translated from the C++ code.
const HOUSE_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAACEAAAAeCAMAAACVFoclAAAAM1BMVEX/\
//////+yJwigPADrplWfMBdnNRBoOhf/r1C5fzrz4d/R0dFVMBWinJezs7OVlZXAurXSwoknAAAAAXRSTlMAQObYZgAAA\
LdJREFUeF6V0FkOgzAMBFCPnQ3odv/TtpmIglGtqCPxYfHiCcg1kEkAzMCNJDwMlBISVSUgiQRQe24FiEQZIYlETCjYER\
ZBNbMiKgJydqJeCFBzRq3nHkeAPhcXR/xLX7RXuLiieAN7SELAHhahxuEW2bZ7C7KkJC8ReUgKsrQmT5HP04J0wRzCmF/\
CTFXNdrGP/R4jyWzV1SwN8R3nO5zgoUNw/O8ee4Zw/4MBJl8LDAKcBNAFwRuJgw4oY9H/hQAAAABJRU5ErkJggg==";

const TREE_GIF: &str = "R0lGODlhDgAcAKIAAACNAP///6HFoaA8AA1/Df///wAAAAAAACH5BAUAA\
AUALAAAAAAOABwAAANVWLoV/AxAGOSM9iqQNe/T51wVIIAYQKDLx12ccALjwxErDd25brufR8nkqhVcBBmtNuTlFk2gz6\
UENmICHJYz1H5wT6l0MhhoFuWzIq1mn92kgPyRAAA7";

struct Flake {
    color: Color,
    x: usize,
    y: usize,
    speed: usize,
}

impl Flake {
    fn draw(&self) {
        draw_pixel(self.x, self.y, self.color);
    }
}

fn random_gray() -> Color {
    let albedo = random_int(205, 255) as u8;
    make_color(albedo, albedo, albedo)
}

fn run() -> Result<(), &'static str> {
    use_double_buffering(true);
    let house = load_image(HOUSE_PNG)?;
    let tree = load_image(TREE_GIF)?;

    const MAX_FLAKES: usize = 300;
    let mut flakes: Vec<Flake> = Vec::with_capacity(MAX_FLAKES);

    let mut show_text = true;
    let mut delay: i32 = 30;
    loop {
        flakes.clear();

        // Set up our initial winter scene
        clear(Color::Black);
        draw_line(0, HEIGHT, WIDTH, HEIGHT, 6, random_gray());
        draw_image(WIDTH / 2 - random_int(0, image_width(&house)), HEIGHT - image_height(&house) - random_int(1, 3), &house);
        draw_image(WIDTH / 4 + random_int(0, 30), HEIGHT - image_height(&tree) - random_int(1, 4), &tree);
        draw_image(3 * WIDTH / 4 + random_int(0, 20), HEIGHT - image_height(&tree) - random_int(1, 4), &tree);

        if show_text {
            draw_string(WIDTH / 2, 20, "Happy\nHolidays!", "Arial", 18, Color::LightBlue, true);
        }

        for y in 0..HEIGHT - 5 {
            for x in 0..WIDTH {
                if read_pixel(x, y) == Color::White {
                    draw_pixel(x, y, random_gray());
                }
            }
        }

        let mut running = true;
        while running {
            present();

            match last_key() {
                Some('+') => delay += 25,
                Some('-') => delay = delay.saturating_sub(25),
                Some('t') | Some('T') => {
                    show_text = !show_text;
                    running = false;
                },
                Some('r') | Some('R') => running = false,
                Some(Esc) => {
                    close_window();
                    return Ok(());
                },
                _ => {},
            }

            delay = clamp(delay, 5, 200);
            sleep(Duration::from_millis(delay as u64));

            const CUTOFF_Y: usize = 15;
            let mut lit_pixels = 0;
            for i in 0..WIDTH {
                if read_pixel(i, CUTOFF_Y) == Color::Black {
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
                if random_double() > SPAWN_CHANCE {
                    continue;
                }

                let x = random_int(0, WIDTH);
                let speed = random_int(1, 4);
                flakes.push(Flake { color: random_gray(), x, y: 0, speed });
            }

            for i in (0..flakes.len()).rev() {
                let flake = &mut flakes[i];
                draw_pixel(flake.x, flake.y, Color::Black);

                let mut landed = false;
                for _ in 0..flake.speed {
                    let left = flake.x > 0 && read_pixel(flake.x - 1, flake.y + 1) == Color::Black;
                    let down = read_pixel(flake.x, flake.y + 1) == Color::Black;
                    let right = flake.x < WIDTH - 1 && read_pixel(flake.x + 1, flake.y + 1) == Color::Black;

                    if down || left || right {
                        flake.y += 1;
                    } else {
                        landed = true;
                        break;
                    }

                    if !down {
                        if left && right {
                            flake.x = if random_bool() { flake.x + 1 } else { flake.x - 1 };
                        } else if left {
                            flake.x -= 1;
                        } else if right {
                            flake.x += 1;
                        }
                    }
                }
                flake.draw();

                if landed {
                    flakes.swap_remove(i);
                }
            }
        }
    }
}

// Stand-in functions and constants to simulate the behavior of the original C++ functions and definitions
const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const Esc: char = '\x1B';

#[derive(Copy, Clone, PartialEq, Eq)]
struct Color(u8, u8, u8);

impl Color {
    const Black: Color = Color(0, 0, 0);
    const White: Color = Color(255, 255, 255);
    const LightBlue: Color = Color(173, 216, 230); // Actual RGB value for LightBlue
}

fn draw_pixel(_: usize, _: usize, _: Color) {}
fn read_pixel(_: usize, _: usize) -> Color { Color::Black }
fn random_int(min: usize, max: usize) -> usize { min }
fn random_double() -> f64 { 0.0 }
fn random_bool() -> bool { false }
fn make_color(r: u8, g: u8, b: u8) -> Color { Color(r, g, b) }
fn draw_string(_: usize, _: usize, _: &str, _: &str, _: u8, _: Color, _: bool) {}
fn draw_line(_: usize, _: usize, _: usize, _: usize, _: u8, _: Color) {}
fn clear(_: Color) {}
fn load_image(_: &str) -> Result<Image, &'static str> { Ok(Image) }
fn image_width(_: &Image) -> usize { 100 }
fn image_height(_: &Image) -> usize { 100 }
struct Image;

fn clamp(value: i32, min: i32, max: i32) -> i32 { min.max(value.min(max)) }
fn present() {}
fn close_window() {}
fn last_key() -> Option<char> { Some(' ') }
fn use_double_buffering(_: bool) {}
fn draw_image(_: usize, _: usize, _img: &Image) {}

fn main() {
    run().unwrap_or_else(|err| eprintln!("Error: {}", err));
}