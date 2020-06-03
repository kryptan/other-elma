use cgmath::{dot, vec2, InnerSpace, Vector2};
use elma::lev::Polygon;
use elma::rec::EventType;
use std::mem;

pub const HEAD_RADIUS: f64 = 0.238;
pub const OBJECT_RADIUS: f64 = 0.4;

pub const WHEEL_RADIUS: f64 = 0.4;
pub const WHEEL_MASS: f64 = 10.0;
pub const WHEEL_ANGULAR_MASS: f64 = 0.32;

pub const BIKE_MASS: f64 = 200.0;
pub const BIKE_ANGULAR_MASS: f64 = 60.5;

pub const MAX_WHEEL_ANGULAR_VELOCITY: f64 = 110.0;
pub const ACCELERATION: f64 = 600.0;
pub const ROTATION_PERIOD: f64 = 0.4;
pub const ROTATION_SPEED_FAST: f64 = 12.0;
pub const ROTATION_SPEED_SLOW: f64 = 3.0;

pub const WHEEL_K: f64 = 10000.0;
pub const WHEEL_K0: f64 = 1000.0;

pub const HEAD_K: f64 = 50000.0;
pub const HEAD_K0: f64 = 3000.0;

pub const GRAVITY: f64 = 10.0;

pub const WHEEL_POSITIONS: [Vector2<f64>; 2] = [vec2(-0.85, -0.6), vec2(0.85, -0.6)];
pub const HEAD_POSITION: Vector2<f64> = vec2(0.0, 0.44);

pub const PI: f64 = 3.141592; // sic

const CELL_SIZE: f64 = 1.0;

struct Segment {
    /// Ends.
    a: Vector2<f64>,
    b: Vector2<f64>,

    /// Normalized AB.
    dir: Vector2<f64>,

    /// Segment length.
    ///
    /// `dir*length == ab`
    length: f64,
}

pub struct Segments {
    min: Vector2<f64>,
    segments: Vec<Segment>,
    width: usize,
    height: usize,
    table: Vec<Vec<u32>>,
}

impl Segment {
    fn collision(&self, pos: Vector2<f64>, r: f64) -> Option<Vector2<f64>> {
        let vector = pos - self.a;
        let fraction = dot(vector, self.dir);

        if fraction < 0.0 {
            if vector.magnitude() >= r {
                None
            } else {
                Some(self.a)
            }
        } else if fraction <= self.length {
            if self.dir.perp_dot(vector).abs() > r {
                None
            } else {
                Some(self.a + self.dir * fraction)
            }
        } else {
            if (pos - self.b).magnitude() >= r {
                None
            } else {
                Some(self.b)
            }
        }
    }
}

