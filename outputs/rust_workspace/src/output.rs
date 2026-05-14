use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::{Add, Div, Mul, Sub};

const PI2: f32 = 2.0 * PI;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum DistanceType {
    Linear,
    InverseLinear,
    Quadratic,
    InverseQuadratic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn distance_to(&self, other: &Vec3) -> f32 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2) + (other.z - self.z).powi(2)).sqrt()
    }

    fn length(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    fn dot_product(&self, v: &Vec3) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    fn angle_to(&self, v: &Vec3) -> f32 {
        let l1 = self.length();
        let l2 = v.length();
        if l1 == 0.0 || l2 == 0.0 {
            return 0.0;
        }
        (self.dot_product(v) / (l1 * l2)).acos() * 360.0 / PI2
    }

    fn normalized(&self) -> Vec3 {
        let length = self.length();
        if length == 0.0 {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            Vec3::new(self.x / length, self.y / length, self.z / length)
        }
    }

    fn negative(&self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }

    fn clamp_length(&self, length: f32) -> Vec3 {
        let l = self.length();
        if l > length {
            self.normalized() * length
        } else {
            *self
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl std::hash::Hash for Vec3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

impl Eq for Vec3 {}

#[derive(Debug)]
struct Boid {
    position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
}

impl Boid {
    fn new(pos: Vec3, vel: Vec3) -> Boid {
        Boid {
            position: pos,
            velocity: vel,
            acceleration: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

struct NearbyBoid<'a> {
    boid: &'a Boid,
    direction: Vec3,
    distance: f32,
}

struct Swarm<'a> {
    perception_radius: f32,
    separation_weight: f32,
    separation_type: DistanceType,
    alignment_weight: f32,
    cohesion_weight: f32,
    steering_weight: f32,
    steering_targets: Vec<Vec3>,
    steering_target_type: DistanceType,
    blindspot_angle_deg: f32,
    max_acceleration: f32,
    max_velocity: f32,
    boids: &'a mut Vec<Boid>,
    voxel_cache: HashMap<Vec3, Vec<&'a Boid>>,
    blindspot_angle_deg_compare_value: f32,
}

impl<'a> Swarm<'a> {
    fn new(boids: &'a mut Vec<Boid>) -> Swarm<'a> {
        Swarm {
            perception_radius: 30.0,
            separation_weight: 1.0,
            separation_type: DistanceType::InverseQuadratic,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            steering_weight: 0.1,
            steering_targets: Vec::new(),
            steering_target_type: DistanceType::Linear,
            blindspot_angle_deg: 20.0,
            max_acceleration: 10.0,
            max_velocity: 20.0,
            boids,
            voxel_cache: HashMap::new(),
            blindspot_angle_deg_compare_value: (PI2 * 20.0 / 360.0).cos(),
        }
    }

    fn update(&mut self, delta: f32) {
        if self.perception_radius == 0.0 {
            self.perception_radius = 1.0;
        }
        self.update_acceleration();

        for b in self.boids.iter_mut() {
            b.velocity = (b.velocity + b.acceleration * delta).clamp_length(self.max_velocity);
            b.position = b.position + b.velocity * delta;
        }
    }

    fn update_acceleration(&mut self) {
        if self.perception_radius == 0.0 {
            self.perception_radius = 1.0;
        }
        self.build_voxel_cache();
        for boid in self.boids.iter_mut() {
            self.update_boid(boid);
        }
    }

    fn update_boid(&self, b: &mut Boid) {
        let mut separation_sum = Vec3::new(0.0, 0.0, 0.0);
        let mut heading_sum = Vec3::new(0.0, 0.0, 0.0);
        let mut position_sum = Vec3::new(0.0, 0.0, 0.0);

        let nearby = self.get_nearby_boids(b);

        for close_boid in nearby.iter() {
            if close_boid.distance == 0.0 {
                separation_sum = separation_sum + Vec3::new(rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>()).normalized() * 1000.0;
            } else {
                let separation_factor = self.transform_distance(close_boid.distance, &self.separation_type);
                separation_sum = separation_sum + close_boid.direction.negative() * separation_factor;
            }
            heading_sum = heading_sum + close_boid.boid.velocity;
            position_sum = position_sum + close_boid.boid.position;
        }

        let mut steering_target = b.position;
        let mut target_distance = -1.0;
        for target in self.steering_targets.iter() {
            let distance = self.transform_distance(target.distance_to(&b.position), &self.steering_target_type);
            if target_distance < 0.0 || distance < target_distance {
                steering_target = *target;
                target_distance = distance;
            }
        }

        let separation = if !nearby.is_empty() { separation_sum / nearby.len() as f32 } else { separation_sum };

        let alignment = if !nearby.is_empty() { heading_sum / nearby.len() as f32 } else { heading_sum };

        let avg_position = if !nearby.is_empty() { position_sum / nearby.len() as f32 } else { b.position };
        let cohesion = avg_position - b.position;

        let steering = (steering_target - b.position).normalized() * target_distance;

        let mut acceleration = Vec3::new(0.0, 0.0, 0.0);
        acceleration = acceleration + separation * self.separation_weight;
        acceleration = acceleration + alignment * self.alignment_weight;
        acceleration = acceleration + cohesion * self.cohesion_weight;
        acceleration = acceleration + steering * self.steering_weight;
        b.acceleration = acceleration.clamp_length(self.max_acceleration);
    }

    fn get_nearby_boids(&self, b: &Boid) -> Vec<NearbyBoid> {
        let mut result = Vec::new();
        result.reserve(self.boids.len());

        let mut voxel_pos = self.get_voxel_for_boid(b);
        voxel_pos.x -= 1.0;
        voxel_pos.y -= 1.0;
        voxel_pos.z -= 1.0;
        for _ in 0..3 {
            for _ in 0..3 {
                for _ in 0..3 {
                    self.check_voxel_for_boids(b, &mut result, &voxel_pos);
                    voxel_pos.z += 1.0;
                }
                voxel_pos.z -= 3.0;
                voxel_pos.y += 1.0;
            }
            voxel_pos.y -= 3.0;
            voxel_pos.x += 1.0;
        }
        result
    }

    fn check_voxel_for_boids<'b>(&self, b: &Boid, result: &mut Vec<NearbyBoid<'b>>, voxel_pos: &Vec3)
    where
        'a: 'b,
    {
        if let Some(boids) = self.voxel_cache.get(voxel_pos) {
            for &test in boids.iter() {
                let p1 = b.position;
                let p2 = test.position;
                let vec = p2 - p1;
                let distance = vec.length();

                let mut compare_value = 0.0;
                let l1 = vec.length();
                let l2 = b.velocity.length();
                if l1 != 0.0 && l2 != 0.0 {
                    compare_value = b.velocity.negative().dot_product(&vec) / (l1 * l2);
                }

                if std::ptr::eq(test, b as _) == false
                    && distance <= self.perception_radius
                    && (self.blindspot_angle_deg_compare_value > compare_value || b.velocity.length() == 0.0)
                {
                    result.push(NearbyBoid {
                        boid: test,
                        distance,
                        direction: vec,
                    });
                }
            }
        }
    }

    fn build_voxel_cache(&mut self) {
        self.voxel_cache.clear();
        for boid in self.boids.iter() {
            let pos = self.get_voxel_for_boid(boid);
            self.voxel_cache.entry(pos).or_insert(Vec::new()).push(boid);
        }
    }

    fn get_voxel_for_boid(&self, b: &Boid) -> Vec3 {
        let r = self.perception_radius.abs();
        let p = b.position;
        Vec3::new((p.x / r).floor() as f32, (p.y / r).floor() as f32, (p.z / r).floor() as f32)
    }

    fn transform_distance(&self, distance: f32, r#type: &DistanceType) -> f32 {
        match r#type {
            DistanceType::Linear => distance,
            DistanceType::InverseLinear => {
                if distance == 0.0 {
                    0.0
                } else {
                    1.0 / distance
                }
            }
            DistanceType::Quadratic => distance.powi(2),
            DistanceType::InverseQuadratic => {
                let quad = distance.powi(2);
                if quad == 0.0 {
                    0.0
                } else {
                    1.0 / quad
                }
            }
        }
    }
}

fn main() {
    // Example usage
    let mut boids = vec![Boid::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0))];
    let mut swarm = Swarm::new(&mut boids);
    swarm.update(0.1);
}