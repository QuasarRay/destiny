// Copyright © 2014-2023 CCP ehf. MIT license, retained from original Destiny.
//! Original Ballpark.cpp linear integration and GotoThrust, with the original
//! expression grouping and Vector3d's reciprocal-then-multiply division.
//! Platform C math and Kani builtin-model correspondence remain required.

use crate::{OriginalParkBall, Vec3};

#[must_use]
pub fn original_integrate_tick(
    position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    mass: f64,
    friction: f64,
    time_factor: f64,
) -> (Vec3, Vec3) {
    // This is the source's t == dt == 1 branch, so it uses the cached factor.
    integrate_formula(
        position,
        velocity,
        acceleration,
        mass,
        friction,
        time_factor,
        1.0,
    )
}

#[must_use]
pub fn original_integrate_duration(
    position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    mass: f64,
    friction: f64,
    time_factor: f64,
    dt: f64,
    duration: f64,
) -> (Vec3, Vec3) {
    if duration == 0.0 {
        return (position, velocity);
    }
    let factor = if duration != dt {
        (-friction / mass * duration).exp()
    } else {
        time_factor
    };
    integrate_formula(
        position,
        velocity,
        acceleration,
        mass,
        friction,
        factor,
        duration,
    )
}

fn integrate_formula(
    p: Vec3,
    v: Vec3,
    a: Vec3,
    m: f64,
    k: f64,
    factor: f64,
    t: f64,
) -> (Vec3, Vec3) {
    let k2 = k * k;
    let ook2 = 1.0 / k2;
    let ook = 1.0 / k;
    let position = if k < 1e-10 * m * t {
        a.scale(-k * t)
            .add(a.scale(t).add(v.scale(k).scale(1.0 / m).scale(t)).scale(k))
            .scale(m)
            .add(p.scale(k2))
            .scale(ook2)
    } else {
        a.scale(m)
            .scale(factor - 1.0)
            .add(a.scale(t).add(v).sub(v.scale(factor)).scale(k))
            .scale(m)
            .add(p.scale(k2))
            .scale(ook2)
    };
    let velocity = a
        .scale(m)
        .sub(a.scale(m).sub(v.scale(k)).scale(factor))
        .scale(ook);
    (position, velocity)
}

#[must_use]
pub fn original_goto_thrust(
    ball: &OriginalParkBall,
    target: Vec3,
    friction: f64,
    dt: f64,
    missile: bool,
) -> Vec3 {
    let mut a = target.sub(ball.position);
    let length2 = a.norm_squared();
    let dist = f64::from(ball.speed_fraction) * f64::from(ball.max_velocity) * dt;
    let max_thrust = friction * f64::from(ball.speed_fraction) * f64::from(ball.max_velocity)
        / (ball.mass * f64::from(ball.agility));
    if !missile && length2 < dist * dist {
        a = a.normalize();
        let coefficient = length2 / (dist * dist);
        a.scale(max_thrust * coefficient * coefficient)
    } else {
        a.normalize().scale(max_thrust)
    }
}

#[must_use]
pub fn original_stop_velocity(mut velocity: Vec3) -> Vec3 {
    // The disabled dynamical-orientation branch damps Y before integration.
    velocity.y = (velocity.y - 0.07 * velocity.y) * 0.9345794392523364485981308411215;
    velocity
}

#[must_use]
pub fn original_follow_thrust(ball:&mut OriginalParkBall,other:&OriginalParkBall) -> Vec3 {
    // Ballpark.cpp:1073. Keep-at-range uses the other body's current position,
    // not a prediction. The distinct missile routine has a different target.
    let delta=ball.position.sub(other.position); let distance=delta.length();
    let radius=f64::from(ball.follow_range)+f64::from(ball.radius)+f64::from(other.radius);
    let target=if distance==0.0 {
        other.position.add(Vec3 { x:1.0,y:0.0,z:0.0 }.scale(radius))
    } else { other.position.add(delta.scale(radius).scale(1.0/distance)) };
    ball.goto=target;
    original_goto_thrust(ball,target,1_000_000.0,1.0,ball.mode==crate::OriginalMode::Missile)
}

