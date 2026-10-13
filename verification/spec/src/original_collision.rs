// Copyright © 2015-2023 CCP ehf. MIT license, retained from original Destiny.
//! The original non-iterative two-sphere response, including its recursive
//! initial-overlap displacement and strongest/equal-time collision choice.
//! This operates on the complete projected original fields. Native candidate
//! enumeration, platform floating-point math and production correspondence
//! remain separate obligations; it is not an Avian response model.

use crate::{OriginalMode,OriginalParkBall,OriginalParkError,Vec3,
    original_integrate_duration,original_integrate_tick};

pub(crate) fn two_sphere_time(p0:Vec3,p1:Vec3,q0:Vec3,q1:Vec3,radius:f64) -> f64 {
    let start=p0.sub(q0);
    let start2=start.dot(start);
    if start2<=radius*radius { return 0.0; }
    let delta=p1.sub(p0).sub(q1.sub(q0));
    let a=delta.dot(delta); let b=2.0*start.dot(delta); let c=start2-radius*radius;
    // Collision.cpp:137: the source does not return after its linear branch.
    // With a==0,b!=0 it still reaches the quadratic division by zero. Retain
    // that behavior, rather than silently replace it with a linear solver.
    if a==0.0 && b==0.0 { return -1.0; }
    let mut discriminant=b*b-4.0*a*c;
    if discriminant<0.0 { return -1.0; }
    discriminant=discriminant.sqrt();
    let second=(-b-discriminant)*0.5/a;
    if second>=0.0 && second<=1.0 { second } else { -1.0 }
}

fn response_acceleration(b:&OriginalParkBall,m:f64,velocity:Vec3) -> Vec3 {
    // Unary negation and both Vector3d divisions preserve source grouping.
    b.last_g.scale(-m).add(b.last_g.scale(b.time_factor*m))
        .sub(b.velocity.scale(b.time_factor).scale(1_000_000.0))
        .add(velocity.scale(1_000_000.0)).scale(-1.0)
        .scale(1.0/m).scale(1.0/(b.time_factor-1.0))
}

fn quadratic(a:f64,b:f64,c:f64) -> Option<(f64,f64)> {
    if a==0.0 && b==0.0 { return None; }
    let determinant=b*b-4.0*a*c;if determinant<0.0 { return None; }
    let determinant=determinant.sqrt();
    Some(((-b+determinant)*0.5/a,(-b-determinant)*0.5/a))
}

fn store_static_response(ball:&mut OriginalParkBall,id:i64,acceleration:Vec3,time:f64) -> Result<(),OriginalParkError> {
    let response=acceleration.scale(0.85);
    if time==ball.last_collision {
        if response.norm_squared()>ball.last_c.norm_squared() { ball.last_c=response; }
    } else { ball.last_c=response;ball.last_collision=time; }
    if !ball.collisions.push(id) { return Err(OriginalParkError::OutsideDeclaredScope); }
    Ok(())
}

fn capsule_react(ball:&mut OriginalParkBall,capsule:&crate::original_park::OriginalCapsule,
    mut position:Vec3,mut velocity:Vec3,normal:Vec3,time:f64) -> Result<(),OriginalParkError> {
    if time==-1.0 { return Ok(()); }
    let mass=ball.mass*f64::from(ball.agility);let center_distance=normal.length();let normal=normal.normalize();
    let component=ball.last_g.dot(normal);let acceleration;
    if time>0.0 {
        if component>0.0 { return Ok(()); }
        (position,velocity)=original_integrate_duration(position,velocity,ball.last_g,mass,1_000_000.0,ball.time_factor,1.0,time);
        velocity=velocity.sub(normal.scale(2.0*velocity.dot(normal)));
        let (_,velocity)=original_integrate_duration(position,velocity,ball.last_g,mass,1_000_000.0,ball.time_factor,1.0,1.0-time);
        acceleration=response_acceleration(ball,mass,velocity);
    } else {
        let coefficient=1.0/(mass+1_000_000.0);
        let distance=f64::from(capsule.radius)+f64::from(ball.radius)-center_distance+1.0;
        acceleration=normal.scale(distance/(coefficient*mass)).sub(ball.velocity).sub(normal.scale(component));
    }
    store_static_response(ball,capsule.id,acceleration,time)
}

