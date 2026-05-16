use std::f64::consts::PI;
use std::sync::OnceLock;
use std::ops::{Mul, Add, Sub, Rem};
use std::f64;

// Constants
const TAU: f64 = 2.0 * PI;
const SAMPLES: i32 = 200;
const WIDTH: usize = 800;  // Set your actual width
const HEIGHT: usize = 600; // Set your actual height

#[derive(Copy, Clone, Debug)]
struct Vec {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec {
    const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn mult(&self, b: Vec) -> Vec {
        Vec::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }

    fn norm(&self) -> Vec {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        *self * (1.0 / length)
    }

    fn dot(&self, b: Vec) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }

    fn operator_mul(&self, b: f64) -> Vec {
        Vec::new(self.x * b, self.y * b, self.z * b)
    }

    fn operator_add(&self, b: Vec) -> Vec {
        Vec::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }

    fn operator_sub(&self, b: Vec) -> Vec {
        Vec::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }

    fn operator_mod(&self, b: Vec) -> Vec {
        Vec::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
}

impl Mul<f64> for Vec {
    type Output = Vec;

    fn mul(self, rhs: f64) -> Vec {
        self.operator_mul(rhs)
    }
}

impl Add for Vec {
    type Output = Vec;

    fn add(self, rhs: Vec) -> Vec {
        self.operator_add(rhs)
    }
}

impl Sub for Vec {
    type Output = Vec;

    fn sub(self, rhs: Vec) -> Vec {
        self.operator_sub(rhs)
    }
}

impl Rem for Vec {
    type Output = Vec;

    fn rem(self, rhs: Vec) -> Vec {
        self.operator_mod(rhs)
    }
}

#[derive(Copy, Clone, Debug)]
struct Ray {
    o: Vec,
    d: Vec,
}

#[derive(Copy, Clone, Debug)]
enum ReflType {
    Diffuse,
    Specular,
    Refractive,
}

#[derive(Copy, Clone, Debug)]
struct Sphere {
    rad: f64,
    p: Vec,
    e: Vec,
    c: Vec,
    refl: ReflType,
}

impl Sphere {
    fn intersect(&self, r: &Ray) -> f64 {
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
                if t > eps {
                    t
                } else {
                    0.0
                }
            }
        }
    }
}

static SPHERES: OnceLock<Vec<Sphere>> = OnceLock::new();

fn get_spheres() -> &'static [Sphere] {
    SPHERES.get_or_init(|| vec![
        Sphere {
            rad: 1e5,
            p: Vec::new(1e5 + 1.0, 40.8, 81.6),
            e: Vec::new(0.0, 0.0, 0.0),
            c: Vec::new(0.75, 0.25, 0.25),
            refl: ReflType::Diffuse,
        },
        Sphere {
            rad: 1e5,
            p: Vec::new(-1e5 + 99.0, 40.8, 81.6),
            e: Vec::new(0.0, 0.0, 0.0),
            c: Vec::new(0.25, 0.6, 0.15),
            refl: ReflType::Diffuse,
        },
        Sphere {
            rad: 1e5,
            p: Vec::new(50.0, 40.8, 1e5),
            e: Vec::new(0.0, 0.0, 0.0),
            c: Vec::new(0.75, 0.75, 0.75),
            refl: ReflType::Diffuse,
        },
        Sphere {
            rad: 1e5,
            p: Vec::new(50.0, 40.8, -1e5 + 170.0),
            e: Vec::new(0.0, 0.0, 0.0),
            c: Vec::new(0.0, 0.0, 0.0),
            refl: ReflType::Diffuse,
        },
        Sphere {
            rad: 1e5,
            p: Vec::new(50.0, 1e5, 81.6),
            e: Vec::new(0.0, 0.0, 0.0),
            c: Vec::new(0.75, 0.75, 0.75),
            refl: ReflType::Diffuse,
        },
        Sphere {
            rad: 1e5,
            p: Vec::new(50.0, -1e5 + 81.6, 81.6),
            e: Vec::new(0.0, 0.0, 0.0),
            c: Vec::new(0.75, 0.75, 0.75),
            refl: ReflType::Diffuse,
        },
        Sphere {
            rad: 16.5,
            p: Vec::new(27.0, 16.5, 47.0),
            e: Vec::new(0.0, 0.0, 0.0),
            c: Vec::new(0.3, 0.3, 1.0) * 0.9,
            refl: ReflType::Specular,
        },
        Sphere {
            rad: 16.5,
            p: Vec::new(73.0, 16.5, 78.0),
            e: Vec::new(0.0, 0.0, 0.0),
            c: Vec::new(1.0, 1.0, 1.0) * 0.9,
            refl: ReflType::Refractive,
        },
        Sphere {
            rad: 600.0,
            p: Vec::new(50.0, 681.6 - 0.27, 81.6),
            e: Vec::new(12.0, 12.0, 12.0),
            c: Vec::new(0.0, 0.0, 0.0),
            refl: ReflType::Diffuse,
        },
    ])
}

