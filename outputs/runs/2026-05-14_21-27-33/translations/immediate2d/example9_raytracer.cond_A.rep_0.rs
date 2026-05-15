// The original code references external functions which are undefined in the code provided.
// To allow successful compilation, dummy implementations for these functions must be added.

use std::f64;

// Dummy implementations for external API functions to enable compilation
#[no_mangle]
pub extern "C" fn UseDoubleBuffering(_use: bool) {
    // No-op for compilation
}

#[no_mangle]
pub extern "C" fn Width() -> f64 {
    800.0 // Example width
}

#[no_mangle]
pub extern "C" fn Height() -> f64 {
    600.0 // Example height
}

#[no_mangle]
pub extern "C" fn RandomDouble() -> f64 {
    0.5 // Fixed return for deterministic result
}

#[no_mangle]
pub extern "C" fn DrawPixel(_x: i32, _y: i32, _color: u32) {
    // No-op for compilation
}

#[no_mangle]
pub extern "C" fn MakeColor(_r: i32, _g: i32, _b: i32) -> u32 {
    0 // Example color
}

#[no_mangle]
pub extern "C" fn Present() {
    // No-op for compilation
}

#[no_mangle]
pub extern "C" fn SaveImage() {
    // No-op for compilation
}

const SAMPLES: usize = 200;

#[derive(Copy, Clone, Default)]
struct Vec {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vec { x, y, z }
    }

    fn norm(mut self) -> Vec {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self.x /= mag;
        self.y /= mag;
        self.z /= mag;
        self
    }

    fn dot(self, b: Vec) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }

    fn mult(self, b: Vec) -> Vec {
        Vec::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }
}

use std::ops::{Add, Mul, Sub, Rem};

impl Add for Vec {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Vec::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Vec::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f64> for Vec {
    type Output = Self;
    fn mul(self, b: f64) -> Self {
        Vec::new(self.x * b, self.y * b, self.z * b)
    }
}

impl Rem for Vec {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        Vec::new(self.y * other.z - self.z * other.y,
                 self.z * other.x - self.x * other.z,
                 self.x * other.y - self.y * other.x)
    }
}

struct Ray {
    o: Vec,
    d: Vec,
}

#[derive(Copy, Clone, PartialEq)]
enum ReflT {
    DIFF,
    SPEC,
    REFR,
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
            return 0.0;
        }

        let det_sqrt = det.sqrt();
        let t = b - det_sqrt;

        if t > eps {
            t
        } else {
            let t = b + det_sqrt;
            if t > eps {
                t
            } else {
                0.0
            }
        }
    }
}

fn sphere_const(rad: f64, p: (f64, f64, f64), e: (f64, f64, f64), c: (f64, f64, f64), refl: ReflT) -> Sphere {
    Sphere::new(rad, Vec::new(p.0, p.1, p.2), Vec::new(e.0, e.1, e.2), Vec::new(c.0, c.1, c.2), refl)
}

fn get_spheres() -> [Sphere; 9] {
    [
        sphere_const(1e5, (1e5 + 1.0, 40.8, 81.6), (0.0, 0.0, 0.0), (0.75, 0.25, 0.25), ReflT::DIFF),
        sphere_const(1e5, (-1e5 + 99.0, 40.8, 81.6), (0.0, 0.0, 0.0), (0.25, 0.6, 0.15), ReflT::DIFF),
        sphere_const(1e5, (50.0, 40.8, 1e5), (0.0, 0.0, 0.0), (0.75, 0.75, 0.75), ReflT::DIFF),
        sphere_const(1e5, (50.0, 40.8, -1e5 + 170.0), (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), ReflT::DIFF),
        sphere_const(1e5, (50.0, 1e5, 81.6), (0.0, 0.0, 0.0), (0.75, 0.75, 0.75), ReflT::DIFF),
        sphere_const(1e5, (50.0, -1e5 + 81.6, 81.6), (0.0, 0.0, 0.0), (0.75, 0.75, 0.75), ReflT::DIFF),
        sphere_const(16.5, (27.0, 16.5, 47.0), (0.0, 0.0, 0.0), (0.3 * 0.9, 0.3 * 0.9, 1.0 * 0.9), ReflT::SPEC),
        sphere_const(16.5, (73.0, 16.5, 78.0), (0.0, 0.0, 0.0), (1.0 * 0.9, 1.0 * 0.9, 1.0 * 0.9), ReflT::REFR),
        sphere_const(600.0, (50.0, 681.6 - 0.27, 81.6), (12.0, 12.0, 12.0), (0.0, 0.0, 0.0), ReflT::DIFF),
    ]
}

fn clamp(x: f64) -> f64 {
    if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
}

fn gamma(x: f64) -> i32 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5).floor() as i32
}

fn intersect(r: &Ray, t: &mut f64, id: &mut usize) -> bool {
    let spheres = get_spheres();
    let n = spheres.len();
    let mut d;
    let inf = 1e20;
    *t = inf;

    for i in 0..n {
        d = spheres[i].intersect(r);
        if d > 0.0 && d < *t {
            *t = d;
            *id = i;
        }
    }
    *t < inf
}

