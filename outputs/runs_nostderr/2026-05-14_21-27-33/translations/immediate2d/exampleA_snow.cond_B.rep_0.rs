use std::vec::Vec;
use std::cmp::{min, max};

// Assuming these functions are implemented elsewhere in the code
fn draw_pixel(_x: usize, _y: usize, _color: Color) {}
fn random_double() -> f64 { 0.0 }
fn random_bool() -> bool { true }
fn last_key() -> Option<char> { None }
fn clear(_color: Color) {}
fn draw_line(_x1: usize, _y1: usize, _x2: usize, _y2: usize, _width: usize, _color: Color) {}
fn draw_image(_x: usize, _y: usize, _image_data: &str) {}
fn draw_string(_x: usize, _y: usize, _text: &str, _font: &str, _size: usize, _color: Color, _centered: bool) {}
fn read_pixel(_x: usize, _y: usize) -> Color { Color::Black }
fn wait(_ms: usize) {}
fn present() {}
fn use_double_buffering(_enable: bool) {}
fn close_window() {}
fn load_image(image_data: &str) -> &str { image_data }
fn image_width(_image_data: &str) -> usize { 100 }
fn image_height(_image_data: &str) -> usize { 100 }
fn make_color(_r: u8, _g: u8, _b: u8) -> Color { Color::Black }

const WIDTH: usize = 800;  // Assume appropriate dimensions
const HEIGHT: usize = 600; // Assume appropriate dimensions

#[derive(Clone, Copy, PartialEq)]
enum Color {
    Black,
    White,
    LightBlue,
    // Other colors...
}

// If your images are small, you can avoid the usual problems of copying the
// files around with your program (or dealing with compiling and linking
// Visual Studio's .RC resource format) by simply embedding the image data
// directly in your code.
//
// Encoding the file as a Base64 string (search for "Base64 Encoder" online),
// we can pass the string directly to Immediate2D's image loading function and
// it will figure things out on its own, automatically.
//
// These images were only a couple hundred bytes, so it makes sense and isn't
// too messy here.  You can shrink PNGs substantially without any loss by using
// tools like pngout, which you can find here: http://advsys.net/ken/utils.htm

static HOUSE_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAACEAAAAeCAMAAACVFoclAAAAM1BMVEX/\
    //////+yJwigPADrplWfMBdnNRBoOhf/r1C5fzrz4d/R0dFVMBWinJezs7OVlZXAurXSwoknAAAAAXRSTlMAQObYZgAAA\
    LdJREFUeF6V0FkOgzAMBFCPnQ3odv/TtpmIglGtqCPxYfHiCcg1kEkAzMCNJDwMlBISVSUgiQRQe24FiEQZIYlETCjYER\
    ZBNbMiKgJydqJeCFBzRq3nHkeAPhcXR/xLX7RXuLiieAN7SELAHhahxuEW2bZ7C7KkJC8ReUgKsrQmT5HP04J0wRzCmF/\
    CTFXNdrGP/R4jyWzV1SwN8R3nO5zgoUNw/O8ee4Zw/4MBJl8LDAKcBNAFwRuJgw4oY9H/hQAAAABJRU5ErkJggg==";

static TREE_GIF: &str = "R0lGODlhDgAcAKIAAACNAP///6HFoaA8AA1/Df///wAAAAAAACH5BAUAA\
    AUALAAAAAAOABwAAANVWLoV/AxAGOSM9iqQNe/T51wVIIAYQKDLx12ccALjwxErDd25brufR8nkqhVcBBmtNuTlFk2gz6\
    UENmICHJYz1H5wT6l0MhhoFuWzIq1mn92kgPyRAAA7";

struct Flake {
    c: Color,
    x: usize,
    y: usize,
    speed: i32,
}

impl Flake {
    fn draw(&self) {
        draw_pixel(self.x, self.y, self.c);
    }
}

// We generate snow flake colors in more than one place, so
// this is pulled out as a separate function for consistency.
fn random_gray() -> Color {
    let albedo = (random_double() * 50.0) as u8 + 205;
    make_color(albedo, albedo, albedo)
}

