// Copyright © 2015 CCP ehf. MIT license, retained from original Destiny.
//! Ball.cpp alignment and Ballpark.cpp SetupWarpConstants/WarpDistance.
//! The concrete evolution callers use t in seconds and interpolating=false.
//! Platform log/exp, compiler and native event correspondence remain required.

use crate::{OriginalMode,OriginalParkBall,Vec3};

#[must_use]
pub fn original_aligned_for_warp(ball:&OriginalParkBall) -> bool {
    if ball.mode!=OriginalMode::Warp || ball.effect_stamp>=0 { return false; }
    let direction=ball.goto.sub(ball.position).normalize();
    let velocity=ball.velocity.normalize();
    ((1.0-velocity.dot(direction)).abs()<0.01
        && ball.velocity.norm_squared()>0.5625*f64::from(ball.max_velocity)*f64::from(ball.max_velocity))
        || ball.effect_stamp.unsigned_abs()>180
}

/// Returns position, velocity, and whether the source performs Stop(ball).
#[must_use]
pub fn original_warp_distance(ball:&OriginalParkBall,time:f64) -> (Vec3,Vec3,bool) {
    let factor=ball.owner_id as f64;
    let distance=ball.last_collision;
    let mut speed=factor*0.001*149_597_870_700.0;
    let acceleration=factor*(1.0/1000.0);
    let initial_deceleration=factor*(1.0/3000.0);
    let deceleration=if 2.0<initial_deceleration { 2.0 } else { initial_deceleration };
    let distance_bound=(distance+1.0)*acceleration*deceleration/(acceleration+deceleration);
    if distance_bound<speed { speed=distance_bound; }
    let acceleration_duration=(speed/acceleration).ln()/acceleration;
    let cruise_duration=distance/speed-1.0/acceleration-1.0/deceleration+1.0/speed;
    let deceleration_duration=(speed/deceleration).ln()/deceleration;
    let acceleration_distance=speed/acceleration;
    let cruise_distance=speed*cruise_duration;
    let deceleration_distance=speed/deceleration-1.0;
    let direction=ball.goto.sub(ball.position).normalize();
    let total=acceleration_distance+cruise_distance+deceleration_distance;
    if time<acceleration_duration {
        let mut current_speed=acceleration*(acceleration*time).exp();
        let old_speed=ball.velocity.length();
        if old_speed>current_speed { current_speed=old_speed; }
        let travelled=(acceleration*time).exp();
        (ball.goto.sub(direction.scale(total-travelled)),direction.scale(current_speed),false)
    } else if time-acceleration_duration<cruise_duration {
        let travelled=speed*(time-acceleration_duration)+acceleration_distance;
        (ball.goto.sub(direction.scale(total-travelled)),direction.scale(speed),false)
    } else if time-cruise_duration-acceleration_duration<deceleration_duration+1.0 {
        let exponential=(-deceleration*(time-cruise_duration-acceleration_duration)).exp();
        let current_speed=speed*exponential;
        let travelled=speed/deceleration-(speed/deceleration)*exponential+acceleration_distance+cruise_distance;
        let half_maximum=f64::from(ball.max_velocity)/2.0;
        let stop=current_speed<(if 100.0<half_maximum { 100.0 } else { half_maximum });
        (ball.goto.sub(direction.scale(total-travelled)),direction.scale(current_speed),stop)
    } else { (ball.goto,Vec3::ZERO,true) }
}
