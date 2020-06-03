// https://github.com/Maxdamantus/recplay/blob/master/recRender.js
use crate::physics::WHEEL_RADIUS;
use crate::scene::Scene;
use crate::transform::Transform;
use crate::{physics, scene};
use cgmath::{vec2, InnerSpace, Vector2};
use std::f64::consts::PI;

pub fn render_moto(scene: &mut Scene, scene_moto: &scene::Moto, physics_moto: &physics::Moto) {
    for i in 0..2 {
        let transform = Transform::unit()
            .translate(physics_moto.wheels[i].position)
            .rotate(physics_moto.wheels[i].angular_position)
            .scale(2.0 * WHEEL_RADIUS)
            .translate(vec2(-0.5, 0.5));

        scene.set_image_pos(scene_moto.wheels[i], transform);
    }

    let mut moto_transform = Transform::unit()
        .translate(physics_moto.bike.position)
        .rotate(physics_moto.bike.angular_position);

    if physics_moto.direction {
        moto_transform = moto_transform.scale2(vec2(-1.0, 1.0));
    }

    let suspension_transform = moto_transform.scale(1.0 / 48.0);
    let mut wheels_pos = [vec2(0.0, 0.0); 2];
    for i in 0..2 {
        wheels_pos[i] = moto_transform
            .inverse()
            .transform(physics_moto.wheels[i].position);
    }
    if physics_moto.direction {
        wheels_pos.swap(0, 1);
    }

    // front suspension
    scene.set_image_pos(
        scene_moto.suspension1,
        suspension_transform.skew(2.0, 0.5, 5.0, 6.0, 48.0 * wheels_pos[0], vec2(-21.5, 17.0)),
    );

    // rear suspension
    scene.set_image_pos(
        scene_moto.suspension2,
        suspension_transform.skew(0.0, 0.5, 5.0, 6.0, vec2(9.0, -20.0), 48.0 * wheels_pos[1]),
    );

    let head_pos = moto_transform
        .inverse()
        .transform(physics_moto.head_position);
    let head_transform = moto_transform.translate(head_pos);

    // head
    scene.set_image_pos(
        scene_moto.head,
        head_transform
            .translate(vec2(-15.5 / 48.0, 42.0 / 48.0))
            .scale2(vec2(23.0 / 48.0, 23.0 / 48.0)),
    );

    // body
    scene.set_image_pos(
        scene_moto.body,
        head_transform
            .translate(vec2(17.0 / 48.0, -9.25 / 48.0))
            .rotate(-PI - 2.0 / 3.0)
            .scale2(vec2(100.0 / 48.0 / 3.0, 58.0 / 48.0 / 3.0)),
    );

    let bum = vec2(19.5 / 48.0, 0.0);
    let pedal = vec2(10.2, -65.0) / 48.0 / 3.0 - head_pos;
    LEG.render(
        scene,
        head_transform,
        bum,
        scene_moto.thigh,
        pedal,
        scene_moto.leg,
    );

    let shoulder = vec2(0.0, 17.5) / 48.0;
    let handle = vec2(-64.5, 59.6) / 48.0 / 3.0 - head_pos;
    // FIXME: animate
    ARM.render(
        scene,
        head_transform,
        shoulder,
        scene_moto.upper_arm,
        handle,
        scene_moto.forearm,
    );

    let bike_transform = moto_transform
        .translate(vec2(-43.0 / 48.0, 12.0 / 48.0))
        .rotate(PI * 0.197)
        .scale2(0.215815 / 48.0 * vec2(380.0, 301.0));

    scene.set_image_pos(scene_moto.bike, bike_transform);
}

struct Limb {
    inner: bool,
    parts: [LimbPart; 2],
}

struct LimbPart {
    length: f64,
    bx: f64,
    by: f64,
    br: f64,
    ih: f64,
}