fn run() {
    use_double_buffering(true);
    let house = load_image(HOUSE_PNG);
    let tree = load_image(TREE_GIF);

    const MAX_FLAKES: usize = 300;

    let mut flakes: Vec<Flake> = Vec::with_capacity(MAX_FLAKES);

    let mut show_text = true;
    let mut delay: i32 = 30;
    loop {
        flakes.clear();

        // Set up our initial winter scene
        clear(Color::Black);
        draw_line(0, HEIGHT, WIDTH, HEIGHT, 6, random_gray());
        draw_image(WIDTH / 2 - ((random_double() * image_width(&house) as f64) as usize),
                   HEIGHT - image_height(&house) - (random_double() * 3.0).floor() as usize,
                   &house);
        draw_image(WIDTH / 4 + (random_double() * 30.0) as isize as usize,
                   HEIGHT - image_height(&tree) - (random_double() * 4.0).floor() as usize,
                   &tree);
        draw_image(3 * WIDTH / 4 + (random_double() * 5.0 + 20.0).floor() as isize as usize,
                   HEIGHT - image_height(&tree) - (random_double() * 4.0).floor() as usize,
                   &tree);

        if show_text {
            draw_string(WIDTH / 2, 20, "Happy\nHolidays!", "Arial", 18, Color::LightBlue, true);
        }

        // We use solid white pixels in our images to denote snow (because the
        // image data compresses better with a single color) but the demo looks
        // better if those solid white pixels are randomized to the same gray
        // scale as the rest of our snow flakes.
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
                }
                Some('r') | Some('R') => running = false,
                Some('\u{1b}') => { // Escape key
                    close_window();
                    return;
                }
                _ => {}
            }

            delay = min(200, max(5, delay));
            wait(delay as usize);

            const CUTOFF_Y: usize = 15;

            // Reset if the snow has built up to the top of the screen.
            let lit_pixels = (0..WIDTH).take_while(|&i| read_pixel(i, CUTOFF_Y) != Color::Black).count();

            if lit_pixels == WIDTH {
                break;
            }

            // Spawn new flakes randomly, with more chances to
            // spawn, the fewer active flakes we currently have.
            for _ in 0..(MAX_FLAKES.saturating_sub(flakes.len())) {
                const SPAWN_CHANCE: f64 = 0.01;
                if random_double() > SPAWN_CHANCE {
                    continue;
                }

                let x = (random_double() * WIDTH as f64).floor() as usize;
                let speed = (random_double() * 4.0 + 1.0).floor() as i32;
                flakes.push(Flake { c: random_gray(), x, y: 0, speed });
            }

            let mut i = 0;
            while i < flakes.len() {
                let mut f = &mut flakes[i];
                draw_pixel(f.x, f.y, Color::Black);

                // Step it down one tick for each "speed".
                let mut landed = false;
                for _ in 0..f.speed {
                    let left = read_pixel(f.x.saturating_sub(1), f.y + 1) == Color::Black && f.x > 0;
                    let down = read_pixel(f.x, f.y + 1) == Color::Black;
                    let right = read_pixel(f.x + 1, f.y + 1) == Color::Black && f.x < WIDTH - 1;

                    // If there's anywhere to go, we always move down a pixel.
                    if down || left || right {
                        f.y += 1;
                    } else {
                        landed = true;
                        break;
                    }

                    // If we can't move straight down, we have to decide
                    // which direction to roll (using a simple coin toss).
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
                    // We need to remove this flake from the list.  But, to avoid
                    // shimmying all the elements in the list after this one up one
                    // to fill in the hole, we just grab the flake at the end of the
                    // list to fill it, then shrink the list by one.  (This works
                    // because our flakes don't have any sort of ordering or inter-
                    // dependence constraints.)
                    //
                    // The i-- steps our loop back one so we don't forget to process
                    // the flake that just took this one's place.
                    flakes[i] = flakes.pop().unwrap();
                } else {
                    i += 1;
                }
            }
        }
    }
}

fn main() {
    run();
}