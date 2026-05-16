use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;

use std::time::{SystemTime, UNIX_EPOCH};

const SAMPLES: i32 = 200;
const TAU: f64 = 2.0 * PI;

// Custom function to generate a random double using the standard library
fn random_double() -> f64 {
    let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let nanos = since_the_epoch.as_nanos() as u64;
    (nanos as f64) / (u64::MAX as f64)
}

#[derive(Copy, Clone)]
struct Vec {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vec { x, y, z }
    }

    fn mult(&self, other: Vec) -> Vec {
        Vec::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    fn norm(self) -> Vec {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self * (1.0 / len)
    }

    fn dot(&self, other: Vec) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

use std::ops::{Add, Mul, Sub, Rem};

impl Add<Vec> for Vec {
    type Output = Vec;

    fn add(self, other: Vec) -> Vec {
        Vec::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub<Vec> for Vec {
    type Output = Vec;

    fn sub(self, other: Vec) -> Vec {
        Vec::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f64> for Vec {
    type Output = Vec;

    fn mul(self, b: f64) -> Vec {
        Vec::new(self.x * b, self.y * b, self.z * b)
    }
}

impl Rem<Vec> for Vec {
    type Output = Vec;

    fn rem(self, other: Vec) -> Vec {
        Vec::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

#[derive(Copy, Clone)]
struct Ray {
    o: Vec,
    d: Vec,
}

#[derive(Copy, Clone)]
enum ReflT {
    DIFF,
    SPEC,
    REFR,
}

#[derive(Copy, Clone)]
struct Sphere {
    rad: f64,
    p: Vec,
    e: Vec,
    c: Vec,
    refl: ReflT,
}

impl Sphere {
    fn new(rad: f64, p: Vec, e: Vec, c: Vec, refl: ReflT) -> Self {
        Sphere { rad, p, e, c, refl }
    }

    fn intersect(&self, r: Ray) -> f64 {
        let op = self.p - r.o;
        let eps = 1e-4;
        let b = op.dot(r.d);
        let det = b * b - op.dot(op) + self.rad * self.rad;

        if det < 0.0 {
            0.0
        } else {
            let det_sqrt = det.sqrt();
            let t = b - det_sqrt;
            if t > eps {
                t
            } else {
                let t = b + det_sqrt;
                if t > eps { t } else { 0.0 }
            }
        }
    }
}

fn clamp(x: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

fn gamma(x: f64) -> i32 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as i32
}

fn intersect(r: Ray, spheres: &[Sphere], t: &mut f64, id: &mut usize) -> bool {
    let mut d;
    let inf = 1e20;
    *t = inf;

    for (i, sphere) in spheres.iter().enumerate() {
        d = sphere.intersect(r);
        if d < *t && d != 0.0 {
            *t = d;
            *id = i;
        }
    }

    *t < inf
}

fn radiance(r: Ray, spheres: &[Sphere], depth: i32) -> Vec {
    let mut t: f64 = 0.0;
    let mut id: usize = 0;

    if !intersect(r, spheres, &mut t, &mut id) {
        return Vec::new(0.0, 0.0, 0.0);
    }

    let obj = spheres[id];
    let x = r.o + r.d * t;
    let n = (x - obj.p).norm();
    let nl = if n.dot(r.d) < 0.0 { n } else { n * -1.0 };
    let f = obj.c;
    let p = f.x.max(f.y).max(f.z);

    if depth > 5 {
        if random_double() < p {
        } else {
            return obj.e;
        }
    }
    
    let reflection_ray = Ray {
        o: x,
        d: r.d - n * 2.0 * n.dot(r.d),
    };

    match obj.refl {
        ReflT::DIFF => {
            let r1 = TAU * random_double();
            let r2 = random_double();
            let r2s = r2.sqrt();

            let w = nl;
            let u = if w.x.abs() > 0.1 {
                Vec::new(0.0, 1.0, 0.0)
            } else {
                Vec::new(1.0, 0.0, 0.0)
            } % w;
            let u = u.norm();
            let v = w % u;
            let d = (u * r1.cos() * r2s + v * r1.sin() * r2s + w * (1.0 - r2).sqrt()).norm();
            obj.e + f.mult(radiance(Ray { o: x, d }, spheres, depth + 1))
        }
        ReflT::SPEC => obj.e + f.mult(radiance(reflection_ray, spheres, depth + 1)),
        ReflT::REFR => {
            let into = n.dot(nl) > 0.0;
            let nnt = if into { 1.0 / 1.5 } else { 1.5 };
            let ddn = r.d.dot(nl);
            let cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);

            if cos2t < 0.0 {
                return obj.e + f.mult(radiance(reflection_ray, spheres, depth + 1));
            }

            let tdir = (r.d * nnt - n * (if into { 1.0 } else { -1.0 } * (ddn * nnt + cos2t.sqrt()))).norm();
            let a = 1.5 - 1.0;
            let b = 1.5 + 1.0;
            let r0 = (a * a) / (b * b);
            let c = 1.0 - if into { -ddn } else { tdir.dot(n) };
            let re = r0 + (1.0 - r0) * c.powi(5);
            let tr = 1.0 - re;
            let p = (0.25 + 0.5 * re) as f64;
            let rp = re / p;
            let tp = tr / (1.0 - p);

            if depth > 2 {
                if random_double() < p {
                    return obj.e + f.mult(radiance(reflection_ray, spheres, depth + 1) * rp);
                } else {
                    return obj.e + f.mult(radiance(Ray { o: x, d: tdir }, spheres, depth + 1) * tp);
                }
            } else {
                return obj.e
                    + f.mult(
                        radiance(reflection_ray, spheres, depth + 1) * re
                            + radiance(Ray { o: x, d: tdir }, spheres, depth + 1) * tr,
                    );
            }
        }
    }
}

fn run() {
    let mut buffer = vec![Vec::new(0.0, 0.0, 0.0); (WIDTH * HEIGHT) as usize];
    let spheres = [
        Sphere::new(1e5, Vec::new(1e5 + 1.0, 40.8, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.25, 0.25), ReflT::DIFF),
        Sphere::new(1e5, Vec::new(-1e5 + 99.0, 40.8, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.25, 0.6, 0.15), ReflT::DIFF),
        Sphere::new(1e5, Vec::new(50.0, 40.8, 1e5), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::DIFF),
        Sphere::new(1e5, Vec::new(50.0, 40.8, -1e5 + 170.0), Vec::new(0.0, 0.0, 0.0), Vec::new(0.0, 0.0, 0.0), ReflT::DIFF),
        Sphere::new(1e5, Vec::new(50.0, 1e5, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::DIFF),
        Sphere::new(1e5, Vec::new(50.0, -1e5 + 81.6, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::DIFF),
        Sphere::new(16.5, Vec::new(27.0, 16.5, 47.0), Vec::new(0.0, 0.0, 0.0), Vec::new(0.3, 0.3, 1.0) * 0.9, ReflT::SPEC),
        Sphere::new(16.5, Vec::new(73.0, 16.5, 78.0), Vec::new(0.0, 0.0, 0.0), Vec::new(1.0, 1.0, 1.0) * 0.9, ReflT::REFR),
        Sphere::new(600.0, Vec::new(50.0, 681.6 - 0.27, 81.6), Vec::new(12.0, 12.0, 12.0), Vec::new(0.0, 0.0, 0.0), ReflT::DIFF),
    ];

    let cam = Ray {
        o: Vec::new(50.0, 50.0, 295.6),
        d: Vec::new(0.0, -0.04, -1.0).norm(),
    };
    
    let cx = Vec::new(WIDTH as f64 * 0.5135 / HEIGHT as f64, 0.0, 0.0);
    let cy = (cx % cam.d).norm() * 0.5135;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut c = Vec::new(0.0, 0.0, 0.0);
            for sy in 0..2 {
                for sx in 0..2 {
                    let mut r = Vec::new(0.0, 0.0, 0.0);
                    for _ in 0..SAMPLES {
                        let r1 = 2.0 * random_double();
                        let dx = if r1 < 1.0 { r1.sqrt() - 1.0 } else { 1.0 - (2.0 - r1).sqrt() };
                        let r2 = 2.0 * random_double();
                        let dy = if r2 < 1.0 { r2.sqrt() - 1.0 } else { 1.0 - (2.0 - r2).sqrt() };
                        let d = cx * (((sx as f64 + 0.5 + dx) / 2.0 + x as f64) / WIDTH as f64 - 0.5)
                            + cy * (((sy as f64 + 0.5 + dy) / 2.0 + y as f64) / HEIGHT as f64 - 0.5)
                            + cam.d;
                        r = r + radiance(Ray { o: cam.o + d * 140.0, d: d.norm() }, &spheres, 0) * (1.0 / SAMPLES as f64);
                    }
                    c = c + Vec::new(clamp(r.x), clamp(r.y), clamp(r.z)) * 0.25;
                }
            }
            buffer[(WIDTH * y + x) as usize] = c;
        }
    }

    save_image(&buffer);
}

fn save_image(buffer: &[Vec]) {
    let mut file = File::create("image.ppm").expect("Unable to create file");
    let _ = file.write_all(format!("P3\n{} {}\n255\n", WIDTH, HEIGHT).as_bytes());

    for vec in buffer {
        let pixel = format!(
            "{} {} {}\n",
            gamma(vec.x),
            gamma(vec.y),
            gamma(vec.z)
        );
        let _ = file.write_all(pixel.as_bytes());
    }
}

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

fn main() {
    run();
}