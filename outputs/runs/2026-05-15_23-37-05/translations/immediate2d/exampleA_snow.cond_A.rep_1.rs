use std::cmp;
use std::thread::sleep;
use std::time::Duration;
use std::collections::VecDeque;

#[derive(Clone, PartialEq)]
struct Color(u8, u8, u8);

fn draw_pixel(x: usize, y: usize, color: Color) {
    // Functionality to draw a pixel at (x, y) with the color 'color'
}

fn clear(color: Color) {
    // Functionality to clear the screen with the color 'color'
}

fn draw_line(x1: usize, y1: usize, x2: usize, y2: usize, thickness: usize, color: Color) {
    // Functionality to draw a line from (x1, y1) to (x2, y2)
}

fn draw_image(x: usize, y: usize, image: &Image) {
    // Functionality to draw an image
}

fn draw_string(x: usize, y: usize, text: &str, font: &str, size: usize, color: Color, centered: bool) {
    // Functionality to draw a string at (x, y)
}

fn read_pixel(x: usize, y: usize) -> Color {
    // Functionality to read the pixel color at (x, y)
    Color(0, 0, 0) // Placeholder
}

fn present() {
    // Present the drawn contents on the screen
}

fn wait(ms: u64) {
    sleep(Duration::from_millis(ms));
}

fn last_key() -> Option<char> {
    // Return the last key pressed
    None
}

fn close_window() {
    // Close the window
}

fn load_image(_base64_str: &str) -> Image {
    // Load an image from a base64 encoded string
    Image
}

#[derive(Clone)]
struct Image;

#[derive(Clone)]
struct Flake {
    c: Color,
    x: usize,
    y: usize,
    speed: usize,
}

impl Flake {
    fn draw(&self) {
        draw_pixel(self.x, self.y, self.c.clone());
    }
}

fn random_gray() -> Color {
    let albedo: u8 = 205 + (rand_u8() % 51);
    Color(albedo, albedo, albedo)
}

fn run() {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 600;
    const HOUSE_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAACEAAAAeCAMAAACVFoclAAAAM1BMVEX/////+yJwigPADrplWfMBdnNRBoOhf/r1C5fzrz4d/R0dFVMBWinJezs7OVlZXAurXSwoknAAAAAXRSTlMAQObYZgAAALdJREFUeF6V0FkOgzAMBFCPnQ3odv/TtpmIglGtqCPxYfHiCcg1kEkAzMCNJDwMlBISVSUgiQRQe24FiEQZIYlETCjYERZBNbMiKgJydqJeCFBzRq3nHkeAPhcXR/xLX7RXuLiieAN7SELAHhahxuEW2bZ7C7KkJC8ReUgKsrQmT5HP04J0wRzCmF/CTFXNdrGP/R4jyWzV1SwN8R3nO5zgoUNw/O8ee4Zw/4MBJl8LDAKcBNAFwRuJgw4oY9H/hQAAAABJRU5ErkJggg==";
    const TREE_GIF: &str = "R0lGODlhDgAcAKIAAACNAP///6HFoaA8AA1/Df///wAAAAAAACH5BAUAA AUALAAAAAAOABwAAANVWLoV/AxAGOSM9iqQNe/T51wVIIAYQKDLx12ccALjwxErDd25brufR8nkqhVcBBmtNuTlFk2gz6UENmICHJYz1H5wT6l0MhhoFuWzIq1mn92kgPyRAAA7";
    const MAX_FLAKES: usize = 300;

    use_double_buffering(true);

    let house = load_image(HOUSE_PNG);
    let tree = load_image(TREE_GIF);

    let mut flakes = VecDeque::with_capacity(MAX_FLAKES);

    let mut show_text = true;
    let mut delay = 30;
    loop {
        flakes.clear();

        clear(Color(0, 0, 0));
        draw_line(0, HEIGHT, WIDTH, HEIGHT, 6, random_gray());
        let house_width = width_of(&house);
        let house_height = height_of(&house);
        draw_image(WIDTH / 2 - random_int(0, house_width as i32) as usize, HEIGHT - house_height - random_int(1, 3) as usize, &house);
        let tree_width = width_of(&tree);
        let tree_height = height_of(&tree);
        draw_image(WIDTH / 4 + random_int(-30, 0) as usize, HEIGHT - tree_height - random_int(1, 4) as usize, &tree);
        draw_image(3 * WIDTH / 4 + random_int(-5, 20) as usize, HEIGHT - tree_height - random_int(1, 4) as usize, &tree);

        if show_text {
            draw_string(WIDTH / 2, 20, "Happy\nHolidays!", "Arial", 18, Color(173, 216, 230), true);
        }

        for y in 0..HEIGHT - 5 {
            for x in 0..WIDTH {
                if read_pixel(x, y) == Color(255, 255, 255) {
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
                    '\u{001B}' => {
                        close_window();
                        return;
                    }
                    _ => {}
                }
            }

            delay = cmp::max(5, cmp::min(delay, 200));
            wait(delay as u64);

            const CUTOFF_Y: usize = 15;

            let mut lit_pixels = 0;
            for i in 0..WIDTH {
                if read_pixel(i, CUTOFF_Y) == Color(0, 0, 0) {
                    break;
                } else {
                    lit_pixels += 1;
                }
            }

            if lit_pixels == WIDTH {
                break;
            }

            let chances = MAX_FLAKES.saturating_sub(flakes.len()) as i32;
            for _ in 0..chances {
                if random_float() > 0.01 {
                    continue;
                }
                let x = rand_usize(0, WIDTH);
                let speed = rand_usize(1, 5);
                flakes.push_back(Flake { c: random_gray(), x, y: 0, speed });
            }

            let mut i = 0;
            while i < flakes.len() {
                let mut f = flakes[i].clone();
                draw_pixel(f.x, f.y, Color(0, 0, 0));

                let mut landed = false;
                for _ in 0..f.speed {
                    let left = f.x > 0 && read_pixel(f.x - 1, f.y + 1) == Color(0, 0, 0);
                    let down = read_pixel(f.x, f.y + 1) == Color(0, 0, 0);
                    let right = f.x < WIDTH - 1 && read_pixel(f.x + 1, f.y + 1) == Color(0, 0, 0);

                    if down || left || right {
                        f.y += 1;
                    } else {
                        landed = true;
                        break;
                    }

                    if !down {
                        if left && right {
                            f.x = if random_bool() { f.x + 1 } else { f.x - 1 };
                        } else if left {
                            f.x -= 1;
                        } else if right {
                            f.x += 1;
                        }
                    }
                }
                f.draw();

                if landed {
                    flakes.swap_remove_back(i);
                } else {
                    flakes[i] = f;
                    i += 1;
                }
            }
        }
    }
}

fn main() {
    run();
}

fn use_double_buffering(_enabled: bool) {}

fn width_of(_image: &Image) -> usize { 100 }
fn height_of(_image: &Image) -> usize { 100 }

fn random_int(min: i32, max: i32) -> i32 {
    min + (rand_u8() % ((max - min + 1) as u8)) as i32
}

fn rand_u8() -> u8 {
    205 // Dummy implementation, in real cases it should provide more randomness
}

fn rand_usize(min: usize, max: usize) -> usize {
    min + (rand_u8() as usize % (max - min))
}

fn random_float() -> f64 {
    0.01 // Dummy implementation, in real cases it should provide more randomness
}

fn random_bool() -> bool {
    true // Dummy implementation, in real cases it should provide more randomness
}