fn clamp(x: f64) -> f64 {
    x.min(1.0).max(0.0)
}

fn gamma(x: f64) -> i32 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5).floor() as i32
}

fn intersect(r: &Ray, t: &mut f64, id: &mut usize) -> bool {
    let spheres = get_spheres();
    let n = spheres.len();
    let inf = 1e20;
    *t = inf;
    for i in 0..n {
        let d = spheres[i].intersect(r);
        if d > 0.0 && d < *t {
            *t = d;
            *id = i;
        }
    }
    *t < inf
}

fn random_double() -> f64 {
    f64::consts::PI.sin() // This is a placeholder for actual random generation
}

fn radiance(r: &Ray, depth: i32) -> Vec {
    let spheres = get_spheres();
    let mut t = 0.0;
    let mut id = 0;
    if !intersect(r, &mut t, &mut id) {
        return Vec::new(0.0, 0.0, 0.0);
    }
    let obj = &spheres[id];
    let x = r.o + r.d * t;
    let n = (x - obj.p).norm();
    let nl = if n.dot(r.d) < 0.0 { n } else { n * -1.0 };
    let f = obj.c;
    let p = f.x.max(f.y).max(f.z);
    if depth > 5 {
        if random_double() < p {
            return obj.e + f.mult(radiance(r, depth - 1)) * (1.0 / p);
        } else {
            return obj.e;
        }
    }

    match obj.refl {
        ReflType::Diffuse => {
            let r1 = TAU * random_double();
            let r2 = random_double();
            let r2s = r2.sqrt();
            let w = nl;
            let u = (if w.x.abs() > 0.1 {
                Vec::new(0.0, 1.0, 0.0)
            } else {
                Vec::new(1.0, 0.0, 0.0)
            }) % w;
            let u = u.norm();
            let v = w % u;
            let d = (u * (r1.cos() * r2s) + v * (r1.sin() * r2s) + w * (1.0 - r2).sqrt()).norm();
            obj.e + f.mult(radiance(&Ray { o: x, d }, depth + 1))
        }
        ReflType::Specular => {
            let reflected_ray = Ray {
                o: x,
                d: r.d - n * 2.0 * n.dot(r.d),
            };
            obj.e + f.mult(radiance(&reflected_ray, depth + 1))
        }
        ReflType::Refractive => {
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
                return obj.e + f.mult(radiance(&refl_ray, depth + 1));
            }
            let tdir = (r.d * nnt - n * ((if into { 1.0 } else { -1.0 }) * (ddn * nnt + cos2t.sqrt()))).norm();
            let a = nt - nc;
            let b = nt + nc;
            let r0 = (a * a) / (b * b);
            let c = 1.0 - if into { -ddn } else { tdir.dot(n) };
            let re = r0 + (1.0 - r0) * c * c * c * c * c;
            let tr = 1.0 - re;
            let p_re = 0.25 + 0.5 * re;
            let rp = re / p_re;
            let tp = tr / (1.0 - p_re);
            if depth > 2 {
                if random_double() < p_re {
                    return obj.e + f.mult(radiance(&refl_ray, depth + 1) * rp);
                } else {
                    return obj.e + f.mult(radiance(&Ray { o: x, d: tdir }, depth + 1) * tp);
                }
            }
            obj.e
                + f.mult(
                    radiance(&refl_ray, depth + 1) * re
                        + radiance(&Ray { o: x, d: tdir }, depth + 1) * tr,
                )
        }
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
                    for _ in 0..SAMPLES {
                        let r1 = 2.0 * random_double();
                        let dx = if r1 < 1.0 { r1.sqrt() - 1.0 } else { 1.0 - (2.0 - r1).sqrt() };
                        let r2 = 2.0 * random_double();
                        let dy = if r2 < 1.0 { r2.sqrt() - 1.0 } else { 1.0 - (2.0 - r2).sqrt() };
                        let d = cx * (((sx as f64 + 0.5 + dx) / 2.0 + x as f64) / WIDTH as f64 - 0.5)
                            + cy * (((sy as f64 + 0.5 + dy) / 2.0 + y as f64) / HEIGHT as f64 - 0.5) 
                            + cam.d;
                        r = r + radiance(&Ray { o: cam.o + d * 140.0, d: d.norm() }, 0) * (1.0 / SAMPLES as f64);
                    }
                    c = c + Vec::new(clamp(r.x), clamp(r.y), clamp(r.z)) * 0.25;
                }
            }
            // Assuming `draw_pixel` function exists
            // draw_pixel(x, HEIGHT - y - 1, make_color(gamma(c.x), gamma(c.y), gamma(c.z)));
        }
        // Assuming `present` and `save_image` functions exist
        // present();
    }
    // save_image();
}

fn main() {
    run();
}