impl Segments {
    pub fn new(polygons: &[Polygon]) -> Segments {
        let mut min_x = f64::INFINITY;
        let mut max_x = -f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = -f64::INFINITY;

        let mut num_segments: i32 = 0;
        for polygon in polygons {
            if polygon.grass {
                continue;
            }

            num_segments -= 1;
            for vertex in &polygon.vertices {
                min_x = min_x.min(vertex.x);
                min_y = min_y.min(vertex.y);
                max_x = max_x.max(vertex.x);
                max_y = max_y.max(vertex.y);
                num_segments += 1;
            }
        }

        let width = ((max_x - min_x) / CELL_SIZE).ceil() as usize + 1;
        let height = ((max_y - min_y) / CELL_SIZE).ceil() as usize + 1;

        let mut segments = Vec::with_capacity(num_segments as usize);
        let mut table = vec![vec![]; width * height];

        for polygon in polygons {
            if polygon.grass {
                continue;
            }

            let mut prev = if let Some(vertex) = polygon.vertices.last() {
                vertex.clone()
            } else {
                continue;
            };

            for vertex in &polygon.vertices {
                let a = vec2(prev.x, prev.y);
                let b = vec2(vertex.x, vertex.y);
                let ab = b - a;
                let length = ab.magnitude();

                let index = segments.len();
                segments.push(Segment {
                    a,
                    b,
                    dir: ab / length,
                    length,
                });

                let mut y0 = ((a.y - min_y) / CELL_SIZE).floor();
                let mut y1 = ((b.y - min_y) / CELL_SIZE).floor();
                if y0 > y1 {
                    mem::swap(&mut y0, &mut y1);
                }

                let mut lower = a;
                let mut upper = b;
                if lower.y > upper.y {
                    mem::swap(&mut lower, &mut upper);
                }

                for y in (y0 as usize)..=(y1 as usize) {
                    // FIXME
                    let y0 = min_y + y as f64 * CELL_SIZE;
                    let y1 = min_y + (y as f64 + 1.0) * CELL_SIZE;

                    let lu = upper - lower;
                    let one = if lower.y >= y0 - 0.1 {
                        lower.x
                    } else {
                        lower.x + lu.x * (y0 - lower.y) / lu.y
                    };

                    let two = if upper.y <= y1 + 0.1 {
                        upper.x
                    } else {
                        lower.x + lu.x * (y1 - lower.y) / lu.y
                    };

                    //  if

                    /*    let x0 = a.x + ab.x * (y0 - a.y) / ab.y;
                    let x1 = a.x + ab.x * (y1 - a.y) / ab.y;
                    let x0 = ((x0 - min_x) / CELL_SIZE).floor();
                    let x1 = ((x1 - min_x) / CELL_SIZE).floor();*/

                    let mut x0 = ((one - min_x) / CELL_SIZE).floor();
                    let mut x1 = ((two - min_x) / CELL_SIZE).floor();
                    if x0 > x1 {
                        mem::swap(&mut x0, &mut x1);
                    }

                    for x in (x0 as usize)..=(x1 as usize) {
                        table[y * width + x].push(index as u32);
                    }
                }

                prev = vertex.clone();
            }
        }

        Segments {
            min: vec2(min_x, min_y),
            segments,
            width,
            height,
            table,
        }
    }

    fn cell(&self, pos: Vector2<f64>) -> &[u32] {
        if pos.x < self.min.x - CELL_SIZE || pos.y < self.min.y - CELL_SIZE {
            return &[];
        }

        let pos = (pos - self.min) / CELL_SIZE;
        let pos = vec2(pos.x.floor(), pos.y.floor());
        if pos.x > self.width as f64 || pos.y > self.height as f64 {
            return &[];
        }

        let x = pos.x as usize;
        let y = pos.y as usize;
        if x > self.width || y > self.height {
            return &[];
        }

        &self.table[y * self.width + x]
    }

    fn collision_test(
        &self,
        pos: Vector2<f64>,
        r: f64,
        collisions: &mut [Vector2<f64>; 2],
    ) -> usize {
        let corners = [
            pos + vec2(r, r),
            pos + vec2(r, -r),
            pos + vec2(-r, -r),
            pos + vec2(-r, r),
        ];

        let mut collision = false;
        for &corner in &corners {
            for &i in self.cell(corner) {
                let segment = &self.segments[i as usize];
                if let Some(point) = segment.collision(pos, r) {
                    if !collision {
                        collisions[0] = point;
                        collision = true;
                    } else {
                        collisions[1] = point;

                        if (collisions[0] - collisions[1]).magnitude() >= 0.1 {
                            return 2;
                        }

                        collisions[0] = (collisions[0] + collisions[1]) * 0.5;
                    }
                }
            }
        }

        collision as usize
    }
}

pub struct Moto {
    pub wheels: [Object; 2],
    pub bike: Object,
    pub head_position: Vector2<f64>,
    head_velocity: Vector2<f64>,
    braking: bool,
    pub direction: bool,
    rotation_left: bool,
    rotation_right: bool,
    eaten_apples: i32,
    brake_da: [f64; 2],
    rotation_time: f64,
    rotation_angular_velocity: f64,
    gravity: Vector2<f64>,
    time: f64,
}