pub(crate) fn capsule_collision(ball:&mut OriginalParkBall,capsule:&crate::original_park::OriginalCapsule) -> Result<(),OriginalParkError> {
    if !ball.is_free { return Ok(()); }
    let mass=ball.mass*f64::from(ball.agility);let p0=ball.position;
    let (p1,predicted_velocity)=original_integrate_tick(p0,ball.velocity,ball.last_g,mass,1_000_000.0,ball.time_factor);
    let radius=f64::from(ball.radius)+f64::from(capsule.radius);let ao=p0.sub(capsule.a);let ab=capsule.ab;
    let m=ab.dot(predicted_velocity)/ab.dot(ab);let n=ab.dot(ao)/ab.dot(ab);
    let q=predicted_velocity.sub(ab.scale(m));let r=ao.sub(ab.scale(n));
    let a=q.dot(q);let b=2.0*q.dot(r);let c=r.dot(r)-radius*radius;
    let mut normal=Vec3::ZERO;let mut time=-1.0;let mut selected=-1.0;
    if a==0.0 {
        let first=two_sphere_time(p0,p1,capsule.a,capsule.a,radius);
        if first!=-1.0 && (time==-1.0 || first<time) { time=first;normal=p0.sub(capsule.a); }
        let second=two_sphere_time(p0,p1,capsule.b,capsule.b,radius);
        if second!=-1.0 && (time==-1.0 || second<time) { time=second;normal=p0.sub(capsule.b); }
        // The original parallel branch passes the already predicted p1/vp1.
        // Other branches pass p0/current velocity. Keep this source asymmetry.
        return capsule_react(ball,capsule,p1,predicted_velocity,normal,time);
    }
    let Some((first,second))=quadratic(a,b,c) else { return Ok(()); };
    if first>=0.0 && first<=1.0 { selected=first; }
    if second>=0.0 && second<=1.0 && second<first { selected=second; }
    else if selected==-1.0 {
        let t=p0.sub(capsule.a).dot(ab)/ab.norm_squared();
        if t>=0.0 && t<=1.0 {
            let projection=capsule.a.add(ab.scale(t));let distance=p0.sub(projection);
            if distance.norm_squared()<radius*radius {
                return capsule_react(ball,capsule,p0,ball.velocity,distance,0.0);
            }
        }
    }
    let heights=[first*m+n,second*m+n];let mut i=0;
    while i<2 {
        let height=heights[i];
        if height<0.0 {
            let hit=two_sphere_time(p0,p1,capsule.a,capsule.a,radius);
            if hit!=-1.0 && (time==-1.0 || hit<time) { time=hit;normal=p0.sub(capsule.a); }
        } else if height>1.0 {
            let hit=two_sphere_time(p0,p1,capsule.b,capsule.b,radius);
            if hit!=-1.0 && (time==-1.0 || hit<time) {
                time=hit;normal=if i==0 { p0.sub(capsule.b) } else { p1.sub(capsule.b) };
            }
        } else if selected!=-1.0 && (time==-1.0 || selected<time) {
            time=selected;let projection=ab.scale(ao.dot(ab)/ab.norm_squared());normal=ao.sub(projection);
            if normal.norm_squared()<radius*radius { time=0.0; }
        }
        i+=1;
    }
    capsule_react(ball,capsule,p0,ball.velocity,normal,time)
}

pub(crate) fn box_collision(ball:&mut OriginalParkBall,object:&crate::original_park::OriginalOrientedBox) -> Result<(),OriginalParkError> {
    if !ball.is_free { return Ok(()); }
    let mass=ball.mass*f64::from(ball.agility);let p0=ball.position;
    let (p1,_)=original_integrate_tick(p0,ball.velocity,ball.last_g,mass,1_000_000.0,ball.time_factor);
    if object.aabb.can_exclude_collision(p0,p1,f64::from(ball.radius)) { return Ok(()); }
    let mut point=Vec3::ZERO;let mut time=-1.0;let mut normal=Vec3::ZERO;
    if !object.shape.collide_with_sphere(p0,f64::from(ball.radius),p1.sub(p0),&mut point,&mut time,&mut normal) { return Ok(()); }
    if time==-1.0 { return Ok(()); }
    let depth=normal.length();normal=normal.normalize();let acceleration;
    if time>0.0 {
        let (position,mut velocity)=original_integrate_duration(p0,ball.velocity,ball.last_g,mass,1_000_000.0,ball.time_factor,1.0,time);
        velocity=velocity.sub(normal.scale(2.0*velocity.dot(normal)));
        let (_,velocity)=original_integrate_duration(position,velocity,ball.last_g,mass,1_000_000.0,ball.time_factor,1.0,1.0-time);
        acceleration=response_acceleration(ball,mass,velocity);
    } else {
        let component=ball.last_g.dot(normal);let coefficient=1.0/(mass+1_000_000.0);
        acceleration=normal.scale((depth+1.0)/(coefficient*mass)).sub(ball.velocity).sub(normal.scale(component));
    }
    store_static_response(ball,object.id,acceleration,time)
}

