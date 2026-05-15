use std::f64::consts::PI;
use std::f64::INFINITY;
use std::sync::OnceLock;

const TAU: f64 = PI * 2.0;
const SAMPLES: usize = 200;

#[derive(Clone, Copy, Debug)]
struct Vec {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vec { x, y, z }
    }

    fn mult(self, b: Vec) -> Vec {
        Vec::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }

    fn norm(&mut self) -> &Vec {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self.x /= length;
        self.y /= length;
        self.z /= length;
        self
    }

    fn dot(self, b: Vec) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
}

impl std::ops::Add for Vec {
    type Output = Self;
    fn add(self, b: Vec) -> Self {
        Vec::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}

impl std::ops::Sub for Vec {
    type Output = Self;
    fn sub(self, b: Vec) -> Self {
        Vec::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}

impl std::ops::Mul<f64> for Vec {
    type Output = Self;
    fn mul(self, b: f64) -> Self {
        Vec::new(self.x * b, self.y * b, self.z * b)
    }
}

impl std::ops::Rem for Vec {
    type Output = Self;
    fn rem(self, b: Vec) -> Self {
        Vec::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
}

struct Ray {
    o: Vec,
    d: Vec,
}

#[derive(Clone, Copy, PartialEq)]
enum ReflT {
    Diff,
    Spec,
    Refr,
}

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
    
    fn intersect(&self, r: &Ray) -> f64 {
        let op = self.p - r.o;
        let eps = 1e-4;
        let b = op.dot(r.d);
        let det = b * b - op.dot(op) + self.rad * self.rad;

        if det < 0.0 {
            INFINITY
        } else {
            let det_sqrt = det.sqrt();
            let t = b - det_sqrt;
            if t > eps {
                t
            } else {
                let t2 = b + det_sqrt;
                if t2 > eps {
                    t2
                } else {
                    INFINITY
                }
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

fn gamma(x: f64) -> u32 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u32
}

fn intersect(r: &Ray, t: &mut f64, id: &mut usize) -> bool {
    let inf = *t = INFINITY;
    if let Some(spheres) = SPHERES.get() {
        let n = spheres.len();
        for i in 0..n {
            let d = spheres[i].intersect(r);
            if d > 0.0 && d < *t {
                *t = d;
                *id = i;
            }
        }
    }
    *t < inf
}

fn radiance(r: &Ray, depth: usize) -> Vec {
    let mut t: f64 = 0.0;
    let mut id = 0;
    if !intersect(r, &mut t, &mut id) {
        return Vec::new(0.0, 0.0, 0.0);
    }

    let spheres = SPHERES.get().unwrap();
    
    let obj = &spheres[id];
    let x = r.o + r.d * t;
    let mut n = x - obj.p;
    n.norm();
    let nl = if n.dot(r.d) < 0.0 { n } else { n * -1.0 };
    let mut f = obj.c;
    let p = f.x.max(f.y).max(f.z);

    if depth > 5 {
        if random_f64() < p {
            f = f * (1.0 / p);
        } else {
            return obj.e;
        }
    }

    if obj.refl == ReflT::Diff {
        let r1 = TAU * random_f64();
        let r2 = random_f64();
        let r2s = r2.sqrt();
        
        let w = nl;
        let mut u = if w.x.abs() > 0.1 { Vec::new(0.0, 1.0, 0.0) } else { Vec::new(1.0, 0.0, 0.0) };
        u = (u % w).norm().clone();
        let v = w % u;
        let d = (u * r1.cos() * r2s + v * r1.sin() * r2s + w * (1.0 - r2).sqrt()).norm().clone();
        return obj.e + f.mult(radiance(&Ray { o: x, d }, depth + 1));
    } else if obj.refl == ReflT::Spec {
        return obj.e + f.mult(radiance(&Ray { o: x, d: r.d - n * 2.0 * n.dot(r.d) }, depth + 1));
    }

    let refl_ray = Ray { o: x, d: r.d - n * 2.0 * n.dot(r.d) };
    let into = n.dot(nl) > 0.0;
    let nc = 1.0;
    let nt = 1.5;
    let nnt = if into { nc / nt } else { nt / nc };
    let ddn = r.d.dot(nl);
    let mut cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);

    if cos2t < 0.0 {
        return obj.e + f.mult(radiance(&refl_ray, depth + 1));
    }

    let tdir = (r.d * nnt - n * ((if into { 1.0 } else { -1.0 }) * (ddn * nnt + cos2t.sqrt()))).norm().clone();
    let a = nt - nc;
    let b = nt + nc;
    let r0 = a * a / (b * b);
    let c = 1.0 - (if into { -ddn } else { tdir.dot(n) });
    let re = r0 + (1.0 - r0) * c * c * c * c * c;
    let tr = 1.0 - re;
    let p = 0.25 + 0.5 * re;
    let rp = re / p;
    let tp = tr / (1.0 - p);

    let rad = if depth > 2 {
        if random_f64() < p {
            radiance(&refl_ray, depth + 1) * rp
        } else {
            radiance(&Ray { o: x, d: tdir }, depth + 1) * tp
        }
    } else {
        radiance(&refl_ray, depth + 1) * re + radiance(&Ray { o: x, d: tdir }, depth + 1) * tr
    };

    obj.e + f.mult(rad)
}

fn random_f64() -> f64 {
    // Placeholder for random number generation
    0.5
}

static SPHERES: OnceLock<[Sphere; 9]> = OnceLock::new();

fn main() {
    let _ = SPHERES.get_or_init(|| [
        Sphere::new(1e5, Vec::new(1e5 + 1.0, 40.8, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.25, 0.25), ReflT::Diff),
        Sphere::new(1e5, Vec::new(-1e5 + 99.0, 40.8, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.25, 0.6, 0.15), ReflT::Diff),
        Sphere::new(1e5, Vec::new(50.0, 40.8, 1e5), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::Diff),
        Sphere::new(1e5, Vec::new(50.0, 40.8, -1e5 + 170.0), Vec::new(0.0, 0.0, 0.0), Vec::new(0.0, 0.0, 0.0), ReflT::Diff),
        Sphere::new(1e5, Vec::new(50.0, 1e5, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::Diff),
        Sphere::new(1e5, Vec::new(50.0, -1e5 + 81.6, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::Diff),
        Sphere::new(16.5, Vec::new(27.0, 16.5, 47.0), Vec::new(0.0, 0.0, 0.0), Vec::new(0.3, 0.3, 1.0) * 0.9, ReflT::Spec),
        Sphere::new(16.5, Vec::new(73.0, 16.5, 78.0), Vec::new(0.0, 0.0, 0.0), Vec::new(1.0, 1.0, 1.0) * 0.9, ReflT::Refr),
        Sphere::new(600.0, Vec::new(50.0, 681.6 - 0.27, 81.6), Vec::new(12.0, 12.0, 12.0), Vec::new(0.0, 0.0, 0.0), ReflT::Diff),
    ]);

    run();
}

fn run() {
    // Assume some initialization for the rendering context
    // Frame buffer setup, camera setup, etc.

    // Your rendering logic here...
}