pub struct Object {
    pub position: Vector2<f64>,
    velocity: Vector2<f64>,
    pub angular_position: f64,
    angular_velocity: f64,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Control {
    pub rotate_left: bool,
    pub rotate_right: bool,
    pub throttle: bool,
    pub brake: bool,
}

impl Moto {
    pub fn new(position: Vector2<f64>) -> Moto {
        let position = position - WHEEL_POSITIONS[0];

        Moto {
            time: 0.0,
            wheels: [
                Object {
                    position: position + WHEEL_POSITIONS[0],
                    velocity: vec2(0.0, 0.0),
                    angular_position: 0.0,
                    angular_velocity: 0.0,
                },
                Object {
                    position: position + WHEEL_POSITIONS[1],
                    velocity: vec2(0.0, 0.0),
                    angular_position: 0.0,
                    angular_velocity: 0.0,
                },
            ],
            bike: Object {
                position: position + vec2(0.0, 0.0),
                velocity: vec2(0.0, 0.0),
                angular_position: 0.0,
                angular_velocity: 0.0,
            },
            head_position: position + HEAD_POSITION,
            head_velocity: vec2(0.0, 0.0),
            direction: false,
            eaten_apples: 0,
            braking: false,
            brake_da: [0.0, 0.0],
            rotation_left: false,
            rotation_right: false,
            rotation_time: -100.0,
            rotation_angular_velocity: 0.0,
            gravity: vec2(0.0, -GRAVITY),
        }
    }

