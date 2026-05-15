#[derive(PartialEq, Clone, Copy)]
struct Color(u8, u8, u8);

static HOUSE_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAACEAAAAeCAMAAACVFoclAAAAM1BMVEX/\
//////+yJwigPADrplWfMBdnNRBoOhf/r1C5fzrz4d/R0dFVMBWinJezs7OVlZXAurXSwoknAAAAAXRSTlMAQObYZgAAA\
LdJREFUeF6V0FkOgzAMBFCPnQ3odv/TtpmIglGtqCPxYfHiCcg1kEkAzMCNJDwMlBISVSUgiQRQe24FiEQZIYlETCjYER\
ZBNbMiKgJydqJeCFBzRq3nHkeAPhcXR/xLX7RXuLiieAN7SELAHhahxuEW2bZ7C7KkJC8ReUgKsrQmT5HP04J0wRzCmF/\
CTFXNdrGP/R4jyWzV1SwN8R3nO5zgoUNw/O8ee4Zw/4MBJl8LDAKcBNAFwRuJgw4oY9H/hQAAAABJRU5ErkJggg==";

static TREE_GIF: &str = "R0lGODlhDgAcAKIAAACNAP///6HFoaA8AA1/Df///wAAAAAAACH5BAUAA\
AUALAAAAAAOABwAAANVWLoV/AxAGOSM9iqQNe/T51wVIIAYQKDLx12ccALjwxErDd25brufR8nkqhVcBBmtNuTlFk2gz6\
UENmICHJYz1H5wT6l0MhhoFuWzIq1mn92kgPyRAAA7";

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const BLACK: Color = Color(0, 0, 0);
const WHITE: Color = Color(255, 255, 255);
const LIGHT_BLUE: Color = Color(173, 216, 230);

#[derive(Copy, Clone)]
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

fn random_int(min: i32, max: i32) -> i32 {
    // Placeholder for actual random int function
    min + (max - min) / 2
}

fn random_double() -> f64 {
    // Placeholder for actual random double function
    0.5
}

fn random_bool() -> bool {
    // Placeholder for actual random bool function
    true
}

fn make_color(r: i32, g: i32, b: i32) -> Color {
    Color(r as u8, g as u8, b as u8)
}

fn draw_pixel(_x: i32, _y: i32, _color: Color) {
    // Placeholder function for drawing a pixel
}

fn read_pixel(_x: i32, _y: i32) -> Color {
    // Placeholder function for reading a pixel
    Color(0, 0, 0)
}

fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _thickness: i32, _color: Color) {
    // Placeholder function for drawing a line
}

fn draw_image(_x: i32, _y: i32, _image_data: &str) {
    // Placeholder function for drawing an image
}

fn image_width(_image_data: &str) -> i32 {
    // Placeholder function for getting image width
    100
}

fn image_height(_image_data: &str) -> i32 {
    // Placeholder function for getting image height
    100
}

fn draw_string(_x: i32, _y: i32, _text: &str, _font_name: &str, _font_size: i32, _color: Color, _center: bool) {
    // Placeholder function for drawing a string
}

fn last_key() -> Option<char> {
    // Placeholder function for getting the last pressed key
    None
}

fn use_double_buffering(_enable: bool) {
    // Placeholder function for enabling or disabling double buffering
}

fn load_image(_data: &str) -> &str {
    // In a real application, this function would return a reference to image data
    "image_data"
}

fn clear(_color: Color) {
    // Placeholder function for clearing the screen
}

fn present() {
    // Placeholder function for presenting the rendered frame
}

fn close_window() {
    // Placeholder function for closing the window
}

fn wait(_ms: i32) {
    // Placeholder function for waiting
}

fn run() {
    use_double_buffering(true);
    let house = load_image(HOUSE_PNG);
    let tree = load_image(TREE_GIF);

    const MAX_FLAKES: usize = 300;

    let mut flakes: Vec<Flake> = Vec::with_capacity(MAX_FLAKES);

    let mut show_text = true;
    let mut delay = 30;
    loop {
        flakes.clear();

        clear(BLACK);
        draw_line(0, HEIGHT, WIDTH, HEIGHT, 6, random_gray());
        draw_image(
            WIDTH / 2 - random_int(0, image_width(house)),
            HEIGHT - image_height(house) - random_int(1, 3),
            house,
        );
        draw_image(
            1 * WIDTH / 4 + random_int(-30, 0),
            HEIGHT - image_height(tree) - random_int(1, 4),
            tree,
        );
        draw_image(
            3 * WIDTH / 4 + random_int(-5, 20),
            HEIGHT - image_height(tree) - random_int(1, 4),
            tree,
        );

        if show_text {
            draw_string(
                WIDTH / 2,
                20,
                "Happy\nHolidays!",
                "Arial",
                18,
                LIGHT_BLUE,
                true,
            );
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
                Some(_ESC) => {
                    close_window();
                    return;
                }
                _ => {}
            }

            delay = delay.clamp(5, 200);
            wait(delay);

            const CUTOFF_Y: i32 = 15;

            let mut lit_pixels = 0;
            for i in 0..WIDTH {
                if read_pixel(i, CUTOFF_Y) == BLACK {
                    break;
                }
                lit_pixels += 1;
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
                flakes.push(Flake {
                    c: random_gray(),
                    x,
                    y: 0,
                    speed,
                });
            }

            flakes.retain(|&f| {
                let mut f = f;
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

fn random_gray() -> Color {
    let albedo = random_int(205, 255);
    make_color(albedo, albedo, albedo)
}

fn main() {
    run();
}