pub(crate) fn sphere_potential(me:&mut OriginalParkBall,other:&mut OriginalParkBall,depth:u32)
    -> Result<(),OriginalParkError> {
    let radius=f64::from(me.radius)+f64::from(other.radius);
    let m1=if me.is_free { me.mass*f64::from(me.agility) } else { 1.0e34 };
    let m2=if other.is_free { other.mass*f64::from(other.agility) } else { 1.0e34 };
    let p0=me.position; let q0=other.position;
    let (predicted_p,_)=original_integrate_tick(p0,me.velocity,me.last_g,m1,1_000_000.0,me.time_factor);
    let (predicted_q,_)=original_integrate_tick(q0,other.velocity,other.last_g,m2,1_000_000.0,other.time_factor);
    let time=two_sphere_time(p0,predicted_p,q0,predicted_q,radius);
    if time==-1.0 { return Ok(()); }
    let mut p1=p0; let mut q1=q0; let mut vp1=me.velocity; let mut vq1=other.velocity;
    let acceleration;
    if time>0.0 {
        (p1,vp1)=original_integrate_duration(p1,vp1,me.last_g,m1,1_000_000.0,me.time_factor,1.0,time);
        (q1,vq1)=original_integrate_duration(q1,vq1,other.last_g,m2,1_000_000.0,other.time_factor,1.0,time);
        let normal=q1.sub(p1).normalize(); let v1=vp1.dot(normal); let v2=vq1.dot(normal);
        if !other.is_free { vp1=vp1.sub(normal.scale(2.0*v1)); }
        else {
            let mm1=me.mass; let mm2=other.mass;
            let v1p=(mm1*v1-mm2*v1+2.0*mm2*v2)/(mm1+mm2);
            vp1=vp1.add(normal.scale(v1p-v1));
        }
        let (_,final_velocity)=original_integrate_duration(p1,vp1,me.last_g,m1,1_000_000.0,me.time_factor,1.0,1.0-time);
        acceleration=response_acceleration(me,m1,final_velocity);
    } else {
        let difference=q0.sub(p0); let separation2=difference.dot(difference);
        let normal=if separation2==0.0 {
            Vec3 { x:if me.id>other.id { 1.0 } else { -1.0 },y:0.0,z:0.0 }
        } else { q1.sub(p1).normalize() };
        let distance=radius-separation2.sqrt()+1.0;
        let d1=m2/(m1+m2)*distance; let d2=m1/(m1+m2)*distance;
        if me.is_free && other.is_free && depth<2 {
            let saved_p=me.position; let saved_q=other.position;
            me.position=me.position.sub(normal.scale(d1)); other.position=other.position.add(normal.scale(d2));
            sphere_potential(me,other,depth+1)?;
            me.position=saved_p; other.position=saved_q;
            if m2/m1>25.0 { me.last_c=me.last_c.normalize().scale(distance); }
            if me.velocity.sub(other.velocity).norm_squared()>0.0001 { return Ok(()); }
        }
        let coefficient=1.0/(m1+1_000_000.0);
        let normal_component=me.last_g.dot(normal);
        acceleration=normal.scale(-d1/(coefficient*m1)).sub(me.velocity)
            .sub(normal.scale(normal_component));
    }
    if (me.mode!=OriginalMode::Missile || other.id!=me.follow_id) && time>=me.last_collision {
        let collision=acceleration.scale(0.85);
        if time==me.last_collision {
            if collision.norm_squared()>me.last_c.norm_squared() { me.last_c=collision; }
        } else { me.last_c=collision; me.last_collision=time; }
    }
    if !me.collisions.push(other.id) { return Err(OriginalParkError::OutsideDeclaredScope); }
    Ok(())
}