pub fn original_missile_thrust(ball:&mut OriginalParkBall,other:&OriginalParkBall,current_time:i64)
    -> Result<Vec3,crate::OriginalParkError> {
    let Some(elapsed)=current_time.checked_sub(ball.effect_stamp).and_then(|n|n.checked_mul(1000)) else {
        return Err(crate::OriginalParkError::OutsideDeclaredScope);
    };
    if elapsed<=800 { return Ok(original_goto_thrust(ball,ball.goto,1_000_000.0,1.0,false)); }
    let delta=ball.position.sub(other.position);let distance=delta.length();
    let intercept_time=distance/f64::from(ball.max_velocity);
    let mut target=other.position.add(other.velocity.scale(intercept_time));
    let radius=f64::from(ball.follow_range)+f64::from(ball.radius)+f64::from(other.radius);
    if distance==0.0 { target=other.position.add(Vec3{x:1.0,y:0.0,z:0.0}.scale(radius)); }
    else { target=target.add(delta.scale(radius).scale(1.0/distance)); }
    ball.goto=target;
    Ok(original_goto_thrust(ball,target,1_000_000.0,1.0,true))
}

fn cross(left:Vec3,right:Vec3) -> Vec3 {
    Vec3 { x:left.y*right.z-left.z*right.y,y:left.z*right.x-left.x*right.z,
        z:left.x*right.y-left.y*right.x }
}

fn truncate_seven(value:f64) -> Result<f64,crate::OriginalParkError> {
    let scaled=value*10_000_000.0;
    if !scaled.is_finite() || scaled < -9_223_372_036_854_775_808.0
        || scaled >= 9_223_372_036_854_775_808.0 {
        return Err(crate::OriginalParkError::OutsideDeclaredScope);
    }
    Ok((scaled as i64) as f64/10_000_000.0)
}

fn default_orbital_normal(id:i64,current_time:i64) -> Result<Vec3,crate::OriginalParkError> {
    let phi1=(current_time as f64)*0.001;
    let phi2=((id&0xffff) as f64)+(current_time as f64)*0.001;
    Ok(Vec3 { x:truncate_seven(phi1.cos()*phi2.cos())?,y:truncate_seven(phi2.sin())?,
        z:truncate_seven(phi1.sin()*phi2.cos())? })
}

pub fn original_old_orbit_thrust(ball:&mut OriginalParkBall,other:&OriginalParkBall,current_time:i64)
    -> Result<Vec3,crate::OriginalParkError> {
    let cruise=f64::from(ball.speed_fraction)*f64::from(ball.max_velocity);
    let maximum=1_000_000.0*cruise/(ball.mass*f64::from(ball.agility));
    let radius=f64::from(ball.follow_range)+f64::from(ball.radius)+f64::from(other.radius);
    let mut to=other.position.sub(ball.position); let distance=to.length(); to=to.normalize();
    let radial=cross(default_orbital_normal(ball.id,current_time)?,to).normalize();
    let component=distance*distance-radius*radius;
    if component>=0.0 {
        let radial_component=radius*component.sqrt()/distance;
        to=to.scale(component/distance).add(radial.scale(radial_component)).normalize();
    }
    let radial_factor=truncate_seven((-(radius-distance)*(radius-distance)/40_000.0).exp())?;
    let angle=to.scale(-1.0).dot(radial);
    let mut transverse=1.0+radial_factor*radial_factor*(angle*angle-1.0);
    transverse=if transverse>0.0 { radial_factor*angle+transverse.sqrt() } else { radial_factor*angle };
    transverse*=if distance-radius>=0.0 { 1.0 } else { -1.0 };
    let acceleration=radial.scale(radial_factor).add(to.scale(transverse)).scale(maximum);
    ball.goto=ball.position.add(acceleration.scale(10.0*149_597_870_700.0));
    Ok(acceleration)
}

