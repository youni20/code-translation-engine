use std::f32;
use std::f32::consts::PI;

const VERSION: &str = "1.7.0 (21/01/2026)";

pub struct DefaultPRNG {
    seed: u32,
}

impl DefaultPRNG {
    pub fn new() -> Self {
        DefaultPRNG { seed: 7133167 }
    }

    pub fn with_seed(seed: u32) -> Self {
        DefaultPRNG { seed }
    }

    pub fn random_float(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(521167);
        let a = (self.seed & 0x007fffff) | 0x40000000;
        // remap to 0..1
        0.5 * (f32::from_bits(a) - 2.0)
    }

    pub fn random_int(&mut self, max_int: u32) -> u32 {
        (self.random_float() * max_int as f32) as u32
    }

    pub fn get_seed(&self) -> u32 {
        self.seed
    }
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    valid: bool,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y, valid: true }
    }

    pub fn default() -> Self {
        Point { x: 0.0, y: 0.0, valid: false }
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
        Point { x: self.x + other.x, y: self.y + other.y, ..self }
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point { x: self.x - other.x, y: self.y - other.y, ..self }
    }
}

pub struct GridPoint {
    pub x: i32,
    pub y: i32,
}

impl GridPoint {
    pub fn new(x: i32, y: i32) -> Self {
        GridPoint { x, y }
    }
}

pub fn get_distance(p1: &Point, p2: &Point) -> f32 {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
}

pub fn image_to_grid(p: &Point, cell_size: f32) -> GridPoint {
    GridPoint {
        x: (p.x / cell_size) as i32,
        y: (p.y / cell_size) as i32,
    }
}

pub struct Grid {
    w: i32,
    h: i32,
    cell_size: f32,
    grid: Vec<Vec<Point>>,
}

impl Grid {
    pub fn new(w: i32, h: i32, cell_size: f32) -> Self {
        let grid = vec![vec![Point::default(); w as usize]; h as usize];
        Grid { w, h, cell_size, grid }
    }

    pub fn insert(&mut self, p: Point) {
        let g = image_to_grid(&p, self.cell_size);
        self.grid[g.x as usize][g.y as usize] = p;
    }

    pub fn is_in_neighbourhood(&self, point: &Point, min_dist: f32, cell_size: f32) -> bool {
        let g = image_to_grid(point, cell_size);
        let d = 5;

        for i in (g.x - d)..=(g.x + d) {
            for j in (g.y - d)..=(g.y + d) {
                if i >= 0 && i < self.w && j >= 0 && j < self.h {
                    let p = &self.grid[i as usize][j as usize];
                    if p.valid && get_distance(p, point) < min_dist {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub fn pop_random(points: &mut Vec<Point>, generator: &mut DefaultPRNG) -> Point {
    let idx = generator.random_int(points.len() as u32 - 1) as usize;
    points.swap_remove(idx)
}

pub fn generate_random_point_around(p: &Point, min_dist: f32, generator: &mut DefaultPRNG) -> Point {
    let r1 = generator.random_float();
    let r2 = generator.random_float();

    let radius = min_dist * (r1 + 1.0);
    let angle = 2.0 * PI * r2;

    let x = p.x + radius * angle.cos();
    let y = p.y + radius * angle.sin();

    Point { x, y, ..*p }
}

pub fn generate_poisson_points(
    num_points: u32,
    mut generator: DefaultPRNG,
    is_circle: bool,
    new_points_count: u32,
    min_dist: f32,
) -> Vec<Point> {
    let mut num_points = num_points * 2;
    if !is_circle {
        num_points = (0.785398163 * num_points as f64) as u32;
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

    let cell_size = min_dist / (2.0f32).sqrt();
    let grid_w = (1.0 / cell_size).ceil() as i32;
    let grid_h = (1.0 / cell_size).ceil() as i32;

    let mut grid = Grid::new(grid_w, grid_h, cell_size);

    let mut first_point: Point;
    loop {
        first_point = Point {
            x: generator.random_float(),
            y: generator.random_float(),
            valid: true,
        };
        if is_circle && first_point.is_in_circle() || !is_circle && first_point.is_in_rectangle() {
            break;
        }
    }

    process_list.push(first_point);
    sample_points.push(first_point);
    grid.insert(first_point);

    while !process_list.is_empty() && sample_points.len() <= num_points as usize {
        let point = pop_random(&mut process_list, &mut generator);

        for _ in 0..new_points_count {
            let new_point = generate_random_point_around(&point, min_dist, &mut generator);
            let can_fit_point = if is_circle {
                new_point.is_in_circle()
            } else {
                new_point.is_in_rectangle()
            };

            if can_fit_point && !grid.is_in_neighbourhood(&new_point, min_dist, cell_size) {
                process_list.push(new_point);
                sample_points.push(new_point);
                grid.insert(new_point);
            }
        }
    }
    sample_points
}

fn sample_vogel_disk(idx: u32, num_points: u32, phi: f32) -> Point {
    let k_golden_angle = 2.4f32;
    let r = ((idx as f32) + 0.5).sqrt() / (num_points as f32).sqrt();
    let theta = idx as f32 * k_golden_angle + phi;

    Point {
        x: r * theta.cos(),
        y: r * theta.sin(),
        valid: true,
    }
}

pub fn generate_vogel_points(num_points: u32, is_circle: bool, phi: f32, center: Point) -> Vec<Point> {
    let mut sample_points = Vec::new();
    let num_samples = if is_circle { 4 * num_points } else { num_points };

    for i in 0..num_points {
        let p = sample_vogel_disk(i, num_samples, phi * PI / 180.0) + center;
        sample_points.push(p);
    }
    sample_points
}

pub fn generate_hammersley_points(num_points: u32) -> Vec<Point> {
    let mut sample_points = Vec::new();
    sample_points.reserve(num_points as usize);

    for i in 0..num_points {
        let p = hammersley2d(i, num_points);
        sample_points.push(p);
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
    Point {
        x: i as f32 / n as f32,
        y: radical_inverse_vdc(i),
        valid: true,
    }
}

pub fn generate_jittered_grid_points(
    num_points: u32,
    mut generator: DefaultPRNG,
    is_circle: bool,
    jitter_radius: f32,
    center: Point,
) -> Vec<Point> {
    let mut sample_points = Vec::new();
    let grid_size = (num_points as f32).sqrt() as u32;

    for x in 0..grid_size {
        for y in 0..grid_size {
            let mut p;
            loop {
                let offs = generate_random_point_around(&Point::default(), jitter_radius, &mut generator) - center + Point { x: 0.5, y: 0.5, valid: true };
                p = Point {
                    x: x as f32 / grid_size as f32,
                    y: y as f32 / grid_size as f32,
                    valid: true,
                } + offs;
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

pub fn shuffle(points: &mut Vec<Point>, mut generator: DefaultPRNG) {
    let length = points.len() as i32;
    if length == 0 {
        return;
    }
    for i in (0..length - 1).rev() {
        let j = generator.random_int(i as u32) as usize;
        points.swap(i as usize, j);
    }
}

// Adding a main function to satisfy the compiler
fn main() {
    println!("This is a Rust module without execution logic in main.");
}