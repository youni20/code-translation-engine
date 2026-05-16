use std::f64::consts::PI;
use std::f64::INFINITY;
use std::ops::{Add, Mul, Sub, Rem};

const SAMPLES: usize = 200;
const W: f64 = 1.0;
const H: f64 = 1.0;
const WIDTH: usize = 800;
const HEIGHT: usize = 600;

#[derive(Copy, Clone, Debug)]
struct Vec {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vec { x, y, z }
    }

    fn norm(&self) -> Vec {
        let norm = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        *self * norm
    }

    fn mult(&self, b: Vec) -> Vec {
        Vec::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }

    fn dot(&self, b: Vec) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
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

impl Mul<f64> for Vec {
    type Output = Vec;

    fn mul(self, b: f64) -> Vec {
        Vec::new(self.x * b, self.y * b, self.z * b)
    }
}

impl Rem<Vec> for Vec {
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

#[derive(Copy, Clone)]
enum Refl {
    Diff,
    Spec,
    Refr,
}

struct Sphere {
    rad: f64,
    p: Vec,
    e: Vec,
    c: Vec,
    refl: Refl,
}

impl Sphere {
    fn new(rad: f64, p: Vec, e: Vec, c: Vec, refl: Refl) -> Self {
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
        let t1 = b - det_sqrt;
        let t2 = b + det_sqrt;

        if t1 > eps {
            t1
        } else if t2 > eps {
            t2
        } else {
            0.0
        }
    }
}

fn create_spheres() -> std::vec::Vec<Sphere> {
    vec![
        Sphere::new(
            1e5,
            Vec::new(1e5 + 1.0, 40.8, 81.6),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.75, 0.25, 0.25),
            Refl::Diff,
        ),
        Sphere::new(
            1e5,
            Vec::new(-1e5 + 99.0, 40.8, 81.6),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.25, 0.6, 0.15),
            Refl::Diff,
        ),
        Sphere::new(
            1e5,
            Vec::new(50.0, 40.8, 1e5),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.75, 0.75, 0.75),
            Refl::Diff,
        ),
        Sphere::new(
            1e5,
            Vec::new(50.0, 40.8, -1e5 + 170.0),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.0, 0.0, 0.0),
            Refl::Diff,
        ),
        Sphere::new(
            1e5,
            Vec::new(50.0, 1e5, 81.6),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.75, 0.75, 0.75),
            Refl::Diff,
        ),
        Sphere::new(
            1e5,
            Vec::new(50.0, -1e5 + 81.6, 81.6),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.75, 0.75, 0.75),
            Refl::Diff,
        ),
        Sphere::new(
            16.5,
            Vec::new(27.0, 16.5, 47.0),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(0.3, 0.3, 1.0) * 0.9,
            Refl::Spec,
        ),
        Sphere::new(
            16.5,
            Vec::new(73.0, 16.5, 78.0),
            Vec::new(0.0, 0.0, 0.0),
            Vec::new(1.0, 1.0, 1.0) * 0.9,
            Refl::Refr,
        ),
        Sphere::new(
            600.0,
            Vec::new(50.0, 681.6 - 0.27, 81.6),
            Vec::new(12.0, 12.0, 12.0),
            Vec::new(0.0, 0.0, 0.0),
            Refl::Diff,
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

fn gamma(x: f64) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}

fn intersect(r: &Ray, id: &mut usize) -> Option<f64> {
    let spheres = create_spheres();
    let mut inf = INFINITY;
    let mut d;
    let mut hit = false;
    for i in 0..spheres.len() {
        d = spheres[i].intersect(r);
        if d > 0.0 && d < inf {
            inf = d;
            *id = i;
            hit = true;
        }
    }
    if hit { Some(inf) } else { None }
}

fn random_double() -> f64 {
    // Simple random generation using a constant seed for reproducibility
    0.5 // Placeholder for deterministic value
}