    pub fn advance(
        &mut self,
        control: Control,
        t: f64,
        segments: &Segments,
        events: &mut impl Events,
    ) {
        let dt = 0.00001;
        while self.time < t {
            self.time += dt;
            advance(self, control, self.time, dt, segments, events);
        }
    }
}

pub trait Events {
    fn event(&mut self, kind: EventType);
}

fn advance(
    moto: &mut Moto,
    control: Control,
    t: f64,
    dt: f64,
    segments: &Segments,
    events: &mut impl Events,
) {
    let mut rotate_left = false;
    let mut rotate_right = false;

    if moto.rotation_time + ROTATION_PERIOD < t {
        if control.rotate_right {
            moto.rotation_time = t;
            rotate_right = true;
            events.event(EventType::VoltRight);
        }
        if control.rotate_left {
            moto.rotation_time = t;
            rotate_left = true;
            events.event(EventType::VoltLeft);
        }
    }

    let mut dorzol_volume = 0.0;

    // Start braking.
    if control.brake && !moto.braking {
        moto.brake_da[0] = moto.wheels[0].angular_position - moto.bike.angular_position;
        moto.brake_da[1] = moto.wheels[1].angular_position - moto.bike.angular_position;
    }
    moto.braking = control.brake;

    let mut wheel_forces = [vec2(0.0, 0.0), vec2(0.0, 0.0)];
    let mut wheel_angular_forces = [0.0, 0.0];
    let mut bike_force = vec2(0.0, 0.0);
    let mut bike_angular_force = 0.0;
    let mut head_force = vec2(0.0, 0.0);

    if control.throttle {
        if moto.direction {
            if moto.wheels[0].angular_velocity > -MAX_WHEEL_ANGULAR_VELOCITY {
                wheel_angular_forces[0] = -ACCELERATION;
            }
        } else {
            if moto.wheels[1].angular_velocity < MAX_WHEEL_ANGULAR_VELOCITY {
                wheel_angular_forces[1] = ACCELERATION;
            }
        }
    }
    if control.brake {
        wheel_angular_forces[0] = (moto.wheels[0].angular_position
            - (moto.brake_da[0] + moto.bike.angular_position))
            * -1000.0
            - (moto.wheels[0].angular_velocity - moto.bike.angular_velocity) * 100.0;
        wheel_angular_forces[1] = (moto.wheels[1].angular_position
            - (moto.brake_da[1] + moto.bike.angular_position))
            * -1000.0
            - (moto.wheels[1].angular_velocity - moto.bike.angular_velocity) * 100.0;
    } else {
        fix_angle(&mut moto.wheels[0].angular_position);
        fix_angle(&mut moto.wheels[0].angular_position);
    }

    //  let bike_basis : Basis2<f64> = Rotation2::from_angle(Rad(moto.bike.angular_position));
    let x = angle_vector(moto.bike.angular_position);
    let y = rotate_90(x);

    let wheel_0_position = x * WHEEL_POSITIONS[0].x + y * WHEEL_POSITIONS[0].y;
    let wheel_1_position = x * WHEEL_POSITIONS[1].x + y * WHEEL_POSITIONS[1].y;

    compute_bike_wheel_forces(
        &moto.bike,
        &moto.wheels[0],
        wheel_0_position,
        &mut wheel_forces[0],
        wheel_angular_forces[0],
        &mut bike_force,
        &mut bike_angular_force,
        &mut dorzol_volume,
    );
    compute_bike_wheel_forces(
        &moto.bike,
        &moto.wheels[1],
        wheel_1_position,
        &mut wheel_forces[1],
        wheel_angular_forces[1],
        &mut bike_force,
        &mut bike_angular_force,
        &mut dorzol_volume,
    );

    compute_head_pos(moto, x, y);
    let head_position = HEAD_POSITION.y * y;
    compute_body_head_forces(moto, head_position, &mut head_force);

    let old_bike_angular_velocity = moto.bike.angular_velocity;

    if moto.rotation_right && t > ROTATION_PERIOD * 0.25 + moto.rotation_time {
        moto.bike.angular_velocity += ROTATION_SPEED_FAST;
        moto.bike.angular_velocity = moto
            .bike
            .angular_velocity
            .min(moto.rotation_angular_velocity);

        if moto.bike.angular_velocity > 0.0 {
            moto.bike.angular_velocity -= ROTATION_SPEED_SLOW;
            moto.bike.angular_velocity = moto.bike.angular_velocity.max(0.0);
        }

        moto.rotation_right = false;
    }
    if moto.rotation_left && t > ROTATION_PERIOD * 0.25 + moto.rotation_time {
        moto.bike.angular_velocity -= ROTATION_SPEED_FAST;
        moto.bike.angular_velocity = moto
            .bike
            .angular_velocity
            .max(moto.rotation_angular_velocity);

        if moto.bike.angular_velocity < 0.0 {
            moto.bike.angular_velocity += ROTATION_SPEED_SLOW;
            moto.bike.angular_velocity = moto.bike.angular_velocity.min(0.0);
        }

        moto.rotation_left = false;
    }

    if rotate_right {
        moto.rotation_angular_velocity = moto.bike.angular_velocity;
        moto.rotation_right = true;
        moto.bike.angular_velocity -= ROTATION_SPEED_FAST;
    }
    if rotate_left {
        moto.rotation_angular_velocity = moto.bike.angular_velocity;
        moto.rotation_left = true;
        moto.bike.angular_velocity += ROTATION_SPEED_FAST;
    }
    if rotate_left || rotate_right {
        let dw = moto.bike.angular_velocity - old_bike_angular_velocity;
        moto.head_velocity += scross(dw, moto.head_position - moto.bike.position);
    }

    moto.head_velocity += (head_force / BIKE_MASS + moto.gravity) * dt;
    moto.head_position += moto.head_velocity * dt;

    for i in 0..2 {
        let mut collisions = [vec2(0.0, 0.0); 2];
        let mut num_collisions =
            segments.collision_test(moto.wheels[i].position, WHEEL_RADIUS, &mut collisions);

        if num_collisions >= 1 {
            moto.wheels[i].push_out(collisions[0]);
        }
        if num_collisions >= 2 {
            moto.wheels[i].push_out(collisions[1]);
        }

        let v = moto.wheels[i].velocity;
        if num_collisions == 2 {
            let v_magnitude = v.magnitude();
            if v_magnitude > 1.0 {
                if !moto.wheels[i].test(
                    collisions[0],
                    collisions[1],
                    v,
                    moto.wheels[i].angular_velocity,
                ) {
                    num_collisions = 1;
                    collisions[0] = collisions[1];
                } else if !moto.wheels[i].test(
                    collisions[1],
                    collisions[0],
                    v,
                    moto.wheels[i].angular_velocity,
                ) {
                    num_collisions = 1;
                }
            }
            if v_magnitude < 1.0 {
                if !moto.wheels[i].test(
                    collisions[0],
                    collisions[1],
                    wheel_forces[i],
                    wheel_angular_forces[i],
                ) {
                    num_collisions = 1;
                    collisions[0] = collisions[1];
                } else if !moto.wheels[i].test(
                    collisions[1],
                    collisions[0],
                    wheel_forces[i],
                    wheel_angular_forces[i],
                ) {
                    num_collisions = 1;
                }
            }
        }

        if num_collisions == 2 && !moto.wheels[i].collision(collisions[1], wheel_forces[i], events)
        {
            num_collisions = 1;
        }
        if num_collisions >= 1 && !moto.wheels[i].collision(collisions[0], wheel_forces[i], events)
        {
            if num_collisions == 2 {
                collisions[0] = collisions[1];
            }
            num_collisions -= 1;
        }

        if num_collisions == 2 {
            moto.wheels[i].velocity = vec2(0.0, 0.0);
            moto.wheels[i].angular_velocity = 0.0;
            continue;
        }
        if num_collisions == 1 {
            let r = moto.wheels[i].position - collisions[0];
            let length = r.magnitude();
            let dir = rotate_90(r / length);
            moto.wheels[i].angular_velocity = dot(dir, v) / WHEEL_RADIUS
                + (dot(dir, wheel_forces[i]) * WHEEL_RADIUS + wheel_angular_forces[i])
                    / (length * length * WHEEL_MASS + WHEEL_ANGULAR_MASS)
                    * dt;
            moto.wheels[i].angular_position += moto.wheels[i].angular_velocity * dt;
            moto.wheels[i].velocity = (moto.wheels[i].angular_velocity * WHEEL_RADIUS) * dir;
            moto.wheels[i].position += moto.wheels[i].velocity * dt;
            continue;
        }

        moto.wheels[i].angular_velocity +=
            wheel_angular_forces[i] * (1.0 / WHEEL_ANGULAR_MASS) * dt;
        moto.wheels[i].angular_position += moto.wheels[i].angular_velocity * dt;
        moto.wheels[i].velocity += (wheel_forces[i] * (1.0 / WHEEL_MASS) + moto.gravity) * dt;
        moto.wheels[i].position += moto.wheels[i].velocity * dt;
    }

    moto.bike.angular_velocity += bike_angular_force * (1.0 / BIKE_ANGULAR_MASS) * dt;
    moto.bike.angular_position += moto.bike.angular_velocity * dt;
    moto.bike.velocity += (bike_force * (1.0 / BIKE_MASS) + moto.gravity) * dt;
    moto.bike.position += moto.bike.velocity * dt;
}

impl Object {
    fn push_out(&mut self, collision: Vector2<f64>) {
        let vector = self.position - collision;
        let dist = vector.magnitude();
        let r = WHEEL_RADIUS - 0.005;
        if dist < r {
            self.position += vector / dist * (r - dist);
        }
    }

