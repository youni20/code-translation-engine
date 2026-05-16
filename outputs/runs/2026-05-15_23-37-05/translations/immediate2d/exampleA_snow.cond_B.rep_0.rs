#![allow(dead_code)]

use std::collections::VecDeque;

const HOUSE_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAACEAAAAeCAMAAACVFoclAAAAM1BMVEX/\
    //////+yJwigPADrplWfMBdnNRBoOhf/r1C5fzrz4d/R0dFVMBWinJezs7OVlZXAurXSwoknAAAAAXRSTlMAQObYZgAAA\
    LdJREFUeF6V0FkOgzAMBFCPnQ3odv/TtpmIglGtqCPxYfHiCcg1kEkAzMCNJDwMlBISVSUgiQRQe24FiEQZIYlETCjYER\
    ZBNbMiKgJydqJeCFBzRq3nHkeAPhcXR/xLX7RXuLiieAN7SELAHhahxuEW2bZ7C7KkJC8ReUgKsrQmT5HP04J0wRzCmF/\
    CTFXNdrGP/R4jyWzV1SwN8R3nO5zgoUNw/O8ee4Zw/4MBJl8LDAKcBNAFwRuJgw4oY9H/hQAAAABJRU5ErkJggg==";

const TREE_GIF: &str = "R0lGODlhDgAcAKIAAACNAP///6HFoaA8AA1/Df///wAAAAAAACH5BAUAA\
    AUALAAAAAAOABwAAANVWLoV/AxAGOSM9iqQNe/T51wVIIAYQKDLx12ccALjwxErDd25brufR8nkqhVcBBmtNuTlFk2gz6\
    UENmICHJYz1H5wT6l0MhhoFuWzIq1mn92kgPyRAAA7";

#[derive(Clone, Copy, PartialEq)]
struct Color(u8, u8, u8);

#[derive(Clone, Copy)]
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

struct Image;

// Placeholder function implementations
fn load_image(_image_data: &str) -> Image {
    Image
}

fn draw_pixel(_x: i32, _y: i32, _color: Color) {}

fn random_int(min: i32, max: i32) -> i32 {
    // Placeholder for random integer generator
    min + (max - min) / 2  // Dummy implementation
}

fn random_double() -> f64 {
    // Placeholder for random float [0, 1) generator
    0.5  // Dummy implementation
}

fn random_bool() -> bool {
    random_double() < 0.5
}

fn make_color(r: u8, g: u8, b: u8) -> Color {
    Color(r, g, b)
}

fn random_gray() -> Color {
    let albedo = random_int(205, 255) as u8;
    make_color(albedo, albedo, albedo)
}

fn wait(_time: i32) {}

fn read_pixel(_x: i32, _y: i32) -> Color {
    Color(0, 0, 0) // Black as placeholder
}

fn use_double_buffering(_value: bool) {}

fn clear(_color: Color) {}

fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _width: i32, _color: Color) {}

fn draw_image(_x: i32, _y: i32, _image: &Image) {}

fn draw_string(_x: i32, _y: i32, _text: &str, _font: &str, _size: i32, _color: Color, _centered: bool) {}

fn present() {}

fn last_key() -> Option<char> {
    None // Placeholder key reading function
}

fn close_window() {}

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn run() {
    use_double_buffering(true);
    let house = load_image(HOUSE_PNG);
    let tree = load_image(TREE_GIF);

    const MAX_FLAKES: usize = 300;

    let mut flakes: VecDeque<Flake> = VecDeque::new();
    flakes.reserve(MAX_FLAKES);

    let mut show_text = true;
    let mut delay = 30;
    loop {
        flakes.clear();

        clear(make_color(0, 0, 0)); // Black
        draw_line(0, Height, Width, Height, 6, random_gray());
        draw_image(Width / 2 - random_int(0, 1), Height - 1 - random_int(1, 3), &house);
        draw_image(1 * Width / 4 + random_int(-30, 0), Height - 1 - random_int(1, 4), &tree);
        draw_image(3 * Width / 4 + random_int(-5, 20), Height - 1 - random_int(1, 4), &tree);

        if show_text {
            draw_string(Width / 2, 20, "Happy\nHolidays!", "Arial", 18, make_color(173, 216, 230), true);
        }

        for y in 0..Height - 5 {
            for x in 0..Width {
                if read_pixel(x, y) == make_color(255, 255, 255) {
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
                    't' | 'T' => {
                        show_text = !show_text;
                        running = false;
                    }
                    'r' | 'R' => running = false,
                    _ => {}
                }
            }

            delay = clamp(delay, 5, 200);
            wait(delay);

            const CUTOFF_Y: i32 = 15;

            let mut lit_pixels = 0;
            for i in 0..Width {
                if read_pixel(i, CUTOFF_Y) == make_color(0, 0, 0) {
                    break;
                } else {
                    lit_pixels += 1;
                }
            }

            if lit_pixels == Width {
                break;
            }

            for _ in 0..((MAX_FLAKES - flakes.len()) as i32) {
                const SPAWN_CHANCE: f64 = 0.01;
                if random_double() > SPAWN_CHANCE {
                    continue;
                }

                let x = random_int(0, Width);
                let speed = random_int(1, 4);
                flakes.push_back(Flake {
                    c: random_gray(),
                    x,
                    y: 0,
                    speed,
                });
            }

            for i in 0..flakes.len() {
                let mut flake = flakes[i];
                draw_pixel(flake.x, flake.y, make_color(0, 0, 0));

                let mut landed = false;
                for _ in 0..flake.speed {
                    let left = read_pixel(flake.x - 1, flake.y + 1) == make_color(0, 0, 0) && flake.x > 0;
                    let down = read_pixel(flake.x, flake.y + 1) == make_color(0, 0, 0);
                    let right = read_pixel(flake.x + 1, flake.y + 1) == make_color(0, 0, 0) && flake.x < Width - 1;

                    if down || left || right {
                        flake.y += 1;
                    } else {
                        landed = true;
                        break;
                    }

                    if !down {
                        if left && right {
                            flake.x += if random_bool() { 1 } else { -1 };
                        } else if left {
                            flake.x -= 1;
                        } else if right {
                            flake.x += 1;
                        }
                    }
                }

                flakes[i] = flake;
                flake.draw();

                if landed {
                    flakes.swap_remove_back(i);
                }
            }
        }
    }
}

const Width: i32 = 640;
const Height: i32 = 480;

fn main() {
    run();
}