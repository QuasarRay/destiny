//! Ballpark.cpp5156..5221 and Thunkers.cpp2044..2075. Preserve the
//! double expression order and the two in/out parameter values on rejection.
use destiny_original_spec::Vec3;
#[derive(Clone, Copy)]
pub(crate) struct Ball {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
}

pub(crate) fn rejected(optimal: f64, tracking: f64) -> [f64; 3] {
    #[cfg(kani)]
    kani::cover!(true, "actual GetAccuracy rejection fields executed");
    [0.0, optimal, tracking]
}
pub(crate) fn values(
    same: bool,
    src: Option<Ball>,
    dst: Option<Ball>,
    parameters: [f64; 5],
) -> [f64; 3] {
    #[cfg(kani)]
    kani::cover!(true, "actual GetAccuracy production kernel executed");
    let [optimal, falloff, tracking, signature, optimal_signature] = parameters;
    let rejected = rejected(optimal, tracking);
    if same {
        return rejected;
    }
    let (Some(src), Some(dst)) = (src, dst) else {
        return rejected;
    };
    if optimal < 0.0 || tracking == 0.0 || signature == 0.0 || falloff == 0.0 {
        return rejected;
    }
    let radial = dst.position.sub(src.position);
    let distance = radial.length();
    let mut short = (distance - f64::from(dst.radius)) - f64::from(src.radius);
    if short < 0.0 {
        short = 0.0;
    }
    if distance == 0.0 {
        return rejected;
    }
    let combined = dst.velocity.sub(src.velocity);
    let radial_speed = combined.dot(radial) / distance;
    let speed2 = (combined.norm_squared() - radial_speed * radial_speed) / (distance * distance);
    let mut exponent = optimal_signature / (tracking * signature);
    exponent *= -exponent * speed2;
    if short > optimal {
        exponent -= (short - optimal) * (short - optimal) / (falloff * falloff);
    }
    [2.0f64.powf(exponent), short, speed2.sqrt()]
}
