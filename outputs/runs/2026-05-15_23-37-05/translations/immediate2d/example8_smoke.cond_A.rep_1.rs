const WIDTH: usize = 100; // Define WIDTH
const HEIGHT: usize = 100; // Define HEIGHT
const SIZE: usize = (WIDTH + 2) * (HEIGHT + 2);

fn id(i: usize, j: usize) -> usize {
    (WIDTH + 2) * j + i
}

fn set_boundary(b: i32, x: &mut [f32]) {
    for j in 1..=HEIGHT {
        x[id(0, j)] = if b == 1 { -x[id(1, j)] } else { x[id(1, j)] };
        x[id(WIDTH + 1, j)] = if b == 1 { -x[id(WIDTH, j)] } else { x[id(WIDTH, j)] };
    }

    for i in 1..=WIDTH {
        x[id(i, 0)] = if b == 2 { -x[id(i, 1)] } else { x[id(i, 1)] };
        x[id(i, HEIGHT + 1)] = if b == 2 { -x[id(i, HEIGHT)] } else { x[id(i, HEIGHT)] };
    }

    x[id(0, 0)] = 0.5 * (x[id(1, 0)] + x[id(0, 1)]);
    x[id(0, HEIGHT + 1)] = 0.5 * (x[id(1, HEIGHT + 1)] + x[id(0, HEIGHT)]);
    x[id(WIDTH + 1, 0)] = 0.5 * (x[id(WIDTH, 0)] + x[id(WIDTH + 1, 1)]);
    x[id(WIDTH + 1, HEIGHT + 1)] = 0.5 * (x[id(WIDTH, HEIGHT + 1)] + x[id(WIDTH + 1, HEIGHT)]);
}

fn linear_solve(b: i32, x: &mut [f32], x0: &[f32], a: f32, c: f32) {
    for _ in 0..20 {
        for j in 1..=HEIGHT {
            for i in 1..=WIDTH {
                x[id(i, j)] = (x0[id(i, j)] + a * (x[id(i-1, j)] + x[id(i+1, j)] + x[id(i, j-1)] + x[id(i, j+1)])) / c;
            }
        }

        set_boundary(b, x);
    }
}

fn diffuse(b: i32, x: &mut [f32], x0: &[f32], diffusion: f32, dt: f32) {
    let a = dt * diffusion * WIDTH as f32 * HEIGHT as f32;
    linear_solve(b, x, x0, a, 1.0 + 4.0 * a);
}

fn advect(b: i32, d: &mut [f32], d0: &[f32], u: &[f32], v: &[f32], dt: f32) {
    let dt0 = dt * HEIGHT as f32;
    for j in 1..=HEIGHT {
        for i in 1..=WIDTH {
            let mut x = i as f32 - dt0 * u[id(i, j)];
            let mut y = j as f32 - dt0 * v[id(i, j)];

            if x < 0.5 { x = 0.5; }
            if x > WIDTH as f32 + 0.5 { x = WIDTH as f32 + 0.5; }
            let i0 = x as usize;
            let i1 = i0 + 1;

            if y < 0.5 { y = 0.5; }
            if y > HEIGHT as f32 + 0.5 { y = HEIGHT as f32 + 0.5; }
            let j0 = y as usize;
            let j1 = j0 + 1;

            let s1 = x - i0 as f32;
            let s0 = 1.0 - s1;
            let t1 = y - j0 as f32;
            let t0 = 1.0 - t1;
            d[id(i, j)] = s0 * (t0 * d0[id(i0, j0)] + t1 * d0[id(i0, j1)])
                        + s1 * (t0 * d0[id(i1, j0)] + t1 * d0[id(i1, j1)]);
        }
    }

    set_boundary(b, d);
}

fn project(u: &mut [f32], v: &mut [f32], p: &mut [f32], div: &mut [f32]) {
    for j in 1..=HEIGHT {
        for i in 1..=WIDTH {
            div[id(i, j)] = -0.5 * (u[id(i+1, j)] - u[id(i-1, j)] + v[id(i, j+1)] - v[id(i, j-1)]) / HEIGHT as f32;
            p[id(i, j)] = 0.0;
        }
    }

    set_boundary(0, div);
    set_boundary(0, p);
    linear_solve(0, p, div, 1.0, 4.0);
    for j in 1..=HEIGHT {
        for i in 1..=WIDTH {
            u[id(i, j)] -= 0.5 * HEIGHT as f32 * (p[id(i+1, j)] - p[id(i-1, j)]);
            v[id(i, j)] -= 0.5 * HEIGHT as f32 * (p[id(i, j+1)] - p[id(i, j-1)]);
        }
    }

    set_boundary(1, u);
    set_boundary(2, v);
}

fn density_step(x: &mut [f32], x0: &mut [f32], u: &[f32], v: &[f32], diffusion: f32, dt: f32) {
    for i in 0..SIZE {
        x[i] += dt * x0[i];
    }
    diffuse(0, x0, x, diffusion, dt);
    advect(0, x, x0, u, v, dt);
}

fn velocity_step(u: &mut [f32], v: &mut [f32], u0: &mut [f32], v0: &mut [f32], viscosity: f32, dt: f32) {
    for i in 0..SIZE {
        u[i] += dt * u0[i];
        v[i] += dt * v0[i];
    }
    diffuse(1, u0, u, viscosity, dt);
    diffuse(2, v0, v, viscosity, dt);
    project(u0, v0, u, v);
    advect(1, u, u0, u0, v0, dt);
    advect(2, v, v0, u0, v0, dt);
    project(u, v, u0, v0);
}

