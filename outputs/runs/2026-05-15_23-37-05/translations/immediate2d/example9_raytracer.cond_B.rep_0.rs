use std::f64::consts::PI;
use std::sync::LazyLock;
use std::ops::{Mul, Add, Sub, Rem};
use std::f64;

const SAMPLES: usize = 200;
const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

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

    fn mult(&self, b: &Vec) -> Vec {
        Vec::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }

    fn norm(&mut self) -> &mut Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self.x /= len;
        self.y /= len;
        self.z /= len;
        self
    }

    fn dot(&self, b: &Vec) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
}

impl Mul<f64> for Vec {
    type Output = Vec;

    fn mul(self, b: f64) -> Vec {
        Vec::new(self.x * b, self.y * b, self.z * b)
    }
}

impl Add for Vec {
    type Output = Vec;

    fn add(self, b: Vec) -> Vec {
        Vec::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}

impl Sub for Vec {
    type Output = Vec;

    fn sub(self, b: Vec) -> Vec {
        Vec::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}

impl Rem for Vec {
    type Output = Vec;

    fn rem(self, b: Vec) -> Vec {
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
        let b = op.dot(&r.d);
        let det = b * b - op.dot(&op) + self.rad * self.rad;
        if det < 0.0 {
            0.0
        } else {
            let det = det.sqrt();
            let mut t = b - det;
            if t > eps {
                t
            } else {
                t = b + det;
                if t > eps {
                    t
                } else {
                    0.0
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

fn gamma(x: f64) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}

static SPHERES: LazyLock<[Sphere; 9]> = LazyLock::new(|| [
    Sphere::new(1e5, Vec::new(1e5 + 1.0, 40.8, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.25, 0.25), ReflT::DIFF),
    Sphere::new(1e5, Vec::new(-1e5 + 99.0, 40.8, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.25, 0.6, 0.15), ReflT::DIFF),
    Sphere::new(1e5, Vec::new(50.0, 40.8, 1e5), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::DIFF),
    Sphere::new(1e5, Vec::new(50.0, 40.8, -1e5 + 170.0), Vec::new(0.0, 0.0, 0.0), Vec::new(0.0, 0.0, 0.0), ReflT::DIFF),
    Sphere::new(1e5, Vec::new(50.0, 1e5, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::DIFF),
    Sphere::new(1e5, Vec::new(50.0, -1e5 + 81.6, 81.6), Vec::new(0.0, 0.0, 0.0), Vec::new(0.75, 0.75, 0.75), ReflT::DIFF),
    Sphere::new(16.5, Vec::new(27.0, 16.5, 47.0), Vec::new(0.0, 0.0, 0.0), Vec::new(0.3, 0.3, 1.0) * 0.9, ReflT::SPEC),
    Sphere::new(16.5, Vec::new(73.0, 16.5, 78.0), Vec::new(0.0, 0.0, 0.0), Vec::new(1.0, 1.0, 1.0) * 0.9, ReflT::REFR),
    Sphere::new(600.0, Vec::new(50.0, 681.6 - 0.27, 81.6), Vec::new(12.0, 12.0, 12.0), Vec::new(0.0, 0.0, 0.0), ReflT::DIFF),
]);

unsafe fn intersect(r: &Ray, t: &mut f64, id: &mut usize) -> bool {
    let n = SPHERES.len();
    let mut d;
    let inf = 1e20;
    *t = inf;
    for i in 0..n {
        if { d = SPHERES[i].intersect(r); d > 0.0 && d < *t } {
            *t = d;
            *id = i;
        }
    }
    *t < inf
}

fn rand() -> f64 {
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as f64)
        / u32::MAX as f64
}

fn radiance(ray: Ray, depth: usize) -> Vec {
    let mut t = 0.0;
    let mut id = 0;
    if !unsafe { intersect(&ray, &mut t, &mut id) } {
        return Vec::new(0.0, 0.0, 0.0);
    }

    unsafe {
        let obj = &SPHERES[id];
        let x = ray.o + ray.d * t;
        let mut n = x - obj.p;
        n.norm();
        let nl = if n.dot(&ray.d) < 0.0 { n } else { n * -1.0 };
        let f = obj.c;

        let p = {
            if f.x > f.y && f.x > f.z {
                f.x
            } else {
                if f.y > f.z {
                    f.y
                } else {
                    f.z
                }
            }
        };

        let mut rng = rand();
        let next_rng = rand();

        if depth > 5 {
            if rng < p {
                rng = rng * (1.0 / p);
            } else {
                return obj.e;
            }
        }

        if obj.refl == ReflT::DIFF {
            let r1 = 2.0 * PI * next_rng;
            let r2 = rng;
            let r2s = r2.sqrt();
            let w = nl;
            let mut u = if w.x.abs() > 0.1 { Vec::new(0.0, 1.0, 0.0) } else { Vec::new(1.0, 0.0, 0.0) };
            u = *(u % w).norm();
            let v = w % u;
            let d = *(u * (r1.cos() * r2s) + v * (r1.sin() * r2s) + w * (1.0 - r2).sqrt()).norm();
            return obj.e + f.mult(&radiance(Ray { o: x, d }, depth + 1));
        } else if obj.refl == ReflT::SPEC {
            return obj.e + f.mult(&radiance(Ray { o: x, d: ray.d - n * 2.0 * n.dot(&ray.d) }, depth + 1));
        } else {
            let reflRay = Ray { o: x, d: ray.d - n * 2.0 * n.dot(&ray.d) };
            let into = n.dot(&nl) > 0.0;
            let nc = 1.0;
            let nt = 1.5;
            let nnt = if into { nc / nt } else { nt / nc };
            let ddn = ray.d.dot(&nl);
            let cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);

            if cos2t < 0.0 {
                return obj.e + f.mult(&radiance(reflRay, depth + 1));
            }

            let tdir = *(ray.d * nnt - n * ((if into { 1.0 } else { -1.0 }) * (ddn * nnt + cos2t.sqrt()))).norm();
            let a = nt - nc;
            let b = nt + nc;
            let R0 = a * a / (b * b);
            let c = 1.0 - if into { -ddn } else { tdir.dot(&n) };
            let Re = R0 + (1.0 - R0) * c * c * c * c * c;
            let Tr = 1.0 - Re;
            let P = 0.25 + 0.5 * Re;
            let RP = Re / P;
            let TP = Tr / (1.0 - P);

            return obj.e
                + f.mult(
                    if depth > 2 {
                        if rng < P {
                            &radiance(reflRay, depth + 1) * RP
                        } else {
                            &radiance(Ray { o: x, d: tdir }, depth + 1) * TP
                        }
                    } else {
                        &radiance(reflRay, depth + 1) * Re + &radiance(Ray { o: x, d: tdir }, depth + 1) * Tr
                    }
                );
        }
    }
}

fn run() {
    let cam = Ray { o: Vec::new(50.0, 50.0, 295.6), d: *Vec::new(0.0, -0.04, -1.0).norm() };
    let cx = Vec::new(WIDTH as f64 * 0.5135 / HEIGHT as f64, 0.0, 0.0);
    let cy = *(cx % cam.d).norm() * 0.5135;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut c = Vec::new(0.0, 0.0, 0.0);

            for sy in 0..2 {
                for sx in 0..2 {
                    let mut r = Vec::new(0.0, 0.0, 0.0);

                    for _ in 0..SAMPLES {
                        let r1: f64 = 2.0 * rand();
                        let dx = if r1 < 1.0 { r1.sqrt() - 1.0 } else { 1.0 - (2.0 - r1).sqrt() };
                        let r2: f64 = 2.0 * rand();
                        let dy = if r2 < 1.0 { r2.sqrt() - 1.0 } else { 1.0 - (2.0 - r2).sqrt() };

                        let mut d = cx * (((sx as f64 + 0.5 + dx) / 2.0 + x as f64) / WIDTH as f64 - 0.5)
                            + cy * (((sy as f64 + 0.5 + dy) / 2.0 + y as f64) / HEIGHT as f64 - 0.5)
                            + cam.d;

                        r = r + radiance(
                            Ray {
                                o: cam.o + d * 140.0,
                                d: *d.norm(),
                            },
                            0,
                        ) * (1.0 / SAMPLES as f64);
                    }

                    c = c + Vec::new(clamp(r.x), clamp(r.y), clamp(r.z)) * 0.25;
                }
            }
        }
    }
}

fn main() {
    run();
}