/**
 * Poisson Disk Points Generator
 *
 * \version 1.7.0
 * \date 21/01/2026
 * \author Sergey Kosarevsky, 2014-2026
 * \author support@linderdaum.com   http://www.linderdaum.com   http://blog.linderdaum.com
 *
 * https://github.com/corporateshark/poisson-disk-generator
 */

use std::f32;
use std::f32::consts::PI;
use std::vec::Vec;

const VERSION: &str = "1.7.0 (21/01/2026)";

pub struct DefaultPRNG {
    seed: u32,
}

impl DefaultPRNG {
    pub fn new() -> Self {
        Self { seed: 7133167 }
    }

    pub fn with_seed(seed: u32) -> Self {
        Self { seed }
    }

    fn random_float(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(521167);
        let a = (self.seed & 0x007fffff) | 0x40000000;
        let f = f32::from_bits(a) - 2.0;
        0.5 * f
    }

    fn random_int(&mut self, max_int: u32) -> u32 {
        (self.random_float() * max_int as f32) as u32
    }

    pub fn get_seed(&self) -> u32 {
        self.seed
    }
}

#[derive(Clone, Copy, Default)]
pub struct Point {
    x: f32,
    y: f32,
    valid: bool,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y, valid: true }
    }

    fn is_in_rectangle(&self) -> bool {
        self.x >= 0.0 && self.y >= 0.0 && self.x <= 1.0 && self.y <= 1.0
    }

    fn is_in_circle(&self) -> bool {
        let fx = self.x - 0.5;
        let fy = self.y - 0.5;
        (fx * fx + fy * fy) <= 0.25
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self.x += other.x;
        self.y += other.y;
        self
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(mut self, other: Self) -> Self {
        self.x -= other.x;
        self.y -= other.y;
        self
    }
}

pub struct GridPoint {
    x: i32,
    y: i32,
}

impl GridPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

fn get_distance(p1: &Point, p2: &Point) -> f32 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    (dx * dx + dy * dy).sqrt()
}

fn image_to_grid(p: &Point, cell_size: f32) -> GridPoint {
    GridPoint::new((p.x / cell_size) as i32, (p.y / cell_size) as i32)
}

struct Grid {
    w: i32,
    h: i32,
    cell_size: f32,
    grid: Vec<Vec<Option<Point>>>,
}

impl Grid {
    fn new(w: i32, h: i32, cell_size: f32) -> Self {
        let grid = vec![vec![None; w as usize]; h as usize];
        Self {
            w,
            h,
            cell_size,
            grid,
        }
    }

    fn insert(&mut self, p: &Point) {
        let g = image_to_grid(p, self.cell_size);
        self.grid[g.x as usize][g.y as usize] = Some(*p);
    }

    fn is_in_neighbourhood(&self, point: &Point, min_dist: f32, cell_size: f32) -> bool {
        let g = image_to_grid(point, cell_size);
        const D: i32 = 5;

        for i in (g.x - D)..=(g.x + D) {
            for j in (g.y - D)..=(g.y + D) {
                if i >= 0 && i < self.w && j >= 0 && j < self.h {
                    if let Some(p) = self.grid[i as usize][j as usize] {
                        if get_distance(&p, point) < min_dist {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

fn pop_random<T, PRNG: FnMut(u32) -> u32>(
    points: &mut Vec<T>,
    mut rng: PRNG,
) -> T {
    let idx = rng(points.len() as u32);
    points.remove(idx as usize)
}

fn generate_random_point_around<PRNG: FnMut() -> f32>(
    p: &Point,
    min_dist: f32,
    mut rng: PRNG,
) -> Point {
    let r1 = rng();
    let r2 = rng();
    let radius = min_dist * (r1 + 1.0);
    let angle = 2.0 * PI * r2;
    let x = p.x + radius * angle.cos();
    let y = p.y + radius * angle.sin();
    Point::new(x, y)
}

fn generate_poisson_points<PRNG: FnMut(u32) -> u32 + FnMut() -> f32>(
    num_points: u32,
    mut rng: PRNG,
    is_circle: bool,
    new_points_count: u32,
    mut min_dist: f32,
) -> Vec<Point> {
    let mut num_points = num_points * 2;

    if !is_circle {
        const PI_4: f32 = PI / 4.0;
        num_points = (PI_4 * num_points as f32) as u32;
    }

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

    let mut first_point;
    loop {
        first_point = Point::new(rng(), rng());
        if is_circle {
            if first_point.is_in_circle() {
                break;
            }
        } else if first_point.is_in_rectangle() {
            break;
        }
    }

    process_list.push(first_point);
    sample_points.push(first_point);
    grid.insert(&first_point);

    while !process_list.is_empty() && sample_points.len() <= num_points as usize {
        let point = pop_random(&mut process_list, &mut rng);

        for _ in 0..new_points_count {
            let new_point = generate_random_point_around(&point, min_dist, &mut rng);
            let can_fit_point = if is_circle {
                new_point.is_in_circle()
            } else {
                new_point.is_in_rectangle()
            };

            if can_fit_point && !grid.is_in_neighbourhood(&new_point, min_dist, cell_size) {
                process_list.push(new_point);
                sample_points.push(new_point);
                grid.insert(&new_point);
            }
        }
    }

    sample_points
}

fn sample_vogel_disk(idx: u32, num_points: u32, phi: f32) -> Point {
    const K_GOLDEN_ANGLE: f32 = 2.4f32;
    let r = ((idx as f32 + 0.5) as f32).sqrt() / ((num_points as f32).sqrt());
    let theta = idx as f32 * K_GOLDEN_ANGLE + phi;
    Point::new(r * theta.cos(), r * theta.sin())
}

fn generate_vogel_points(
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

fn generate_hammersley_points(num_points: u32) -> Vec<Point> {
    let mut sample_points = Vec::with_capacity(num_points as usize);

    for i in 0..num_points {
        let p = hammersley2d(i, num_points);
        sample_points.push(p)
    }
    sample_points
}

fn radical_inverse_vdc(mut bits: u32) -> f32 {
    bits = (bits << 16) | (bits >> 16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xaaaaaaaa) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xcccccccc) >> 2);
    bits = ((bits & 0x0f0f0f0f) << 4) | ((bits & 0xf0f0f0f0) >> 4);
    bits = ((bits & 0x00ff00ff) << 8) | ((bits & 0xff00ff00) >> 8);
    f32::from_bits(bits) * 2.3283064365386963e-10
}

fn hammersley2d(i: u32, n: u32) -> Point {
    Point::new(i as f32 / n as f32, radical_inverse_vdc(i))
}

fn generate_jittered_grid_points<PRNG: FnMut(u32) -> u32 + FnMut() -> f32>(
    num_points: u32,
    mut rng: PRNG,
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
                let offs = generate_random_point_around(
                    &Point::new(0.0, 0.0),
                    jitter_radius,
                    &mut rng,
                ) - center + Point::new(0.5, 0.5);
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

fn shuffle<PRNG: FnMut(u32) -> u32>(
    points: &mut Vec<Point>,
    mut rng: PRNG,
) {
    let length = points.len();
    if length == 0 {
        return;
    }
    for i in (1..length).rev() {
        let j = rng(i as u32 + 1) as usize;
        points.swap(i, j);
    }
}

fn main() {
    // Main function for testing purposes
}