    fn test(
        &self,
        collision0: Vector2<f64>,
        collision1: Vector2<f64>,
        v: Vector2<f64>,
        a: f64,
    ) -> bool {
        let r = self.position - collision0;
        r.perp_dot(collision0 - collision1) * (r.perp_dot(v) + a) >= 0.0
    }

    fn collision(
        &mut self,
        collision: Vector2<f64>,
        f: Vector2<f64>,
        events: &mut impl Events,
    ) -> bool {
        let x = (self.position - collision).normalize();
        let vx = dot(x, self.velocity);
        if vx > -0.01 && dot(x, f) > 0.0 {
            return false;
        }

        self.velocity -= vx * x;
        let vx = vx.abs();
        if vx > 1.5 {
            events.event(EventType::Ground((vx * 0.125).min(0.99) as f32));
        }

        true
    }
}

fn compute_bike_wheel_forces(
    bike: &Object,
    wheel: &Object,
    bn: Vector2<f64>,
    wheel_force: &mut Vector2<f64>,
    wheel_angular_force: f64,
    bike_force: &mut Vector2<f64>,
    bike_angular_force: &mut f64,
    dorzol_volume: &mut f64,
) {
    let kp = WHEEL_K;
    let kv = WHEEL_K0;

    let bw = wheel.position - bike.position;
    let mut dp = bn - bw;
    let dv = scross(bike.angular_velocity, bw) + bike.velocity - wheel.velocity;

    if dp.x.abs() <= 0.0001 && dp.y.abs() <= 0.0001 {
        dp = vec2(0.0, 0.0);
    }

    let f = kp * dp + kv * dv;
    *wheel_force = f - scross(wheel_angular_force, bw) / bw.magnitude2(); // FIXME: Fix singularity but preserve bump
    *bike_force -= *wheel_force;
    *bike_angular_force -= bw.perp_dot(f);

    compute_dorzol_volume(bike, dp, dv, dorzol_volume);
}

fn compute_head_pos(moto: &mut Moto, x: Vector2<f64>, y: Vector2<f64>) {
    let bh = moto.head_position - moto.bike.position;
    let mut bh = vec2(x.dot(bh), y.dot(bh));
    if moto.direction {
        bh.x = -bh.x;
    }

    let vec_453be0 = vec2(-0.23, 0.49).normalize();
    let normal_position = vec2(0.26, 0.48);

    let dot_product = (bh - vec2(-0.35, 0.13)).dot(vec_453be0);
    if dot_product < 0.0 {
        bh -= vec_453be0 * dot_product;
    }

    bh.x = bh.x.min(normal_position.x);
    bh.y = bh.y.min(normal_position.y);
    bh.x = bh.x.max(-0.5);

    if bh.x > 0.0 {
        let v33 = vec2(bh.x / normal_position.x, bh.y / normal_position.y).magnitude2();
        if v33 > 1.0 {
            bh *= 1.0 / v33.sqrt();
        }
    }

    if moto.direction {
        bh.x = -bh.x;
    }
    let bh = bh.x * x + bh.y * y;

    moto.head_position = moto.bike.position + bh;
}

fn compute_body_head_forces(moto: &Moto, bn: Vector2<f64>, head_force: &mut Vector2<f64>) {
    let kp = HEAD_K;
    let kv = HEAD_K0;

    let bh = moto.head_position - moto.bike.position;
    let dp = bn - bh;
    let dv = scross(moto.bike.angular_velocity, bh) + moto.bike.velocity - moto.head_velocity;

    *head_force += kp * dp + dv * kv;
}

fn compute_dorzol_volume(
    bike: &Object,
    dp: Vector2<f64>,
    dv: Vector2<f64>,
    dorzol_volume: &mut f64,
) {
    let x = angle_vector(bike.angular_position - PI / 2.0);
    let a = x.dot(dp);
    let b = x.dot(dv);

    if a > 0.0 && b > 0.0 && a * b > *dorzol_volume {
        *dorzol_volume = a * b;
    }
}

fn scross(a: f64, v: Vector2<f64>) -> Vector2<f64> {
    vec2(-v.y, v.x) * a
}

fn angle_vector(a: f64) -> Vector2<f64> {
    let (sin, cos) = a.sin_cos();
    vec2(cos, sin)
}

fn rotate_90(v: Vector2<f64>) -> Vector2<f64> {
    vec2(-v.y, v.x)
}

fn fix_angle(a: &mut f64) {
    if *a < -PI {
        *a += 2.0 * PI;
    } else if *a > PI {
        *a -= 2.0 * PI;
    }
}
