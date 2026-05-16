use std::cmp::{min, max};

// Constants
const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const SIZE: usize = (WIDTH + 2) * (HEIGHT + 2);

fn id(i: usize, j: usize) -> usize {
    (WIDTH + 2) * j + i
}

fn set_boundary(b: usize, x: &mut [f32]) {
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

fn linear_solve(b: usize, x: &mut [f32], x0: &[f32], a: f32, c: f32) {
    for _ in 0..20 {
        for j in 1..=HEIGHT {
            for i in 1..=WIDTH {
                x[id(i, j)] = (x0[id(i, j)] + a * (x[id(i - 1, j)] + x[id(i + 1, j)] + x[id(i, j - 1)] + x[id(i, j + 1)])) / c;
            }
        }
        set_boundary(b, x);
    }
}

fn diffuse(b: usize, x: &mut [f32], x0: &[f32], diffusion: f32, dt: f32) {
    let a = dt * diffusion * WIDTH as f32 * HEIGHT as f32;
    linear_solve(b, x, x0, a, 1.0 + 4.0 * a);
}

fn advect(b: usize, d: &mut [f32], d0: &[f32], u: &[f32], v: &[f32], dt: f32) {
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

            d[id(i, j)] = s0 * (t0 * d0[id(i0, j0)] + t1 * d0[id(i0, j1)]) +
                          s1 * (t0 * d0[id(i1, j0)] + t1 * d0[id(i1, j1)]);
        }
    }
    set_boundary(b, d);
}

fn project(u: &mut [f32], v: &mut [f32], p: &mut [f32], div: &mut [f32]) {
    for j in 1..=HEIGHT {
        for i in 1..=WIDTH {
            div[id(i, j)] = -0.5 * (u[id(i + 1, j)] - u[id(i - 1, j)] + v[id(i, j + 1)] - v[id(i, j - 1)]) / HEIGHT as f32;
            p[id(i, j)] = 0.0;
        }
    }
    set_boundary(0, div);
    set_boundary(0, p);
    linear_solve(0, p, div, 1.0, 4.0);

    for j in 1..=HEIGHT {
        for i in 1..=WIDTH {
            u[id(i, j)] -= 0.5 * HEIGHT as f32 * (p[id(i + 1, j)] - p[id(i - 1, j)]);
            v[id(i, j)] -= 0.5 * HEIGHT as f32 * (p[id(i, j + 1)] - p[id(i, j - 1)]);
        }
    }
    set_boundary(1, u);
    set_boundary(2, v);
}

fn density_step(x: &mut [f32], x0: &[f32], u: &[f32], v: &[f32], diffusion: f32, dt: f32) {
    for i in 0..SIZE {
        x[i] += dt * x0[i];
    }
    diffuse(0, x, x0, diffusion, dt);
    advect(0, x, x0, u, v, dt);
}

fn velocity_step(u: &mut [f32], v: &mut [f32], u0: &[f32], v0: &[f32], viscosity: f32, dt: f32) {
    for i in 0..SIZE {
        u[i] += dt * u0[i];
        v[i] += dt * v0[i];
    }
    let mut u0_mut = u0.to_vec();
    let mut v0_mut = v0.to_vec();
    diffuse(1, &mut u0_mut, u, viscosity, dt);
    diffuse(2, &mut v0_mut, v, viscosity, dt);
    project(&mut u0_mut, &mut v0_mut, u, v);
    advect(1, u, &u0_mut, &u0_mut, &v0_mut, dt);
    advect(2, v, &v0_mut, &u0_mut, &v0_mut, dt);
    project(u, v, &mut u0_mut, &mut v0_mut);
}

// Dummy implementations for missing parts to make the code compile.
#[derive(Clone)]
struct Color; // Placeholder for Color type

fn make_color_hsb(_h: i32, _s: i32, _v: i32) -> Color { Color }

fn present(_screen: &[Color]) {}

fn wait(_time: u64) {}

fn last_key() -> char { ' ' }

fn close_window() {}

fn mouse_x() -> i32 { 1 }

fn mouse_y() -> i32 { 1 }

fn left_mouse_pressed() -> bool { false }

fn right_mouse_pressed() -> bool { false }

fn fluid_color(u: f32, v: f32, density: f32, show_velocity: bool) -> Color {
    if show_velocity {
        let h = min(360, max(0, (1500.0 * (u * u + v * v).sqrt()).round() as i32));
        let v = min(255, max(0, (density * 500.0).round() as i32));
        return make_color_hsb(h, 255, v);
    }
    let value = min(360, max(0, (density * 100.0).round() as i32));
    make_color_hsb(max(0, value - 310), value / 2, value)
}

