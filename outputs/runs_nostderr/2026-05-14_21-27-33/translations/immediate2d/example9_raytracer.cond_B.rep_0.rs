use std::f64::consts::PI;
use std::ops::Rem;

// Removed unused import of TAU from std
const TAU: f64 = 2.0 * PI;

static SAMPLES: usize = 200;
static WIDTH: usize = 640;
static HEIGHT: usize = 480;

fn random_double() -> f64 {
    // Removed the usage of the rand crate
    // Placeholder function to simulate random number generation
    0.5
}

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

    fn mult(&self, b: &Vec) -> Vec {
        Vec::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }

    fn norm(&mut self) -> &mut Vec {
        let norm = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self.x *= norm;
        self.y *= norm;
        self.z *= norm;
        self
    }

    fn dot(&self, b: &Vec) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
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

impl std::ops::Mul for Vec {
    type Output = Vec;

    fn mul(self, b: Vec) -> Vec {
        Vec::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }
}

impl std::ops::Mul<f64> for Vec {
    type Output = Vec;

    fn mul(self, b: f64) -> Vec {
        Vec::new(self.x * b, self.y * b, self.z * b)
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

#[derive(Clone)]
struct Ray {
    o: Vec,
    d: Vec,
}

#[derive(Clone, Copy, PartialEq)]
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
            let t = b - det;
            if t > eps {
                t
            } else {
                let t = b + det;
                if t > eps {
                    t
                } else {
                    0.0
                }
            }
        }
    }
}

fn sphere_array() -> [Sphere; 9] {
    [
        Sphere::new(
            1e5,
            Vec::new(1e5 + 1.0, 40.8, 81.6),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.75, 0.25, 0.25),
            ReflT::DIFF,
        ),
        Sphere::new(
            1e5,
            Vec::new(-1e5 + 99.0, 40.8, 81.6),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.25, 0.6, 0.15),
            ReflT::DIFF,
        ),
        Sphere::new(
            1e5,
            Vec::new(50.0, 40.8, 1e5),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.75, 0.75, 0.75),
            ReflT::DIFF,
        ),
        Sphere::new(
            1e5,
            Vec::new(50.0, 40.8, -1e5 + 170.0),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.0, 0.0, 0.0),
            ReflT::DIFF,
        ),
        Sphere::new(
            1e5,
            Vec::new(50.0, 1e5, 81.6),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.75, 0.75, 0.75),
            ReflT::DIFF,
        ),
        Sphere::new(
            1e5,
            Vec::new(50.0, -1e5 + 81.6, 81.6),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.75, 0.75, 0.75),
            ReflT::DIFF,
        ),
        Sphere::new(
            16.5,
            Vec::new(27.0, 16.5, 47.0),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.3, 0.3, 1.0) * 0.9,
            ReflT::SPEC,
        ),
        Sphere::new(
            16.5,
            Vec::new(73.0, 16.5, 78.0),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(1.0, 1.0, 1.0) * 0.9,
            ReflT::REFR,
        ),
        Sphere::new(
            600.0,
            Vec::new(50.0, 681.6 - 0.27, 81.6),
            Vec::new(12.0, 12.0, 12.0),
            Vec::new(0.0, 0.0, 0.0),
            ReflT::DIFF,
        ),
    ]
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

fn intersect(r: &Ray, t: &mut f64, id: &mut usize) -> bool {
    let inf = 1e20;
    *t = inf;
    let spheres = sphere_array();
    let n = spheres.len();

    for i in 0..n {
        let d = spheres[i].intersect(r);
        if d != 0.0 && d < *t {
            *t = d;
            *id = i;
        }
    }

    *t < inf
}

