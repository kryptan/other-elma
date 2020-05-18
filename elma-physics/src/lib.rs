extern crate cgmath;

use cgmath::{Vector2, Basis2, Rotation2, Rad, InnerSpace, vec2};

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

pub const WHEEL_POSITIONS: [Vector2<f64>; 2] = [Vector2{ x: -0.85, y: -0.6 }, Vector2{ x: 0.85, y: -0.6 }];
pub const HEAD_POSITION: Vector2<f64> = Vector2{ x: 0.0, y: 0.44 };

pub const PI: f64 = 3.141592; // sic

struct Segment {
    /// One of the segment's points.
    a: Vector2<f64>,

    /// Vector from point A to point B.
    ab: Vector2<f64>,

    /// Normalized AB.
    dir: Vector2<f64>,

    /// Segment length.
    ///
    /// `dir*length == ab`
    length : f64,
}

pub struct Moto {
    wheels: [Object; 2],
    bike: Object,
    head_position: Vector2<f64>,
    head_velocity: Vector2<f64>,
    braking: bool,
    direction: bool,
    rotation_left: bool,
    rotation_right: bool,
    eaten_apples: i32,
    brake_da: [f64; 2],
    rotation_time: f64,
    rotation_angular_velocity: f64,
    gravity: Vector2<f64>,
}

struct Object {
    position: Vector2<f64>,
    velocity: Vector2<f64>,
    angular_position: f64,
    angular_velocity: f64,
}

pub struct Level {
    num_apples: i32,
    apples: Vec<Vector2<f64>>,
    finishes: Vec<Vector2<f64>>,
}

#[derive(Debug, Copy, Clone)]
pub struct Control {
    pub rotate_left: bool,
    pub rotate_right: bool,
    pub throttle: bool,
    pub brake: bool,
}

pub enum Event {
    HitObject {
        index: i32
    },
    UtodesSound {
        volume: f64,
    },
    AppleSound,
    RotateLeft,
    RotateRight,
}

pub trait EventRecorder {
    fn add_event(&mut self, event: Event);
}