// Helper function to draw a line of pixels between two points
fn line(x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dy == 0 {
        (std::cmp::min(x1, x2)..=std::cmp::max(x1, x2)).for_each(|x| result.push((x, y1)));
        return result;
    }
    if dx == 0 {
        (std::cmp::min(y1, y2)..=std::cmp::max(y1, y2)).for_each(|y| result.push((x1, y)));
        return result;
    }

    if dx.abs() > dy.abs() {
        let slope = dy as f64 / dx.abs() as f64;
        let mut y = y1 as f64;
        let dir_x = if x1 > x2 { -1 } else { 1 };
        for x in (0..=dx.abs()).map(|i| x1 + i * dir_x) {
            result.push((x, y as i32));
            y += slope;
        }
    } else {
        let slope = dx as f64 / dy.abs() as f64;
        let mut x = x1 as f64;
        let dir_y = if y1 > y2 { -1 } else { 1 };
        for y in (0..=dy.abs()).map(|j| y1 + j * dir_y) {
            result.push((x as i32, y));
            x += slope;
        }
    }
    result
}

fn mouse_drag(field: &mut [f32], x1: i32, y1: i32, x2: i32, y2: i32, value: f32) {
    let points = line(x1, y1, x2, y2);
    for p in &points {
        field[id(p.0 as usize, p.1 as usize)] = value / points.len() as f32;
    }
}

// Stub functions to replace unimplemented functionality
fn present(_screen: &[Color]) {
    // Replace with drawing code
}

fn last_key() -> Option<char> {
    None // Replace with actual key handling
}

fn wait(_milliseconds: u64) {
    // Replace with actual wait functionality
}

fn mouse_x() -> i32 {
    0 // Replace with actual mouse handling
}

fn mouse_y() -> i32 {
    0 // Replace with actual mouse handling
}

fn left_mouse_pressed() -> bool {
    false // Replace with actual mouse button handling
}

fn right_mouse_pressed() -> bool {
    false // Replace with actual mouse button handling
}

fn close_window() {
    // Replace with actual window closing functionality
}

#[derive(Clone)]
struct Color; // Placeholder for Color type

fn fluid_color(_u: f32, _v: f32, _density: f32, _show_velocity: bool) -> Color {
    // Placeholder for MakeColorHSB implementation
    Color
}

fn draw_string(_density: &mut [f32], _y: usize, _s: &str) {
    // Placeholder for drawing text
}

fn run() {
    let dt = 0.1;
    let diffusion = 0.0;
    let viscosity = 0.0;
    let mut u = vec![0.0; SIZE];
    let mut v = vec![0.0; SIZE];
    let mut u_prev = vec![0.0; SIZE];
    let mut v_prev = vec![0.0; SIZE];
    let mut density = vec![0.0; SIZE];
    let mut density_prev = vec![0.0; SIZE];
    let mut screen = vec![Color; WIDTH * HEIGHT];

    draw_string(&mut density, HEIGHT / 7, "Left mouse drag to move air");
    draw_string(&mut density, 2 * HEIGHT / 7, "Right mouse drag to add smoke");
    draw_string(&mut density, 3 * HEIGHT / 7, "Holding both is the most fun!");
    draw_string(&mut density, 5 * HEIGHT / 7, "Use spacebar to toggle velocity view");
    draw_string(&mut density, 6 * HEIGHT / 7, "(Press 'C' to clear the screen)");

    let mut show_velocity = false;
    let mut mouse_was_down = false;
    let mut down_x = -1;
    let mut down_y = -1;

    loop {
        wait(1);

        for i in 0..SIZE {
            u_prev[i] = 0.0;
            v_prev[i] = 0.0;
            density_prev[i] = 0.0;
        }

        if let Some(key) = last_key() {
            if key == 'c' {
                for i in 0..SIZE {
                    u[i] = 0.0;
                    v[i] = 0.0;
                    u_prev[i] = 0.0;
                    v_prev[i] = 0.0;
                    density[i] = 0.0;
                    density_prev[i] = 0.0;
                }
            } else if key == ' ' {
                show_velocity = !show_velocity;
            } else if key == '\u{1b}' {
                close_window();
                break;
            }
        }

        let m_x = mouse_x() + 1;
        let m_y = mouse_y() + 1;

        let mouse_is_down = left_mouse_pressed() || right_mouse_pressed();
        if mouse_is_down && !mouse_was_down {
            down_x = m_x;
            down_y = m_y;
        }
        mouse_was_down = mouse_is_down;

        if mouse_is_down && m_x >= 1 && m_x <= WIDTH as i32 && m_y >= 1 && m_y <= HEIGHT as i32 {
            if left_mouse_pressed() {
                mouse_drag(&mut u_prev, down_x, down_y, m_x, m_y, 10.0 * (m_x - down_x) as f32);
                mouse_drag(&mut v_prev, down_x, down_y, m_x, m_y, 10.0 * (m_y - down_y) as f32);
            }

            if right_mouse_pressed() {
                mouse_drag(&mut density_prev, down_x, down_y, m_x, m_y, 100.0);
            }

            down_x = m_x;
            down_y = m_y;
        }

        velocity_step(&mut u, &mut v, &mut u_prev, &mut v_prev, viscosity, dt);
        density_step(&mut density, &mut density_prev, &u, &v, diffusion, dt);

        for j in 0..HEIGHT {
            for i in 0..WIDTH {
                screen[j * WIDTH + i] = fluid_color(u[id(i+1, j+1)], v[id(i+1, j+1)], density[id(i+1, j+1)], show_velocity);
            }
        }

        present(&screen);
    }
}

fn main() {
    run();
}