fn radiance(r: &Ray, depth: usize) -> Vec {
    let mut t = 0.0;
    let mut id = 0;
    let spheres = sphere_array();

    if !intersect(r, &mut t, &mut id) {
        return Vec::new(0.0, 0.0, 0.0);
    }

    let obj = spheres[id];
    let mut x = r.o + r.d * t;
    let mut n = (x - obj.p).norm().clone();
    let nl = if n.dot(&r.d) < 0.0 { n } else { n * -1.0 };
    let f = obj.c;

    let mut p = if f.x > f.y && f.x > f.z {
        f.x
    } else if f.y > f.z {
        f.y
    } else {
        f.z
    };

    if depth > 5 {
        if random_double() < p {
            p = 1.0 / p;
        } else {
            return obj.e;
        }
    }

    if obj.refl == ReflT::DIFF {
        let r1 = TAU * random_double();
        let r2 = random_double();
        let r2s = r2.sqrt();

        let w = nl.clone();
        let u = if w.x.abs() > 0.1 {
            Vec::new(0.0, 1.0, 0.0)
        } else {
            Vec::new(1.0, 0.0, 0.0)
        }
        .rem(w)
        .norm()
        .clone();
        let v = w.rem(u);
        let d = (u * r1.cos() * r2s + v * r1.sin() * r2s + w * (1.0 - r2).sqrt()).norm().clone();
        obj.e + f.mult(&radiance(&Ray { o: x, d }, depth + 1))
    } else if obj.refl == ReflT::SPEC {
        obj.e + f.mult(&radiance(&Ray { o: x, d: r.d - n * 2.0 * n.dot(&r.d) }, depth + 1))
    } else {
        let refl_ray = Ray {
            o: x,
            d: r.d - n * 2.0 * n.dot(&r.d),
        };

        let into = n.dot(&nl) > 0.0;
        let nc = 1.0;
        let nt = 1.5;
        let nnt = if into { nc / nt } else { nt / nc };
        let ddn = r.d.dot(&nl);
        let mut cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);

        if cos2t < 0.0 {
            return obj.e + f.mult(&radiance(&refl_ray, depth + 1));
        }

        let tdir = (r.d * nnt
            - n * ((if into { 1.0 } else { -1.0 }) * (ddn * nnt + cos2t.sqrt())))
        .norm()
        .clone();
        let a = nt - nc;
        let b = nt + nc;
        let r0 = (a * a) / (b * b);
        let c = 1.0 - (if into { -ddn } else { tdir.dot(&n) });
        let re = r0 + (1.0 - r0) * c * c * c * c * c;
        let tr = 1.0 - re;
        let p = 0.25 + 0.5 * re;
        let rp = re / p;
        let tp = tr / (1.0 - p);

        obj.e + f.mult(if depth > 2 {
            if random_double() < p {
                &radiance(&refl_ray, depth + 1) * rp
            } else {
                &radiance(&Ray { o: x, d: tdir }, depth + 1) * tp
            }
        } else {
            &radiance(&refl_ray, depth + 1) * re
                + &radiance(&Ray { o: x, d: tdir }, depth + 1) * tr
        })
    }
}

fn run() {
    let cam = Ray {
        o: Vec::new(50.0, 50.0, 295.6),
        d: Vec::new(0.0, -0.04, -1.0).norm().clone(),
    };
    let cx = Vec::new(WIDTH as f64 * 0.5135 / HEIGHT as f64, 0.0, 0.0);
    let cy = (cx.rem(cam.d)).norm().clone() * 0.5135;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut c = Vec::new(0.0, 0.0, 0.0);

            for sy in 0..2 {
                for sx in 0..2 {
                    let mut r = Vec::new(0.0, 0.0, 0.0);
                    for _ in 0..SAMPLES {
                        let r1 = 2.0 * random_double();
                        let dx = if r1 < 1.0 {
                            r1.sqrt() - 1.0
                        } else {
                            1.0 - (2.0 - r1).sqrt()
                        };

                        let r2 = 2.0 * random_double();
                        let dy = if r2 < 1.0 {
                            r2.sqrt() - 1.0
                        } else {
                            1.0 - (2.0 - r2).sqrt()
                        };

                        let mut d = cx * (((sx as f64 + 0.5 + dx) / 2.0 + x as f64) / WIDTH as f64 - 0.5)
                            + cy * (((sy as f64 + 0.5 + dy) / 2.0 + y as f64) / HEIGHT as f64 - 0.5)
                            + cam.d;
                        r = r + radiance(&Ray { o: cam.o + d * 140.0, d: d.norm().clone() }, 0)
                            * (1.0 / SAMPLES as f64);
                    }
                    c = c + Vec::new(clamp(r.x), clamp(r.y), clamp(r.z)) * 0.25;
                }
            }

            let _pixel_color = (gamma(c.x) as u8, gamma(c.y) as u8, gamma(c.z) as u8);
        }
    }
}

fn main() {
    run();
}