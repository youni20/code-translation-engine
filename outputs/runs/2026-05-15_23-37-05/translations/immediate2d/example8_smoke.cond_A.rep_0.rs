const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SIZE: usize = (WIDTH + 2) * (HEIGHT + 2);

fn id(i: usize, j: usize) -> usize {
    (WIDTH + 2) * j + i
}

fn set_boundary(b: i32, x: &mut Vec<f32>) {
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

fn linear_solve(b: i32, x: &mut Vec<f32>, x0: &Vec<f32>, a: f32, c: f32) {
    for _ in 0..20 {
        for j in 1..=HEIGHT {
            for i in 1..=WIDTH {
                x[id(i, j)] = (x0[id(i, j)]
                    + a * (x[id(i - 1, j)] + x[id(i + 1, j)] + x[id(i, j - 1)] + x[id(i, j + 1)]))
                    / c;
            }
        }
        set_boundary(b, x);
    }
}

fn diffuse(b: i32, x: &mut Vec<f32>, x0: &Vec<f32>, diffusion: f32, dt: f32) {
    let a = dt * diffusion * WIDTH as f32 * HEIGHT as f32;
    linear_solve(b, x, x0, a, 1.0 + 4.0 * a);
}

fn advect(
    b: i32,
    d: &mut Vec<f32>,
    d0: &Vec<f32>,
    u: &Vec<f32>,
    v: &Vec<f32>,
    dt: f32,
) {
    let dt0 = dt * HEIGHT as f32;
    for j in 1..=HEIGHT {
        for i in 1..=WIDTH {
            let mut x = i as f32 - dt0 * u[id(i, j)];
            let mut y = j as f32 - dt0 * v[id(i, j)];

            if x < 0.5 { x = 0.5; }
            if x > (WIDTH as f32 + 0.5) { x = WIDTH as f32 + 0.5; }
            let i0 = x as usize;
            let i1 = i0 + 1;

            if y < 0.5 { y = 0.5; }
            if y > (HEIGHT as f32 + 0.5) { y = HEIGHT as f32 + 0.5; }
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

fn project(u: &mut Vec<f32>, v: &mut Vec<f32>, p: &mut Vec<f32>, div: &mut Vec<f32>) {
    for j in 1..=HEIGHT {
        for i in 1..=WIDTH {
            div[id(i, j)] = -0.5
                * (u[id(i + 1, j)] - u[id(i - 1, j)] + v[id(i, j + 1)] - v[id(i, j - 1)])
                / HEIGHT as f32;
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

fn density_step(
    x: &mut Vec<f32>,
    x0: &mut Vec<f32>,
    u: &Vec<f32>,
    v: &Vec<f32>,
    diffusion: f32,
    dt: f32,
) {
    for i in 0..SIZE {
        x[i] += dt * x0[i];
    }
    diffuse(0, x0, x, diffusion, dt);
    advect(0, x, x0, u, v, dt);
}

fn velocity_step(
    u: &mut Vec<f32>,
    v: &mut Vec<f32>,
    u0: &mut Vec<f32>,
    v0: &mut Vec<f32>,
    viscosity: f32,
    dt: f32,
) {
    for i in 0..SIZE {
        u[i] += dt * u0[i];
    }
    for i in 0..SIZE {
        v[i] += dt * v0[i];
    }
    diffuse(1, u0, u, viscosity, dt);
    diffuse(2, v0, v, viscosity, dt);
    project(u0, v0, u, v);
    advect(1, u, u0, u0, v0, dt);
    advect(2, v, v0, u0, v0, dt);
    project(u, v, u0, v0);
}

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

fn present(_screen: &[Color]) {}

fn make_color_hsb(h: i32, s: i32, v: i32) -> Color {
    Color { r: h as u8, g: s as u8, b: v as u8 }
}

fn fluid_color(u: f32, v: f32, density: f32, show_velocity: bool) -> Color {
    if show_velocity {
        let h = ((u * u + v * v).sqrt() * 1500.0)
            .min(360.0)
            .max(0.0) as i32;
        let v = (density * 500.0)
            .min(255.0)
            .max(0.0) as i32;
        return make_color_hsb(h, 255, v);
    }

    let value = (density * 100.0).min(360.0).max(0.0) as i32;
    make_color_hsb((value - 310).max(0), value / 2, value)
}

fn line(x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(i32, i32)> {
    let mut result = vec![];
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dy == 0 {
        for x in x1.min(x2)..=x1.max(x2) {
            result.push((x, y1));
        }
        return result;
    }
    if dx == 0 {
        for y in y1.min(y2)..=y1.max(y2) {
            result.push((x1, y));
        }
        return result;
    }

    if dx.abs() > dy.abs() {
        let slope = dy as f64 / dx.abs() as f64;
        let mut y = y1 as f64;
        for (_i, x) in (x1..).take((dx.abs() + 1) as usize).enumerate() {
            result.push((x, y.round() as i32));
            y += slope;
        }
        return result;
    }

    let slope = dx as f64 / dy.abs() as f64;
    let mut x = x1 as f64;
    for (_j, y) in (y1..).take((dy.abs() + 1) as usize).enumerate() {
        result.push((x.round() as i32, y));
        x += slope;
    }
    result
}

fn mouse_drag(field: &mut Vec<f32>, x1: i32, y1: i32, x2: i32, y2: i32, value: f32) {
    let points = line(x1, y1, x2, y2);
    let len = points.len() as f32;
    for (x, y) in &points {
        field[id(*x as usize, *y as usize)] = value / len;
    }
}

fn draw_string(density: &mut Vec<f32>, y: usize, s: &str) {
    static mut FONT: [u32; 128 - 32] = [
        0x10000000, 0x10000017, 0x30000C03, 0x50AFABEA, 0x509AFEB2, 
        0x30004C99, 0x400A26AA, 0x10000003, 0x2000022E, 0x200001D1, 
        0x30001445, 0x300011C4, 0x10000018, 0x30001084, 0x10000010, 
        0x30000C98, 0x30003A2E, 0x300043F2, 0x30004AB9, 0x30006EB1, 
        0x30007C87, 0x300026B7, 0x300076BF, 0x30007C21, 0x30006EBB, 
        0x30007EB7, 0x1000000A, 0x1000001A, 0x30004544, 0x4005294A, 
        0x30001151, 0x30000AA1, 0x506ADE2E, 0x300078BE, 0x30002ABF, 
        0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E, 
        0x30007C9F, 0x1000001F, 0x30003E08, 0x30003E08, 0x30006C9F, 
        0x3000421F, 0x51F1105F, 0x51F1105F, 0x51F4105F, 0x4007462E, 
        0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 
        0x30007E1F, 0x30003E0F, 0x50F8320F, 0x30006C9B, 0x30000F83, 
        0x30004EB9, 0x30004764, 0x1000001F, 0x30001371, 0x50441044, 
        0x00000000, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF,
        0x300004BF, 0x3000662E, 0x30007C9F, 0x1000001F, 0x30003E08, 
        0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E, 
        0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 
        0x30007E1F, 0x30003E0F, 0x50F8320F, 0x30006C9B, 0x30000F83, 
        0x30004EB9, 0x30004764, 0x1000001F, 0x30001371, 0x50441044, 
        0x00000000,
    ];

    let text_width = s.chars().map(|c| (unsafe { FONT[c as usize - 32] } >> 28) + 1).sum::<u32>() as usize;
    let mut x = (WIDTH - text_width) / 2;
    for c in s.chars() {
        if c < ' ' || c > '~' {
            continue;
        }
        let mut glyph = unsafe { FONT[c as usize - 32] };
        let width = (glyph >> 28) as usize;
        for u in x..x + width {
            for v in y..y + 5 {
                if glyph & 1 == 1 {
                    density[id(u, v)] = 3.0;
                }
                glyph >>= 1;
            }
        }
        if width > 0 {
            x += width + 1;
        }
    }
}

fn run() {
    loop {
        // Placeholder
    }
}

fn main() {
    run();
}