const LEG: Limb = Limb {
    inner: true,
    parts: [
        LimbPart {
            length: 26.25 / 48.0,
            bx: 0.0,
            by: 0.6,
            br: 6.0 / 48.0,
            ih: 39.4 / 48.0 / 3.0,
        },
        LimbPart {
            length: 1.0 - 26.25 / 48.0,
            bx: 5.0 / 48.0 / 3.0,
            by: 0.45,
            br: 4.0 / 48.0,
            ih: 60.0 / 48.0 / 3.0,
        },
    ],
};

const ARM: Limb = Limb {
    inner: false,
    parts: [
        LimbPart {
            length: 0.3234,
            bx: 12.2 / 48.0 / 3.0,
            by: 0.5,
            br: 13.0 / 48.0 / 3.0,
            ih: -32.0 / 48.0 / 3.0,
        },
        LimbPart {
            length: 0.3444,
            bx: 3.0 / 48.0,
            by: 0.5,
            br: 13.2 / 48.0 / 3.0,
            ih: 22.8 / 48.0 / 3.0,
        },
    ],
};

impl Limb {
    fn render(
        &self,
        scene: &mut Scene,
        transform: Transform,
        p1: Vector2<f64>,
        first: usize,
        p2: Vector2<f64>,
        second: usize,
    ) {
        let dist = (p2 - p1).magnitude();
        let mut first_len = self.parts[0].length;
        let second_len = self.parts[1].length;

        let prod = (dist + first_len + second_len)
            * (dist - first_len + second_len)
            * (dist + first_len - second_len)
            * (-dist + first_len + second_len);
        let angle = (p2.y - p1.y).atan2(p2.x - p1.x);
        let mut jointangle = 0.0;
        if prod >= 0.0 && dist < first_len + second_len {
            // law of sines
            let circumr = dist * first_len * second_len / prod.sqrt();
            jointangle = (second_len / (2.0 * circumr)).asin();
        } else {
            first_len = first_len / (first_len + second_len) * dist;
        }

        if self.inner {
            jointangle *= -1.0;
        }

        let joint = p1 + first_len * vec2((angle + jointangle).cos(), (angle + jointangle).sin());

        scene.set_image_pos(
            first,
            transform.skew(
                self.parts[0].bx,
                self.parts[0].by,
                self.parts[0].br,
                self.parts[0].ih,
                joint,
                p1,
            ),
        );
        scene.set_image_pos(
            second,
            transform.skew(
                self.parts[1].bx,
                self.parts[1].by,
                self.parts[1].br,
                self.parts[1].ih,
                p2,
                joint,
            ),
        );
    }
}