fn radiance(r: &Ray, depth: usize) -> Vec {
    let spheres = get_spheres();
    let mut t = 0.0;
    let mut id = 0;

    if !intersect(r, &mut t, &mut id) {
        return Vec::default();
    }

    let obj = &spheres[id];
    let x = r.o + r.d * t;
    let n = (x - obj.p).norm();
    let nl = if n.dot(r.d) < 0.0 { n } else { n * -1.0 };
    let mut f = obj.c;
    let max_refl = f.x.max(f.y).max(f.z);

    let depth = depth + 1;
    if depth > 5 {
        if unsafe { RandomDouble() } < max_refl {
            f = f * (1.0 / max_refl);
        } else {
            return obj.e;
        }
    }

    if obj.refl == ReflT::DIFF {
        let r1 = 2.0 * f64::consts::PI * unsafe { RandomDouble() };
        let r2 = unsafe { RandomDouble() };
        let r2s = r2.sqrt();

        let w = nl;
        let u = if w.x.abs() > 0.1 { Vec::new(0.0, 1.0, 0.0) } else { Vec::new(1.0, 0.0, 0.0) } % w;
        let u = u.norm();
        let v = w % u;
        let d = (u * r1.cos() * r2s + v * r1.sin() * r2s + w * (1.0 - r2).sqrt()).norm();

        return obj.e + f.mult(radiance(&Ray { o: x, d }, depth));
    } else if obj.refl == ReflT::SPEC {
        return obj.e + f.mult(radiance(&Ray { o: x, d: r.d - n * 2.0 * n.dot(r.d) }, depth));
    }

    let refl_ray = Ray { o: x, d: r.d - n * 2.0 * n.dot(r.d) };
    let into = n.dot(nl) > 0.0;
    let nc = 1.0;
    let nt = 1.5;
    let nnt = if into { nc / nt } else { nt / nc };
    let ddn = r.d.dot(nl);
    let cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);

    if cos2t < 0.0 {
        return obj.e + f.mult(radiance(&refl_ray, depth));
    }

    let tdir = (r.d * nnt - n * ((if into { 1.0 } else { -1.0 }) * (ddn * nnt + cos2t.sqrt()))).norm();
    let a = nt - nc;
    let b = nt + nc;
    let r0 = a * a / (b * b);
    let c = 1.0 - if into { -ddn } else { tdir.dot(n) };
    let re = r0 + (1.0 - r0) * c.powi(5);
    let tr = 1.0 - re;
    let p = 0.25 + 0.5 * re;
    let rp = re / p;
    let tp = tr / (1.0 - p);

    if depth > 2 {
        if unsafe { RandomDouble() } < p {
            return obj.e + f.mult(radiance(&refl_ray, depth) * rp);
        } else {
            return obj.e + f.mult(radiance(&Ray { o: x, d: tdir }, depth) * tp);
        }
    } else {
        return obj.e + f.mult(radiance(&refl_ray, depth) * re + radiance(&Ray { o: x, d: tdir }, depth) * tr);
    }
}

fn run() {
    unsafe { UseDoubleBuffering(true) };

    let cam = Ray { o: Vec::new(50.0, 50.0, 295.6), d: Vec::new(0.0, -0.04, -1.0).norm() };
    
    let cx = unsafe { Vec::new(Width() * 0.5135 / Height(), 0.0, 0.0) };
    let cy = (cx % cam.d).norm() * 0.5135;

    let (width, height) = unsafe {
        (Width() as usize, Height() as usize)
    };

    (0..height).for_each(|y| {
        (0..width).for_each(|x| {
            let mut c = Vec::default();
            (0..2).for_each(|sy| {
                (0..2).for_each(|sx| {
                    let mut r = Vec::default();
                    (0..SAMPLES).for_each(|_| {
                        let r1 = 2.0 * unsafe { RandomDouble() };
                        let dx = if r1 < 1.0 { r1.sqrt() - 1.0 } else { 1.0 - (2.0 - r1).sqrt() };
                        let r2 = 2.0 * unsafe { RandomDouble() };
                        let dy = if r2 < 1.0 { r2.sqrt() - 1.0 } else { 1.0 - (2.0 - r2).sqrt() };
                        let d = cx * (((sx as f64 + 0.5 + dx) / 2.0 + x as f64) / unsafe { Width() } - 0.5)
                            + cy * (((sy as f64 + 0.5 + dy) / 2.0 + y as f64) / unsafe { Height() } - 0.5)
                            + cam.d;

                        r = r + radiance(&Ray { o: cam.o + d * 140.0, d: d.norm() }, 0) * (1.0 / SAMPLES as f64);
                    });
                    c = c + Vec::new(clamp(r.x), clamp(r.y), clamp(r.z)) * 0.25;
                });
            });
            unsafe {
                DrawPixel(
                    x as i32,
                    height as i32 - y as i32 - 1,
                    MakeColor(gamma(c.x), gamma(c.y), gamma(c.z)),
                );
            }
        });
        unsafe { Present(); }
    });

    unsafe { SaveImage(); }
}

fn main() {
    run();
}