impl Moto {
    fn new() -> Moto {
        Moto {
            wheels: [
                Object {
                    position: WHEEL_POSITIONS[0],
                    velocity: vec2(0.0, 0.0),
                    angular_position: 0.0,
                    angular_velocity: 0.0,
                },
                Object {
                    position: WHEEL_POSITIONS[1],
                    velocity: vec2(0.0, 0.0),
                    angular_position: 0.0,
                    angular_velocity: 0.0,
                },
            ],
            bike: Object {
                position: vec2(0.0, 0.0),
                velocity: vec2(0.0, 0.0),
                angular_position: 0.0,
                angular_velocity: 0.0,
            },
            head_position: HEAD_POSITION,
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
}

pub fn advance<E: EventRecorder>(moto : &mut Moto, control: Control, level: &Level, t: f64, dt: f64, event_recorder: &mut E) {
    let mut rotate_left = false;
    let mut rotate_right = false;

    if moto.rotation_time + ROTATION_PERIOD < t {
        if control.rotate_right {
            moto.rotation_time = t;
            rotate_right = true;
            event_recorder.add_event(Event::RotateRight);
        }
        if control.rotate_left {
            moto.rotation_time = t;
            rotate_left = true;
            event_recorder.add_event(Event::RotateLeft);
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
        wheel_angular_forces[0] = (moto.wheels[0].angular_position - (moto.brake_da[0] + moto.bike.angular_position)) * -1000.0 - (moto.wheels[0].angular_velocity - moto.bike.angular_velocity) * 100.0;
        wheel_angular_forces[1] = (moto.wheels[1].angular_position - (moto.brake_da[1] + moto.bike.angular_position)) * -1000.0 - (moto.wheels[1].angular_velocity - moto.bike.angular_velocity) * 100.0;
    } else {
        fix_angle(&mut moto.wheels[0].angular_position);
        fix_angle(&mut moto.wheels[0].angular_position);
    }

    //  let bike_basis : Basis2<f64> = Rotation2::from_angle(Rad(moto.bike.angular_position));
    let x = angle_vector(moto.bike.angular_position);
    let y = rotate_90(x);

    let wheel_0_position = moto.wheels[0].position - moto.bike.position;
    let wheel_1_position = moto.wheels[1].position - moto.bike.position;

    let wheel_0_position = x*wheel_0_position.x + y*wheel_0_position.y;
    let wheel_1_position = x*wheel_1_position.x + y*wheel_1_position.y;

    compute_bike_wheel_forces(&moto.bike, &moto.wheels[0], wheel_0_position, &mut wheel_forces[0], wheel_angular_forces[0], &mut bike_force, &mut bike_angular_force, &mut dorzol_volume);
    compute_bike_wheel_forces(&moto.bike, &moto.wheels[1], wheel_1_position, &mut wheel_forces[1], wheel_angular_forces[1], &mut bike_force, &mut bike_angular_force, &mut dorzol_volume);

    compute_head_pos(moto, x, y);
    let head_position = HEAD_POSITION.y*y;
    compute_body_head_forces(moto, head_position, &mut head_force);

    let old_bike_angular_velocity = moto.bike.angular_velocity;

    if moto.rotation_right && t > ROTATION_PERIOD*0.25 + moto.rotation_time {
        moto.bike.angular_velocity += ROTATION_SPEED_FAST;
        moto.bike.angular_velocity = min(moto.bike.angular_velocity, moto.rotation_angular_velocity);

        if moto.bike.angular_velocity > 0.0 {
            moto.bike.angular_velocity -= ROTATION_SPEED_SLOW;
            moto.bike.angular_velocity = max(moto.bike.angular_velocity, 0.0);
        }

        moto.rotation_right = false;
    }
    if moto.rotation_left && t > ROTATION_PERIOD*0.25 + moto.rotation_time {
        moto.bike.angular_velocity -= ROTATION_SPEED_FAST;
        moto.bike.angular_velocity = max(moto.bike.angular_velocity, moto.rotation_angular_velocity);

        if moto.bike.angular_velocity < 0.0 {
            moto.bike.angular_velocity += ROTATION_SPEED_SLOW;
            moto.bike.angular_velocity = min(moto.bike.angular_velocity, 0.0);
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

    moto.head_position += (head_force/BIKE_MASS + moto.gravity)*dt;
    moto.head_position += moto.head_velocity*dt;

    // FIXME: add collision detection for wheels here

    for i in 0..2 {
        moto.wheels[i].angular_velocity += wheel_angular_forces[i]*(1.0/WHEEL_ANGULAR_MASS)*dt;
        moto.wheels[i].angular_position += moto.wheels[i].angular_velocity*dt;
        moto.wheels[i].velocity += wheel_forces[i]*(1.0/WHEEL_MASS)*dt;
        moto.wheels[i].position += moto.wheels[i].velocity*dt;
    }

    moto.bike.angular_velocity += bike_angular_force*(1.0/BIKE_ANGULAR_MASS)*dt;
    moto.bike.angular_position += moto.bike.angular_velocity*dt;
    moto.bike.velocity += bike_force*(1.0/BIKE_MASS)*dt;
    moto.bike.position += moto.bike.velocity*dt;
}

fn compute_bike_wheel_forces(bike: &Object, wheel: &Object, bn: Vector2<f64>, wheel_force: &mut Vector2<f64>, wheel_angular_force: f64, bike_force: &mut Vector2<f64>, bike_angular_force: &mut f64, dorzol_volume: &mut f64) {
    let kp = WHEEL_K;
    let kv = WHEEL_K0;

    let bw = wheel.position - bike.position;
    let mut dp = bn - bw;
    let dv = scross(bike.angular_velocity, bw) + bike.velocity - wheel.velocity;

    if dp.x.abs() <= 0.0001 && dp.y.abs() <= 0.0001 {
        dp = vec2(0.0, 0.0);
    }

    let f = kp*dp + kv*dv;
    *wheel_force = f - scross(wheel_angular_force, bw)/bw.magnitude2();
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

    let dot_product = (bh -  vec2(-0.35, 0.13)).dot(vec_453be0);
    if dot_product < 0.0 {
        bh -= vec_453be0*dot_product;
    }

    bh.x = min(bh.x, normal_position.x);
    bh.y = min(bh.y, normal_position.y);
    bh.x = max(bh.x, -0.5);

    if bh.x > 0.0 && bh.y > 0.0 {
        let k = normal_position.y/normal_position.x;
        let v32 = k*k*bh.x*bh.x + bh.y*bh.y;
        if v32 > normal_position.y*normal_position.x {
            bh *= normal_position.y/v32.sqrt();
        }
    }

    if moto.direction {
        bh.x = -bh.x;
    }
    let bh = bh.x*x + bh.y*y;

    moto.head_position = moto.bike.position + bh;
}

fn compute_body_head_forces(moto: &Moto, bn: Vector2<f64>, head_force: &mut Vector2<f64>) {
    let kp = HEAD_K;
    let kv = HEAD_K0;

    let bh = moto.head_position - moto.bike.position;
    let dp = bn - bh;
    let dv = scross(moto.bike.angular_velocity, bh) + moto.bike.velocity - moto.head_velocity;

    *head_force += kp*dp + dv*kv;
}

fn compute_dorzol_volume(bike: &Object, dp: Vector2<f64>, dv: Vector2<f64>, dorzol_volume: &mut f64) {
    let x = angle_vector(bike.angular_position - PI/2.0);
    let a = x.dot(dp);
    let b = x.dot(dv);

    if a > 0.0 && b > 0.0 && a*b > *dorzol_volume {
        *dorzol_volume = a*b;
    }
}

fn scross(a: f64, v: Vector2<f64>) -> Vector2<f64> {
    vec2(-v.y, v.x)*a
}

fn angle_vector(a: f64) -> Vector2<f64> {
    vec2(a.cos(), a.sin())
}

fn rotate_90(v: Vector2<f64>) -> Vector2<f64> {
    vec2(-v.y, v.x)
}

fn fix_angle(a: &mut f64) {
    if *a < -PI {
        *a += 2.0*PI;
    } else if *a > PI {
        *a -= 2.0*PI;
    }
}

fn min(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}

fn max(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}