fn line(x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dy == 0 {
        for x in min(x1, x2)..=max(x1, x2) {
            result.push((x, y1));
        }
        return result;
    }
    if dx == 0 {
        for y in min(y1, y2)..=max(y1, y2) {
            result.push((x1, y));
        }
        return result;
    }

    if dx.abs() > dy.abs() {
        let slope = dy as f64 / dx.abs() as f64;
        let mut y = y1 as f64;
        for i in 0..=dx.abs() {
            let x = if x1 > x2 { x1 - i } else { x1 + i };
            result.push((x, y.round() as i32));
            y += slope;
        }
    } else {
        let slope = dx as f64 / dy.abs() as f64;
        let mut x = x1 as f64;
        for j in 0..=dy.abs() {
            let y = if y1 > y2 { y1 - j } else { y1 + j };
            result.push((x.round() as i32, y));
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

fn draw_string(density: &mut [f32], y: usize, s: &str) {
    const FONT: [u32; 128 - 32] = [
        0x10000000, 0x10000017, 0x30000C03, 0x50AFABEA, 0x509AFEB2, 0x30004C99, 0x400A26AA, 0x10000003,
        0x2000022E, 0x200001D1, 0x30001445, 0x300011C4, 0x10000018, 0x30001084, 0x10000010, 0x30000C98,
        0x30003A2E, 0x300043F2, 0x30004AB9, 0x30006EB1, 0x30007C87, 0x300026B7, 0x300076BF, 0x30007C21,
        0x30006EBB, 0x30007EB7, 0x1000000A, 0x1000001A, 0x30004544, 0x4005294A, 0x30001151, 0x30000AA1,
        0x506ADE2E, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E,
        0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
        0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F,
        0x30006C9B, 0x30000F83, 0x30004EB9, 0x2000023F, 0x30006083, 0x200003F1, 0x30000822, 0x30004210,
        0x20000041, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E,
        0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
        0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F,
        0x30006C9B, 0x30000F83, 0x30004EB9, 0x30004764, 0x1000001F, 0x30001371, 0x50441044, 0x00000000,
    ];

    let text_width: usize = s.chars().map(|c| if c < ' ' || c > '~' { 0 } else { ((FONT[(c as usize) - 32] >> 28) + 1) as usize }).sum();
    let mut x = (WIDTH - text_width) / 2;

    for c in s.chars() {
        if c < ' ' || c > '~' {
            continue;
        }
        let glyph = FONT[(c as usize) - 32];
        let width = glyph >> 28;
        for u in x..x + width as usize {
            for v in y..y + 5 {
                if ((glyph >> ((u - x) * 5 + v - y)) & 1) != 0 {
                    density[id(u, v)] = 3.0;
                }
            }
        }
        x += if width > 0 { (width + 1) as usize } else { 0 };
    }
}

fn run() {
    let dt = 0.1;
    let diffusion = 0.0;
    let viscosity = 0.0;
    
    let mut u = vec![0.0_f32; SIZE];
    let mut v = vec![0.0_f32; SIZE];
    let mut u_prev = vec![0.0_f32; SIZE];
    let mut v_prev = vec![0.0_f32; SIZE];
    let mut density = vec![0.0_f32; SIZE];
    let mut density_prev = vec![0.0_f32; SIZE];
    let mut screen = vec![Color; WIDTH * HEIGHT];

    draw_string(&mut density, 1 * HEIGHT / 7, "Left mouse drag to move air");
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

        let key = last_key();
        if key == 'c' {
            for i in 0..SIZE {
                u[i] = 0.0;
                v[i] = 0.0;
                u_prev[i] = 0.0;
                v_prev[i] = 0.0;
                density[i] = 0.0;
                density_prev[i] = 0.0;
            }
        }
        if key == ' ' {
            show_velocity = !show_velocity;
        }
        if key == '\u{1b}' { // Escape key
            close_window();
        }

        let mx = mouse_x() + 1;
        let my = mouse_y() + 1;

        let mouse_is_down = left_mouse_pressed() || right_mouse_pressed();
        if mouse_is_down && !mouse_was_down {
            down_x = mx;
            down_y = my;
        }
        mouse_was_down = mouse_is_down;

        if mouse_is_down && mx >= 1 && mx <= WIDTH as i32 && my >= 1 && my <= HEIGHT as i32 {
            if left_mouse_pressed() {
                mouse_drag(&mut u_prev, down_x, down_y, mx, my, 10.0 * (mx - down_x) as f32);
                mouse_drag(&mut v_prev, down_x, down_y, mx, my, 10.0 * (my - down_y) as f32);
            }
            if right_mouse_pressed() {
                mouse_drag(&mut density_prev, down_x, down_y, mx, my, 100.0);
            }
            down_x = mx;
            down_y = my;
        }

        velocity_step(&mut u, &mut v, &u_prev, &v_prev, viscosity, dt);
        density_step(&mut density, &density_prev, &u, &v, diffusion, dt);

        for j in 0..HEIGHT {
            for i in 0..WIDTH {
                let idx = j * WIDTH + i;
                let color = fluid_color(u[id(i + 1, j + 1)], v[id(i + 1, j + 1)], density[id(i + 1, j + 1)], show_velocity);
                screen[idx] = color;
            }
        }
        
        present(&screen);
    }
}

fn main() {
    run();
}