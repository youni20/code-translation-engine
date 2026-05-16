use std::f64;
use std::f64::consts::PI;
use std::sync::LazyLock;

// Simple pseudo-random number generator for double values
fn random_double() -> f64 {
    // Using a basic LCG for simplicity here.
    static mut STATE: u64 = 0;
    unsafe {
        STATE = STATE.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((STATE >> 12) as f64) / ((1u64 << 52) as f64)
    }
}

#[derive(Copy, Clone)]
struct Vec {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec {
    fn new(x: f64, y: f64, z: f64) -> Vec {
        Vec { x, y, z }
    }

    fn mult(self, b: Vec) -> Vec {
        Vec::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }

    fn norm(&mut self) -> Vec {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Vec::new(self.x / length, self.y / length, self.z / length)
    }

    fn dot(self, b: Vec) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
}

impl std::ops::Mul<f64> for Vec {
    type Output = Vec;
    fn mul(self, b: f64) -> Vec {
        Vec::new(self.x * b, self.y * b, self.z * b)
    }
}

impl std::ops::Add for Vec {
    type Output = Vec;
    fn add(self, b: Vec) -> Vec {
        Vec::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}

impl std::ops::Sub for Vec {
    type Output = Vec;
    fn sub(self, b: Vec) -> Vec {
        Vec::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}

impl std::ops::Rem for Vec {
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

#[derive(PartialEq)]
enum ReflT {
    DIFF,
    SPEC,
    REFR,
}

struct Sphere {
    rad: f64,  // radius
    p: Vec,    // position
    e: Vec,    // emission
    c: Vec,    // color
    refl: ReflT, // reflection type
}

impl Sphere {
    fn new(rad: f64, p: Vec, e: Vec, c: Vec, refl: ReflT) -> Sphere {
        Sphere { rad, p, e, c, refl }
    }

    fn intersect(&self, r: &Ray) -> f64 {
        let op = self.p - r.o;
        let eps = 1e-4;
        let b = op.dot(r.d);
        let det = b * b - op.dot(op) + self.rad * self.rad;
        if det < 0.0 {
            0.0
        } else {
            let det = det.sqrt();
            let t1 = b - det;
            if t1 > eps {
                t1
            } else {
                let t2 = b + det;
                if t2 > eps {
                    t2
                } else {
                    0.0
                }
            }
        }
    }
}

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
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

fn clamp(x: f64) -> f64 {
    if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
}

fn gamma(x: f64) -> i32 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as i32
}

fn intersect(r: &Ray) -> Option<(f64, usize)> {
    let mut t = 1e20f64;
    let mut id = 0;
    for i in 0..SPHERES.len() {
        let d = SPHERES[i].intersect(r);
        if d != 0.0 && d < t {
            t = d;
            id = i;
        }
    }
    if t < 1e20 {
        Some((t, id))
    } else {
        None
    }
}

fn radiance(r: Ray, depth: i32) -> Vec {
    if let Some((t, id)) = intersect(&r) {
        let obj = &SPHERES[id];
        let x = r.o + r.d * t;
        let mut n = x - obj.p;
        n = n.norm();
        let mut nl = n;
        if n.dot(r.d) >= 0.0 {
            nl = n * -1.0;
        }
        let mut f = obj.c;
        let p = if f.x > f.y && f.x > f.z { f.x } else if f.y > f.z { f.y } else { f.z };

        if depth > 5 {
            if random_double() < p {
                f = f * (1.0 / p);
            } else {
                return obj.e;
            }
        }

        if obj.refl == ReflT::DIFF {
            let r1 = 2.0 * PI * random_double();
            let r2 = random_double();
            let r2s = r2.sqrt();

            let w = nl;
            let mut u = if w.x.abs() > 0.1 { Vec::new(0.0, 1.0, 0.0) } else { Vec::new(1.0, 0.0, 0.0) } % w;
            u = u.norm();
            let v = w % u;
            let d = (u * f64::cos(r1) * r2s + v * f64::sin(r1) * r2s + w * (1.0 - r2).sqrt()).norm();
            return obj.e + f.mult(radiance(Ray { o: x, d }, depth + 1));
        }

        if obj.refl == ReflT::SPEC {
            return obj.e + f.mult(radiance(Ray { o: x, d: r.d - n * 2.0 * n.dot(r.d) }, depth + 1));
        }

        let refl_ray = Ray { o: x, d: r.d - n * 2.0 * n.dot(r.d) };
        let into = n.dot(nl) > 0.0;
        let nc = 1.0;
        let nt = 1.5;
        let nnt = if into { nc / nt } else { nt / nc };
        let ddn = r.d.dot(nl);
        let cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);

        if cos2t < 0.0 {
            return obj.e + f.mult(radiance(refl_ray, depth + 1));
        }

        let tdir = (r.d * nnt - n * ((if into { 1.0 } else { -1.0 }) * (ddn * nnt + cos2t.sqrt()))).norm();
        let a = nt - nc;
        let b = nt + nc;
        let r0 = a * a / (b * b);
        let c = 1.0 - if into { -ddn } else { tdir.dot(n) };
        let re = r0 + (1.0 - r0) * c * c * c * c * c;
        let tr = 1.0 - re;
        let p = 0.25 + 0.5 * re;
        let rp = re / p;
        let tp = tr / (1.0 - p);

        if depth > 2 {
            if random_double() < p {
                obj.e + f.mult(radiance(refl_ray, depth + 1) * rp)
            } else {
                obj.e + f.mult(radiance(Ray { o: x, d: tdir }, depth + 1) * tp)
            }
        } else {
            obj.e + f.mult(radiance(refl_ray, depth + 1) * re + radiance(Ray { o: x, d: tdir }, depth + 1) * tr)
        }
    } else {
        Vec::new(0.0, 0.0, 0.0)
    }
}

fn run() {
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
                    for _ in 0..200 {
                        let r1 = 2.0 * random_double();
                        let dx = if r1 < 1.0 { r1.sqrt() - 1.0 } else { 1.0 - (2.0 - r1).sqrt() };

                        let r2 = 2.0 * random_double();
                        let dy = if r2 < 1.0 { r2.sqrt() - 1.0 } else { 1.0 - (2.0 - r2).sqrt() };

                        let mut d = cx * (((sx as f64 + 0.5 + dx) / 2.0 + x as f64) / WIDTH as f64 - 0.5)
                            + cy * (((sy as f64 + 0.5 + dy) / 2.0 + y as f64) / HEIGHT as f64 - 0.5)
                            + cam.d;
                        r = r + radiance(Ray { o: cam.o + d * 140.0, d: d.norm() }, 0) * (1.0 / 200.0);
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