fn orbit_goto_follow(ball:&OriginalParkBall,target:Vec3,target_velocity:Vec3) -> Vec3 {
    let speed=target_velocity.length();
    let friction_over_mass=1_000_000.0/(ball.mass*f64::from(ball.agility));
    let max_acceleration=friction_over_mass*f64::from(ball.speed_fraction)*f64::from(ball.max_velocity);
    if speed<5e-2*f64::from(ball.max_velocity)*f64::from(ball.speed_fraction) {
        if max_acceleration<=0.0 { return Vec3::ZERO; }
        let mut a=target.sub(ball.position).scale(friction_over_mass).sub(ball.velocity);
        let time=a.length()/max_acceleration;
        if time>1.0 { a=a.scale(1.0/time); }
        if time<0.1 { return Vec3::ZERO; }
        return a;
    }
    if max_acceleration<=0.0 { return Vec3::ZERO; }
    let mut acceleration=target.sub(ball.position).scale(friction_over_mass)
        .sub(ball.velocity.scale(1.0-ball.time_factor)).scale(friction_over_mass)
        .scale(1.0/(friction_over_mass-(1.0-ball.time_factor)));
    let velocity_acceleration=target_velocity.sub(ball.velocity.scale(ball.time_factor))
        .scale(friction_over_mass).scale(1.0/(1.0-ball.time_factor));
    acceleration=acceleration.add(velocity_acceleration).scale(1.0/2.0);
    let length2=acceleration.norm_squared();
    if length2>max_acceleration*max_acceleration { acceleration=acceleration.scale(max_acceleration/length2.sqrt()); }
    acceleration
}

pub fn original_new_orbit_thrust(ball:&mut OriginalParkBall,other:&OriginalParkBall,current_time:i64)
    -> Result<Vec3,crate::OriginalParkError> {
    let cruise=f64::from(ball.speed_fraction)*f64::from(ball.max_velocity);
    let tau=(ball.mass*f64::from(ball.agility))/1_000_000.0; let max_thrust=cruise/tau;
    let radius=f64::from(ball.follow_range)+f64::from(ball.radius)+f64::from(other.radius);
    let mut to=other.position.sub(ball.position).add(other.velocity);
    let mut distance=to.length();
    to=if distance>0.0 { to.scale(1.0/distance) } else { Vec3 { x:0.0,y:0.0,z:1.0 } };
    let normal=cross(to,ball.velocity);
    let length2=normal.norm_squared();
    let orbital=if length2>1e-12 { normal.scale(1.0/length2.sqrt()) }
        else { default_orbital_normal(ball.id,current_time)? };
    let radial=cross(orbital,to).normalize();
    let speed=ball.velocity.length(); let max_speed=if cruise<speed { speed } else { cruise };
    let mut angle=0.1*(radius-distance)/max_speed;
    if angle < -1.0 { angle=-1.0; } if angle>1.0 { angle=1.0; }
    if distance<0.1 { distance=0.1; }
    let max_velocity=(distance*max_thrust).sqrt();
    let estimated_velocity=if max_velocity<max_speed { max_velocity } else { max_speed };
    let estimated_distance=estimated_velocity;
    let goto_point; let mut goto_velocity;
    if angle.abs()<1e-10 {
        let desired=estimated_distance/distance;
        goto_point=to.scale(distance*(1.0-desired.cos())).add(radial.scale(distance*desired.sin()));
        goto_velocity=radial.scale(desired.cos()*estimated_velocity).add(to.scale(desired.sin()*estimated_velocity));
    } else {
        let factor=estimated_distance*angle/distance/(angle*angle+1.0).sqrt();
        let desired=if factor < -0.999 { 0.001_f64.ln()/angle }
            else if factor.abs()>1e-10 { (1.0+factor).ln()/angle } else { factor/angle };
        let exponential=distance*(angle*desired).exp();
        goto_point=to.scale(distance-exponential*desired.cos()).add(radial.scale(exponential*desired.sin()));
        goto_velocity=to.scale(-angle*desired.cos()+desired.sin())
            .add(radial.scale(angle*desired.sin()+desired.cos())).normalize().scale(estimated_velocity);
    }
    ball.goto=ball.position.add(goto_point); goto_velocity=goto_velocity.add(other.velocity);
    Ok(orbit_goto_follow(ball,ball.goto,goto_velocity))
}
