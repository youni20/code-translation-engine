use std::f32::consts::PI;
use std::f32;
use std::vec::Vec;

pub const VERSION: &str = "1.7.0 (21/01/2026)";

pub struct DefaultPRNG {
    seed_: u32,
}

impl Default for DefaultPRNG {
    fn default() -> Self {
        Self { seed_: 7133167 }
    }
}

impl DefaultPRNG {
    pub fn new(seed: u32) -> Self {
        Self { seed_: seed }
    }

    pub fn random_float(&mut self) -> f32 {
        self.seed_ = self.seed_.wrapping_mul(521167);
        let a: u32 = (self.seed_ & 0x007fffff) | 0x40000000;
        let f: f32 = f32::from_bits(a);
        0.5 * (f - 2.0)
    }

    pub fn random_int(&mut self, max_int: u32) -> u32 {
        (self.random_float() * max_int as f32) as u32
    }

    pub fn get_seed(&self) -> u32 {
        self.seed_
    }
}

#[derive(Clone, Copy)]
pub struct Point {
    x: f32,
    y: f32,
    valid_: bool,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x,
            y: y,
            valid_: true,
        }
    }

    pub fn is_in_rectangle(&self) -> bool {
        self.x >= 0.0 && self.y >= 0.0 && self.x <= 1.0 && self.y <= 1.0
    }

    pub fn is_in_circle(&self) -> bool {
        let fx = self.x - 0.5;
        let fy = self.y - 0.5;
        (fx * fx + fy * fy) <= 0.25
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
            valid_: self.valid_,
        }
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
            valid_: self.valid_,
        }
    }
}

pub struct GridPoint {
    x: i32,
    y: i32,
}

impl GridPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x: x, y: y }
    }
}

fn get_distance(p1: &Point, p2: &Point) -> f32 {
    ((p1.x - p2.x) * (p1.x - p2.x) + (p1.y - p2.y) * (p1.y - p2.y)).sqrt()
}

fn image_to_grid(p: &Point, cell_size: f32) -> GridPoint {
    GridPoint::new((p.x / cell_size) as i32, (p.y / cell_size) as i32)
}

pub struct Grid {
    w_: i32,
    h_: i32,
    cell_size_: f32,
    grid_: Vec<Vec<Point>>,
}

impl Grid {
    pub fn new(w: i32, h: i32, cell_size: f32) -> Self {
        let mut grid_ = vec![vec![Point { x: 0.0, y: 0.0, valid_: false }; w as usize]; h as usize];
        Self {
            w_: w,
            h_: h,
            cell_size_: cell_size,
            grid_: grid_,
        }
    }

    pub fn insert(&mut self, p: &Point) {
        let g = image_to_grid(p, self.cell_size_);
        self.grid_[g.x as usize][g.y as usize] = *p;
    }

    pub fn is_in_neighbourhood(&self, point: &Point, min_dist: f32, cell_size: f32) -> bool {
        let g = image_to_grid(point, cell_size);

        let d = 5;

        for i in (g.x - d)..=(g.x + d) {
            for j in (g.y - d)..=(g.y + d) {
                if i >= 0 && i < self.w_ && j >= 0 && j < self.h_ {
                    let p = self.grid_[i as usize][j as usize];
                    if p.valid_ && get_distance(&p, point) < min_dist {
                        return true;
                    }
                }
            }
        }

        false
    }
}

pub fn pop_random(points: &mut Vec<Point>, generator: &mut DefaultPRNG) -> Point {
    let idx = generator.random_int((points.len() as u32) - 1) as usize;
    points.remove(idx)
}

pub fn generate_random_point_around(
    p: &Point,
    min_dist: f32,
    generator: &mut DefaultPRNG,
) -> Point {
    let r1 = generator.random_float();
    let r2 = generator.random_float();

    let radius = min_dist * (r1 + 1.0);
    let angle = 2.0 * PI * r2;

    Point {
        x: p.x + radius * angle.cos(),
        y: p.y + radius * angle.sin(),
        valid_: true,
    }
}

