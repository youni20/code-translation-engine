const WIDTH: usize = 1920; // Example dimension
const HEIGHT: usize = 1080; // Example dimension

// The +2 in each dimension is for the boundary walls that are just off-screen
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
                x[id(i, j)] = (x0[id(i, j)] + a * (x[id(i - 1, j)] + x[id(i + 1, j)] + x[id(i, j - 1)] + x[id(i, j + 1)])) / c;
            }
        }

        set_boundary(b, x);
    }
}

fn diffuse(b: i32, x: &mut Vec<f32>, x0: &Vec<f32>, diffusion: f32, dt: f32) {
    let a = dt * diffusion * WIDTH as f32 * HEIGHT as f32;
    linear_solve(b, x, x0, a, 1.0 + 4.0 * a);
}

fn advect(b: i32, d: &mut Vec<f32>, d0: &Vec<f32>, u: &Vec<f32>, v: &Vec<f32>, dt: f32) {
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
            d[id(i, j)] = s0 * (t0 * d0[id(i0, j0)] + t1 * d0[id(i0, j1)]) + s1 * (t0 * d0[id(i1, j0)] + t1 * d0[id(i1, j1)]);
        }
    }

    set_boundary(b, d);
}

fn project(u: &mut Vec<f32>, v: &mut Vec<f32>, p: &mut Vec<f32>, div: &mut Vec<f32>) {
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

fn density_step(x: &mut Vec<f32>, x0: &mut Vec<f32>, u: &Vec<f32>, v: &Vec<f32>, diffusion: f32, dt: f32) {
    for i in 0..SIZE {
        x[i] += dt * x0[i];
    }
    diffuse(0, x0, x, diffusion, dt);
    advect(0, x, x0, u, v, dt);
}

fn velocity_step(u: &mut Vec<f32>, v: &mut Vec<f32>, u0: &mut Vec<f32>, v0: &mut Vec<f32>, viscosity: f32, dt: f32) {
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

fn present(_screen: &[Color]) {
    // A stub for the "Present" function which must be implemented as per rendering backend
}

#[derive(Copy, Clone)]
struct Color(u8, u8, u8);

fn make_color_hsb(_h: i32, _s: i32, _b: i32) -> Color {
    // A stub for the "make color hsb" which should convert HSB to RGB
    Color(0, 0, 0) // Placeholder
}

fn fluid_color(u: f32, v: f32, density: f32, show_velocity: bool) -> Color {
    if show_velocity {
        let h = f32::min(360.0, f32::max(0.0, (u * u + v * v).sqrt() * 1500.0)) as i32;
        let v = f32::min(255.0, f32::max(0.0, density * 500.0)) as i32;
        return make_color_hsb(h, 255, v);
    }

    let value = f32::min(360.0, f32::max(0.0, density * 100.0)) as i32;
    make_color_hsb(f32::max(0.0, value as f32 - 310.0) as i32, value / 2, value)
}

fn line(x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dy == 0 {
        for x in i32::min(x1, x2)..=i32::max(x1, x2) {
            result.push((x, y1));
        }
        return result;
    }
    if dx == 0 {
        for y in i32::min(y1, y2)..=i32::max(y1, y2) {
            result.push((x1, y));
        }
        return result;
    }

    if dx.abs() > dy.abs() { // Is the line X or Y-major?
        let slope = dy as f64 / dx.abs() as f64;
        let mut y = y1 as f64;
        for i in 0..=dx.abs() {
            let x = if x1 > x2 { x1 - i } else { x1 + i };
            result.push((x, y as i32));
            y += slope;
        }
    } else {
        let slope = dx as f64 / dy.abs() as f64;
        let mut x = x1 as f64;
        for j in 0..=dy.abs() {
            let y = if y1 > y2 { y1 - j } else { y1 + j };
            result.push((x as i32, y));
            x += slope;
        }
    }

    result
}

fn mouse_drag(field: &mut Vec<f32>, x1: i32, y1: i32, x2: i32, y2: i32, value: f32) {
    let points = line(x1, y1, x2, y2);
    for p in &points {
        field[id(p.0 as usize, p.1 as usize)] = value / points.len() as f32;
    }
}

fn draw_string(density: &mut Vec<f32>, y: usize, s: &str) {
    const FONT: [u32; 96] = [
        0x10000000, 0x10000017, 0x30000C03, 0x50AFABEA, 0x509AFEB2, 0x30004C99, 0x400A26AA, 0x10000003,
        0x2000022E, 0x200001D1, 0x30001445, 0x300011C4, 0x10000018, 0x30001084, 0x10000010, 0x30000C98,
        0x30003A2E, 0x300043F2, 0x30004AB9, 0x30006EB1, 0x30007C87, 0x300026B7, 0x300076BF, 0x30007C21,
        0x30006EBB, 0x30007EB7, 0x1000000A, 0x1000001A, 0x30004544, 0x4005294A, 0x30001151, 0x30000AA1,
        0x506ADE2E, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E,
        0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
        0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F,
        0x30006C9B, 0x30000F83, 0x30004EB9, 0x30004764, 0x1000001F, 0x30001371, 0x50441044, 0x00000000,
    ];

    let text_width: usize = s.bytes()
        .filter(|&c| c >= 32 && c <= 127)
        .map(|c| ((FONT[c as usize - 32] >> 28) + 1) as usize)
        .sum();

    let mut x = (WIDTH - text_width) / 2;

    for ch in s.bytes() {
        if ch < 32 || ch > 127 { continue; }
        let mut glyph = FONT[ch as usize - 32];
        let width = (glyph >> 28) as usize;
        for u in x..(x + width) {
            for v in y..(y + 5) {
                if (glyph & 1) == 1 {
                    density[id(u, v)] = 3.0;
                }
                glyph >>= 1;
            }
        }
        if width > 0 { x += width + 1; }
    }
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
    let mut screen = vec![Color(0, 0, 0); WIDTH * HEIGHT];

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
        std::thread::sleep(std::time::Duration::from_millis(1));

        for i in 0..SIZE {
            u_prev[i] = 0.0;
            v_prev[i] = 0.0;
            density_prev[i] = 0.0;
        }

        let key = ' '; // A placeholder for key inputs
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
        if false { // Placeholder: close window
            break;
        }

        let m_x = 0 + 1; // Placeholder for mouse x
        let m_y = 0 + 1; // Placeholder for mouse y

        let mouse_is_down = false; // Placeholder for mouse input
        if mouse_is_down && !mouse_was_down {
            down_x = m_x;
            down_y = m_y;
        }
        mouse_was_down = mouse_is_down;

        if mouse_is_down && m_x >= 1 && m_x <= WIDTH && m_y >= 1 && m_y <= HEIGHT {
            if false { // Placeholder for left mouse button
                mouse_drag(&mut u_prev, down_x, down_y, m_x, m_y, 10.0 * (m_x - down_x) as f32);
                mouse_drag(&mut v_prev, down_x, down_y, m_x, m_y, 10.0 * (m_y - down_y) as f32);
            }

            if false { // Placeholder for right mouse button
                mouse_drag(&mut density_prev, down_x, down_y, m_x, m_y, 100.0);
            }

            down_x = m_x;
            down_y = m_y;
        }

        velocity_step(&mut u, &mut v, &mut u_prev, &mut v_prev, viscosity, dt);
        density_step(&mut density, &mut density_prev, &u, &v, diffusion, dt);

        for j in 0..HEIGHT {
            for i in 0..WIDTH {
                screen[j * WIDTH + i] = fluid_color(u[id((i + 1).try_into().unwrap(), (j + 1).try_into().unwrap())], v[id((i + 1).try_into().unwrap(), (j + 1).try_into().unwrap())], density[id((i + 1).try_into().unwrap(), (j + 1).try_into().unwrap())], show_velocity);
            }
        }

        present(&screen);
    }
}

fn main() {
    run();
}