fn radiance(r: &Ray, mut depth: i32) -> Vec {
    let spheres = create_spheres();
    let mut id = 0;
    if let Some(t) = intersect(r, &mut id) {
        let obj = &spheres[id];
        let x = r.o + r.d * t;
        let n = (x - obj.p).norm();
        let nl = if n.dot(r.d) < 0.0 {
            n
        } else {
            n * -1.0
        };
        let f = obj.c;

        let mut p = f.x.max(f.y).max(f.z);

        if depth >= 5 {
            if random_double() < p {
                return f * (1.0 / p);
            } else {
                return obj.e;
            }
        }
        depth += 1;

        match obj.refl {
            Refl::Diff => {
                let r1 = 2.0 * PI * random_double();
                let r2 = random_double();
                let r2s = r2.sqrt();

                let w = nl;
                let u = ((if w.x.abs() > 0.1 { Vec::new(0.0, 1.0, 0.0) } else { Vec::new(1.0, 0.0, 0.0) })
                    % w)
                    .norm();
                let v = w % u;
                let d = (u * r1.cos() * r2s + v * r1.sin() * r2s + w * (1.0 - r2).sqrt()).norm();
                obj.e + f.mult(radiance(&Ray { o: x, d }, depth))
            }

            Refl::Spec => {
                let refl_ray = Ray {
                    o: x,
                    d: r.d - n * 2.0 * n.dot(r.d),
                };
                obj.e + f.mult(radiance(&refl_ray, depth))
            }

            Refl::Refr => {
                let refl_ray = Ray {
                    o: x,
                    d: r.d - n * 2.0 * n.dot(r.d),
                };
                let into = n.dot(nl) > 0.0;
                let nc = 1.0;
                let nt = 1.5;
                let nnt = if into { nc / nt } else { nt / nc };
                let ddn = r.d.dot(nl);
                let cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);
                if cos2t < 0.0 {
                    return obj.e + f.mult(radiance(&refl_ray, depth));
                }
                let tdir = (r.d * nnt - n * ((if into {
                    1.0
                } else {
                    -1.0
                }) * (ddn * nnt + cos2t.sqrt())))
                .norm();
                let a = nt - nc;
                let b = nt + nc;
                let r0 = a * a / (b * b);
                let c = 1.0 - if into { -ddn } else { tdir.dot(n) };
                let re = r0 + (1.0 - r0) * c * c * c * c * c;
                let tr = 1.0 - re;
                let p = 0.25 + 0.5 * re;
                let rp = re / p;
                let tp = tr / (1.0 - p);

                obj.e + f.mult(if depth > 2 {
                    if random_double() < p {
                        radiance(&refl_ray, depth) * rp
                    } else {
                        radiance(&Ray { o: x, d: tdir }, depth) * tp
                    }
                } else {
                    radiance(&refl_ray, depth) * re + radiance(&Ray { o: x, d: tdir }, depth) * tr
                })
            }
        }
    } else {
        Vec::new(0.0, 0.0, 0.0)
    }
}

fn run() {
    let cam_pos = Vec::new(50.0, 50.0, 295.6);
    let cam_dir = Vec::new(0.0, -0.04, -1.0).norm();
    let cam = Ray {
        o: cam_pos,
        d: cam_dir,
    };

    let cx = Vec::new(WIDTH as f64 * W / HEIGHT as f64, 0.0, 0.0);
    let cy = (cx % cam.d).norm() * 0.5135;

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

                        let d = cx * ((sx as f64 + 0.5 + dx) / 2.0 + x as f64 / WIDTH as f64 - 0.5)
                            + cy * ((sy as f64 + 0.5 + dy) / 2.0 + y as f64 / HEIGHT as f64 - 0.5)
                            + cam.d;

                        r = r + radiance(&Ray { o: cam.o + d * 140.0, d: d.norm() }, 0)
                            * (1.0 / SAMPLES as f64);
                    }

                    c = c + Vec::new(clamp(r.x), clamp(r.y), clamp(r.z)) * 0.25;
                }
            }

            let red = gamma(c.x);
            let green = gamma(c.y);
            let blue = gamma(c.z);
            println!("Pixel at {},{} Colored RGB({},{},{})", x, HEIGHT - y - 1, red, green, blue);
        }
    }
}

fn main() {
    run();
}