pub fn generate_poisson_points(
    num_points: u32,
    generator: &mut DefaultPRNG,
    is_circle: bool,
    new_points_count: u32,
    min_dist: f32,
) -> Vec<Point> {
    let mut num_points = num_points * 2;

    if !is_circle {
        let pi_4 = 0.785398163397448309616;
        num_points = (pi_4 * num_points as f64) as u32;
    }

    let mut min_dist = min_dist;
    if min_dist < 0.0 {
        min_dist = (num_points as f32).sqrt() / num_points as f32;
    }

    let mut sample_points = Vec::new();
    let mut process_list = Vec::new();

    if num_points == 0 {
        return sample_points;
    }

    let cell_size = min_dist / 2.0_f32.sqrt();

    let grid_w = (1.0 / cell_size).ceil() as i32;
    let grid_h = (1.0 / cell_size).ceil() as i32;

    let mut grid = Grid::new(grid_w, grid_h, cell_size);

    let mut first_point;
    loop {
        first_point = Point::new(generator.random_float(), generator.random_float());
        if (is_circle && first_point.is_in_circle()) || (!is_circle && first_point.is_in_rectangle()) {
            break;
        }
    }

    process_list.push(first_point);
    sample_points.push(first_point);
    grid.insert(&first_point);

    while !process_list.is_empty() && sample_points.len() <= num_points as usize {
        let point = pop_random(&mut process_list, generator);

        for _ in 0..new_points_count {
            let new_point = generate_random_point_around(&point, min_dist, generator);
            if (is_circle && new_point.is_in_circle() || !is_circle && new_point.is_in_rectangle())
                && !grid.is_in_neighbourhood(&new_point, min_dist, cell_size)
            {
                process_list.push(new_point);
                sample_points.push(new_point);
                grid.insert(&new_point);
            }
        }
    }

    sample_points
}

pub fn sample_vogel_disk(idx: u32, num_points: u32, phi: f32) -> Point {
    const GOLDEN_ANGLE: f32 = 2.4;

    let r = ((idx as f32 + 0.5).sqrt()) / (num_points as f32).sqrt();
    let theta = idx as f32 * GOLDEN_ANGLE + phi;

    Point {
        x: r * theta.cos(),
        y: r * theta.sin(),
        valid_: true,
    }
}

pub fn generate_vogel_points(
    num_points: u32,
    is_circle: bool,
    phi: f32,
    center: Point,
) -> Vec<Point> {
    let mut sample_points = Vec::with_capacity(num_points as usize);

    let num_samples = if is_circle { 4 * num_points } else { num_points };

    for i in 0..num_points {
        let p = sample_vogel_disk(i, num_samples, phi * PI / 180.0) + center;
        sample_points.push(p);
    }

    sample_points
}

pub fn generate_jittered_grid_points(
    num_points: u32,
    generator: &mut DefaultPRNG,
    is_circle: bool,
    jitter_radius: f32,
    center: Point,
) -> Vec<Point> {
    let mut sample_points = Vec::with_capacity(num_points as usize);

    let grid_size = (num_points as f32).sqrt() as u32;

    for x in 0..grid_size {
        for y in 0..grid_size {
            let mut p;
            loop {
                let offs =
                    generate_random_point_around(&Point::new(0.0, 0.0), jitter_radius, generator)
                        - center
                        + Point::new(0.5, 0.5);
                p = Point::new(x as f32 / grid_size as f32, y as f32 / grid_size as f32) + offs;

                if p.is_in_rectangle() {
                    break;
                }
            }

            if is_circle && !p.is_in_circle() {
                continue;
            }

            sample_points.push(p);
        }
    }

    sample_points
}

fn radical_inverse_vdc(bits: u32) -> f32 {
    let mut bits = bits;
    bits = (bits << 16) | (bits >> 16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
    bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
    bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
    (bits as f32) * 2.3283064365386963e-10
}

fn hammersley2d(i: u32, n: u32) -> Point {
    Point::new(i as f32 / n as f32, radical_inverse_vdc(i))
}

pub fn generate_hammersley_points(num_points: u32) -> Vec<Point> {
    let mut sample_points = Vec::with_capacity(num_points as usize);

    for i in 0..num_points {
        let p = hammersley2d(i, num_points);
        sample_points.push(p);
    }
    sample_points
}

pub fn shuffle(points: &mut Vec<Point>, generator: &mut DefaultPRNG) {
    let length = points.len();
    if length == 0 {
        return;
    }
    for i in (1..length).rev() {
        points.swap(i, generator.random_int(i as u32) as usize);
    }
}

fn main() {
    // Placeholder for main function to satisfy the compiler.
    println!("Poisson Disk Sampling Module");
}