/*


exports.renderer = function recRender(reader){
    var turnFrames = function(){
        var fc = reader.frameCount();
        var o = [], t = 0;
        for(var f = 0; f < fc; f++){
            var tmp = reader.turn(f) >> 1 & 1;
            if(tmp != t)
                o.push(f);
            t = tmp;
        }
        return o;
    }();

    var volts = [];
    void function(){
        var ec = reader.eventCount();
        var o = [];
        for(var e = 0; e < ec; e++)
            reader.event(e, function(time, info, type, a, b){
                var frame = Math.ceil(time/.01456);
                switch(type){
                    case 5: // turn
//						turnFrames.push(frame);
                        break;
                    case 6: // right volt
                        volts.push([frame, true]);
                        break;
                    case 7: // left volt
                        volts.push([frame, false]);
                        break;
                }
            });
            return o;
    }();

    function lastTurn(frame){
        for(var x = 0; x < turnFrames.length; x++)
            if(turnFrames[x] > frame)
                break;
        return x? turnFrames[x - 1] : -1;
    }

    function lastVolt(frame){
        for(var x = 0; x < volts.length; x++)
            if(volts[x][0] > frame)
                break;
        return x? volts[x - 1] : null;
    }

    function interpolate(fn){
        return function(n){
            var f = Math.floor(n), o = n - f, r = fn(f);
            if(o == 0)
                return r;
            return r + (fn(f + 1) - r)*o;
        };
    }

    function interpolateAng(fn, mod){
        return function(n){
            var f = Math.floor(n), o = n - f, r = fn(f);
            if(o == 0)
                return r;
            var rs = fn(f + 1), offs = 0;
            var diff1 = rs - r, diff2 = (rs + mod/2)%mod - (r + mod/2)%mod;
            var diff = Math.abs(diff1) < Math.abs(diff2)? diff1 : diff2;
            return r + diff*o;
        };
    }

    function turnScale(x){
        return -Math.cos(x*Math.PI);
    }

    var bikeXi = interpolate(reader.bikeX);
    var bikeYi = interpolate(reader.bikeY);
    var bikeRi = interpolateAng(reader.bikeR, 10000);
    var leftXi = interpolate(reader.leftX);
    var leftYi = interpolate(reader.leftY);
    var leftRi = interpolateAng(reader.leftR, 250);
    var rightXi = interpolate(reader.rightX);
    var rightYi = interpolate(reader.rightY);
    var rightRi = interpolateAng(reader.rightR, 250);
    var headXi = interpolate(reader.headX);
    var headYi = interpolate(reader.headY);

    function wheel(canv, lgr, wheelX, wheelY, wheelR){
        canv.save();
            canv.translate(wheelX, -wheelY);
            canv.rotate(-wheelR);
            canv.scale(38.4/48, 38.4/48);
            canv.translate(-0.5, -0.5);
            lgr.wheel.draw(canv);
        canv.restore();
    }

    // (x, y): top left in Elma coordinates
    // arguably a microoptimisation, but it doesn't produce any objects in the JS world
    function draw(canv, lgr, shirt, frame, x, y, scale){
        canv.save();
            canv.translate(/*Math.ceil*/(scale*(-x + bikeXi(frame))), /*Math.ceil*/(scale*(-y - bikeYi(frame))));
            canv.scale(scale, scale);
            canv.beginPath();

            var bikeR = bikeRi(frame)*Math.PI*2/10000;
            var turn = reader.turn(Math.floor(frame)) >> 1 & 1;
            var leftX = leftXi(frame)/1000;
            var leftY = leftYi(frame)/1000;
            var leftR = leftRi(frame)*Math.PI*2/250;
            var rightX = rightXi(frame)/1000;
            var rightY = rightYi(frame)/1000;
            var rightR = rightRi(frame)*Math.PI*2/250;
            var headX = headXi(frame)/1000;
            var headY = headYi(frame)/1000;
            var lastTurnF = lastTurn(frame);
            var lv = lastVolt(frame);

            var animlen = 28;
            var animpos = lv != null && frame - lv[0] < animlen? (frame - lv[0])/animlen : 0;
            var turnpos = lastTurnF >= 0 && lastTurnF + 24 > frame? (frame - lastTurnF)/24 : 0;

            var backX = !turn? rightX : leftX;
            var backY = !turn? rightY : leftY;
            var backR = !turn? rightR : leftR;
            var frontX = turn? rightX : leftX;
            var frontY = turn? rightY : leftY;
            var frontR = turn? rightR : leftR;

            if(turnpos == 0 || turnpos > 0.5)
                wheel(canv, lgr, backX, backY, backR);
            if(turnpos <= 0.5)
                wheel(canv, lgr, frontX, frontY, frontR);

            canv.save();
                canv.rotate(-bikeR);
                if(turn)
                    canv.scale(-1, 1);
                if(turnpos > 0)
                    canv.scale(turnScale(turnpos), 1);

                var wx, wy, a, r;
                var hbarsX = -21.5, hbarsY = -17;
                canv.save();
                    canv.scale(1/48, 1/48);

                    // front suspension
                    wx = turn? rightX : leftX;
                    wy = turn? -rightY : -leftY;
                    a = Math.atan2(wy, (turn? -1 : 1) * wx) + (turn? -1 : 1) * bikeR;
                    r = hypot(wx, wy);
                    skewimage(canv, lgr.susp1, 2, 0.5, 5, 6, 48*r * Math.cos(a), 48*r * Math.sin(a), hbarsX, hbarsY);

                    // rear suspension
                    wx = turn? leftX : rightX;
                    wy = turn? -leftY : -rightY;
                    a = Math.atan2(wy, (turn? -1 : 1) * wx) + (turn? -1 : 1) * bikeR;
                    r = hypot(wx, wy);
                    //skewimage(canv, lgr.susp2, 5, 0.5, 5, 6.5, 48*r*Math.cos(a), 48*r*Math.sin(a), 10, 20);
                    skewimage(canv, lgr.susp2, 0, 0.5, 5, 6, 9, 20, 48*r*Math.cos(a), 48*r*Math.sin(a));
                canv.restore();

                canv.save(); // bike
                    canv.translate(-43/48, -12/48);
                    canv.rotate(-Math.PI*0.197);
                    canv.scale(0.215815*380/48, 0.215815*301/48);
                    lgr.bike.draw(canv);
                canv.restore();

                canv.save(); // kuski
                    r = hypot(headX, headY);
                    a = Math.atan2(-headY, turn? -headX : headX) + (turn? -bikeR : bikeR);
                    wx = r*Math.cos(a);
                    wy = r*Math.sin(a);
                    canv.translate(wx, wy);

                    canv.save(); // head
                        canv.translate(-15.5/48, -42/48);
                        canv.scale(23/48, 23/48);
                        lgr.head.draw(canv);
                    canv.restore();

                    var bumx = 19.5/48, bumy = 0;
                    var pedalx = -wx + 10.2/48/3, pedaly = -wy + 65/48/3;
                    legLimb(canv, lgr.q1thigh, bumx, bumy, lgr.q1leg, pedalx, pedaly);

                    canv.save(); // torso
                        canv.translate(17/48, 9.25/48);
                        canv.rotate(Math.PI + 2/3);
                        canv.scale(100/48/3, 58/48/3);
                        if(shirt && shirt.touch()){
                            // assumes shirts are rotated as on EOL site
                            canv.translate(0.5, 0.5);
                            canv.rotate(Math.PI/2);
                            canv.translate(-0.5, -0.5);
                            shirt.draw(canv);
                        }else
                            lgr.q1body.draw(canv);
                    canv.restore();

                    var shoulderx = 0/48, shouldery = -17.5/48;
                    var handlex = -wx - 64.5/48/3, handley = -wy - 59.6/48/3;
                    var handx = handlex, handy = handley;

                    var animx = shoulderx, animy = shouldery;
                    if(animpos > 0){
                        var dangle, ascale;
                        if(lv[1] == turn){
                            if(animpos >= 0.25)
                                animpos = 0.25 - 0.25*(animpos - 0.25)/0.75;
                            dangle = 10.8*animpos;
                            ascale = 1 - 1.2*animpos;
                        }else{
                            if(animpos >= 0.2)
                                animpos = 0.2 - 0.2*(animpos - 0.2)/0.8;
                            dangle = -8*animpos;
                            ascale = 1 + 0.75*animpos;
                        }
                        var at = Math.atan2(handley - animy, handlex - animx) + dangle;
                        var dist = ascale*hypot(handley - animy, handlex - animx);
                        handx = animx + dist*Math.cos(at);
                        handy = animy + dist*Math.sin(at);
                    }

                    armLimb(canv, lgr.q1up_arm, shoulderx, shouldery, lgr.q1forarm, handx, handy);
                canv.restore();
            canv.restore();

            if(turnpos != 0 && turnpos <= 0.5)
                wheel(canv, lgr, backX, backY, backR);
            if(turnpos > 0.5)
                wheel(canv, lgr, frontX, frontY, frontR);
        canv.restore();
    }

    return {
        draw: draw,
        bikeXi: bikeXi,
        bikeYi: bikeYi
    };
};

*/
