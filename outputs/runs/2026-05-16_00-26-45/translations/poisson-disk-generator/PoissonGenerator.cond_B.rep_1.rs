pub mod poisson_generator {
    use std::ops::{Add, Sub};

    pub const VERSION: &str = "1.7.0 (21/01/2026)";

    #[derive(Clone, Copy, Default)]
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
            let float_a = f32::from_bits(a) - 2.0;
            0.5 * float_a
        }

        pub fn random_int(&mut self, max_int: u32) -> u32 {
            (self.random_float() * max_int as f32) as u32
        }

        pub fn seed(&self) -> u32 {
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
        pub fn new(x: f32, y: f32) -> Self {
            Point { x, y, valid: true }
        }

        pub fn is_in_rectangle(&self) -> bool {
            self.x >= 0.0 && self.y >= 0.0 && self.x <= 1.0 && self.y <= 1.0
        }

        pub fn is_in_circle(&self) -> bool {
            let fx = self.x - 0.5;
            let fy = self.y - 0.5;
            (fx * fx + fy * fy) <= 0.25
        }

        pub fn add(&mut self, other: &Point) {
            self.x += other.x;
            self.y += other.y;
        }

        pub fn sub(&mut self, other: &Point) {
            self.x -= other.x;
            self.y -= other.y;
        }
    }

    impl Add for Point {
        type Output = Point;

        fn add(self, other: Point) -> Point {
            Point {
                x: self.x + other.x,
                y: self.y + other.y,
                valid: self.valid || other.valid,
            }
        }
    }

    impl Sub for Point {
        type Output = Point;

        fn sub(self, other: Point) -> Point {
            Point {
                x: self.x - other.x,
                y: self.y - other.y,
                valid: self.valid && other.valid,
            }
        }
    }

    pub struct GridPoint {
        x: i32,
        y: i32,
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
            Grid {
                w,
                h,
                cell_size,
                grid,
            }
        }

        pub fn insert(&mut self, p: Point) {
            let g = image_to_grid(&p, self.cell_size);
            self.grid[g.x as usize][g.y as usize] = p;
        }

        pub fn is_in_neighbourhood(&self, point: &Point, min_dist: f32) -> bool {
            let g = image_to_grid(point, self.cell_size);
            const D: i32 = 5;
            for i in (g.x - D)..=(g.x + D) {
                for j in (g.y - D)..=(g.y + D) {
                    if i >= 0 && i < self.w && j >= 0 && j < self.h {
                        let p = self.grid[i as usize][j as usize];
                        if p.valid && get_distance(&p, point) < min_dist {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }

    pub fn pop_random(points: &mut Vec<Point>, generator: &mut DefaultPRNG) -> Point {
        let idx = generator.random_int(points.len() as u32 - 1);
        points.swap_remove(idx as usize)
    }

    pub fn generate_random_point_around(
        p: &Point,
        min_dist: f32,
        generator: &mut DefaultPRNG,
    ) -> Point {
        let r1 = generator.random_float();
        let r2 = generator.random_float();
        let radius = min_dist * (r1 + 1.0);
        let angle = 2.0 * std::f32::consts::PI * r2;
        let x = p.x + radius * angle.cos();
        let y = p.y + radius * angle.sin();
        Point::new(x, y)
    }

    pub fn generate_poisson_points(
        num_points: u32,
        generator: &mut DefaultPRNG,
        is_circle: bool,
        new_points_count: u32,
        mut min_dist: f32,
    ) -> Vec<Point> {
        let mut num_points = 2 * num_points;
        if !is_circle {
            const PI_OVER_4: f64 = std::f64::consts::PI / 4.0;
            num_points = (PI_OVER_4 * num_points as f64) as u32;
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
            first_point = Point::new(generator.random_float(), generator.random_float());
            if is_circle && first_point.is_in_circle() || !is_circle && first_point.is_in_rectangle() {
                break;
            }
        }

        process_list.push(first_point);
        sample_points.push(first_point);
        grid.insert(first_point);

        while !process_list.is_empty() && sample_points.len() <= num_points as usize {
            let point = pop_random(&mut process_list, generator);
            for _ in 0..new_points_count {
                let new_point = generate_random_point_around(&point, min_dist, generator);
                let can_fit_point = if is_circle {
                    new_point.is_in_circle()
                } else {
                    new_point.is_in_rectangle()
                };
                if can_fit_point && !grid.is_in_neighbourhood(&new_point, min_dist) {
                    process_list.push(new_point);
                    sample_points.push(new_point);
                    grid.insert(new_point);
                    continue;
                }
            }
        }
        sample_points
    }

    pub fn sample_vogel_disk(idx: u32, num_points: u32, phi: f32) -> Point {
        const K_GOLDEN_ANGLE: f32 = 2.4;
        let r = ((idx as f32) + 0.5).sqrt() / (num_points as f32).sqrt();
        let theta = idx as f32 * K_GOLDEN_ANGLE + phi;
        Point::new(r * theta.cos(), r * theta.sin())
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
            let p = sample_vogel_disk(i, num_samples, phi.to_radians()) + center;
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
                    let offs = generate_random_point_around(&Point::new(0.0, 0.0), jitter_radius, generator) - center + Point::new(0.5, 0.5);
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

    fn radical_inverse_vdc(mut bits: u32) -> f32 {
        bits = (bits << 16) | (bits >> 16);
        bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
        bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
        bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
        bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
        bits as f32 * 2.3283064365386963e-10
    }

    pub fn hammersley_2d(i: u32, n: u32) -> Point {
        Point::new(i as f32 / n as f32, radical_inverse_vdc(i))
    }

    pub fn generate_hammersley_points(num_points: u32) -> Vec<Point> {
        let mut sample_points = Vec::with_capacity(num_points as usize);
        for i in 0..num_points {
            let p = hammersley_2d(i, num_points);
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
            let j = generator.random_int(i as u32) as usize;
            points.swap(i, j);
        }
    }
}

fn main() {
    println!("This is a library for Poisson disk sampling.");
}