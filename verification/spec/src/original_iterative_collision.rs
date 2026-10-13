// Copyright © 2015, 2023 CCP ehf. Original Destiny MIT license retained.
//! Original old-orientation iterative impacts, substeps and invalidation.
//!
//! Ballpark.cpp:332,1988..2713 and CollisionBallProperties.cpp are behavioral
//! authority. The source default is twenty passes, not a convergence shortcut.
//! Eight projected balls and twenty-two locations per ball bound the concrete
//! catalog scenarios; the source reserve(22) is not an unbounded-vector theorem.
//! Capacity/unsupported inputs return scope errors and are checked by callers.
//! Native candidate order, object/pointer identity, dynamic orientation and
//! platform math remain separate correspondence obligations.

use crate::{BoundedList,OriginalMode,OriginalParkBall,OriginalParkError,Vec3,
    original_integrate_duration};
use crate::original_collision::two_sphere_time;
use crate::original_park::{OriginalCapsule,OriginalOrientedBox};

#[derive(Debug,Clone,Copy,PartialEq)]
pub(crate) struct CollisionLocation { pub time:f64,pub collider:i64,pub p:Vec3,pub v:Vec3 }
#[derive(Debug,Clone,Copy,PartialEq)]
pub(crate) struct CollisionProperties {
    pub locations:BoundedList<CollisionLocation,22>,
    pub t_start:f64,
    pub t_end:f64,
}
#[derive(Debug,Clone,Copy)]
struct Impact { time:f64,collider:i64,velocity_change:Vec3 }
#[derive(Debug,Clone,Copy)]
struct CollisionItem { time:f64,collider:usize,normal:Vec3,is_static:bool }
#[derive(Debug,Clone,Copy)]
pub(crate) enum StaticCollider { Capsule(OriginalCapsule),Box(OriginalOrientedBox) }
impl StaticCollider {
    pub fn id(self)->i64 { match self { Self::Capsule(c)=>c.id,Self::Box(b)=>b.id } }
    fn check(self,p0:Vec3,p1:Vec3,radius:f32)->Option<(f64,Vec3)> {
        match self {
            Self::Capsule(c)=>capsule_check(c,p0,p1,radius),
            Self::Box(b)=> {
                if b.aabb.can_exclude_collision(p0,p1,f64::from(radius)) { return None; }
                let mut point=Vec3::ZERO;let mut time=0.0;let mut normal=Vec3::ZERO;
                if !b.shape.collide_with_sphere(p0,f64::from(radius),p1.sub(p0),&mut point,&mut time,&mut normal) { return None; }
                let length=normal.length();
                normal=if length>1e-10 { normal.scale(1.0/length) } else { Vec3{x:0.0,y:0.0,z:1.0} };
                Some((time,normal))
            }
        }
    }
}
fn push<T:Copy,const N:usize>(list:&mut BoundedList<T,N>,value:T)->Result<(),OriginalParkError> {
    if list.push(value) { Ok(()) } else { Err(OriginalParkError::OutsideDeclaredScope) }
}
fn get<T:Copy,const N:usize>(list:&BoundedList<T,N>,index:usize)->Result<T,OriginalParkError> {
    list.get(index).ok_or(OriginalParkError::OutsideDeclaredScope)
}
fn body(balls:&[Option<OriginalParkBall>;8],index:usize)->Result<OriginalParkBall,OriginalParkError> {
    balls.get(index).copied().flatten().ok_or(OriginalParkError::OutsideDeclaredScope)
}
fn store(balls:&mut [Option<OriginalParkBall>;8],index:usize,ball:OriginalParkBall)->Result<(),OriginalParkError> {
    *balls.get_mut(index).ok_or(OriginalParkError::OutsideDeclaredScope)?=Some(ball);Ok(())
}
fn integrate(ball:OriginalParkBall,p:Vec3,v:Vec3,friction:f64,duration:f64)->(Vec3,Vec3) {
    original_integrate_duration(p,v,ball.last_g.add(ball.last_c),ball.mass*f64::from(ball.agility),friction,ball.time_factor,1.0,duration)
}
impl CollisionProperties {
    fn initialize(ball:OriginalParkBall,friction:f64)->Result<Self,OriginalParkError> {
        let mut locations=BoundedList::new();
        push(&mut locations,CollisionLocation{time:0.0,collider:-1,p:ball.position,v:ball.velocity})?;
        let (p,v)=integrate(ball,ball.position,ball.velocity,friction,1.0);
        push(&mut locations,CollisionLocation{time:1.0,collider:-1,p,v})?;
        Ok(Self{locations,t_start:0.0,t_end:1.0})
    }
    pub fn position(&self,ball:OriginalParkBall,friction:f64,time:f64)->Result<(Vec3,Vec3),OriginalParkError> {
        let mut index=0;
        while index<self.locations.len() && get(&self.locations,index)?.time+1e-4<time { index+=1; }
        if index!=0 { index-=1; }
        let point=get(&self.locations,index)?;
        if (point.time-time).abs()>=1e-4 { Ok(integrate(ball,point.p,point.v,friction,time-point.time)) }
        else { Ok((point.p,point.v)) }
    }
    fn ignore_collision(&self,collider:i64,time:f64)->Result<bool,OriginalParkError> {
        if self.locations.len()<2 || time>1e-4 { return Ok(false); }
        let mut index=self.locations.len()-2;let lower=get(&self.locations,index)?.time-1e-4;
        loop {
            let location=get(&self.locations,index)?;
            if location.time<lower { break; }
            if location.collider==collider { return Ok(true); }
            if index==0 { break; }index-=1;
        }
        Ok(false)
    }
    fn calculate_end(&mut self,ball:OriginalParkBall,friction:f64)->Result<(),OriginalParkError> {
        let previous=get(&self.locations,self.locations.len().checked_sub(1).ok_or(OriginalParkError::OutsideDeclaredScope)?)?;
        if 1.0>previous.time {
            let (p,v)=integrate(ball,previous.p,previous.v,friction,1.0-previous.time);
            push(&mut self.locations,CollisionLocation{time:1.0,collider:-1,p,v})?;
            self.t_end=1.0;self.t_start=previous.time;
        }
        Ok(())
    }
    fn add_impact(&mut self,ball:OriginalParkBall,friction:f64,impact:Impact)->Result<BoundedList<(f64,i64),22>,OriginalParkError> {
        let mut invalidated=BoundedList::new();
        if self.t_end<=impact.time { return Ok(invalidated); }
        let mut index=self.locations.len();
        while index>0 {
            let location=get(&self.locations,index-1)?;
            if impact.time>=location.time { break; }
            if location.collider!=-1 { push(&mut invalidated,(location.time,location.collider))?; }
            index-=1;
        }
        let mut kept=BoundedList::new();let mut i=0;
        while i<index { push(&mut kept,get(&self.locations,i)?)?;i+=1; }
        let (p,v)=if index==0 { (ball.position,ball.velocity) }
            else { let previous=get(&self.locations,index-1)?;integrate(ball,previous.p,previous.v,friction,impact.time-previous.time) };
        push(&mut kept,CollisionLocation{time:impact.time,collider:impact.collider,p,v:v.add(impact.velocity_change)})?;
        self.locations=kept;self.t_end=impact.time;Ok(invalidated)
    }
    fn remove_impact(&mut self,ball:OriginalParkBall,friction:f64,time:f64,collider:i64)->Result<BoundedList<(f64,i64),22>,OriginalParkError> {
        let mut invalidated=BoundedList::new();let mut index=0;let mut i=0;
        // Source initializes found=true. Preserve its index-zero fallthrough
        // when no matching impact exists instead of repairing the apparent bug.
        while i<self.locations.len() {
            let point=get(&self.locations,i)?;
            if point.collider==collider && (point.time-time).abs()<1e-4 { index=i;break; }i+=1;
        }
        if index==0 {
            let mut first=get(&self.locations,0)?;first.collider=-1;first.v=ball.velocity;
            if !self.locations.set(0,first) { return Err(OriginalParkError::OutsideDeclaredScope); }
        }
        i=index+1;while i<self.locations.len() {
            let point=get(&self.locations,i)?;
            if point.collider!=-1 { push(&mut invalidated,(point.time,point.collider))?; }i+=1;
        }
        let mut kept=BoundedList::new();i=0;
        while i<index.max(1) { push(&mut kept,get(&self.locations,i)?)?;i+=1; }
        self.locations=kept;self.calculate_end(ball,friction)?;Ok(invalidated)
    }
}

#[derive(Default)]
pub(crate) struct OriginalIterativeState { pub properties:[Option<CollisionProperties>;8] }
impl OriginalIterativeState {
    fn ensure(&mut self,balls:&[Option<OriginalParkBall>;8],index:usize,friction:f64)->Result<(),OriginalParkError> {
        if self.properties.get(index).ok_or(OriginalParkError::OutsideDeclaredScope)?.is_none() {
            self.properties[index]=Some(CollisionProperties::initialize(body(balls,index)?,friction)?);
        }
        Ok(())
    }
    fn properties(&self,index:usize)->Result<CollisionProperties,OriginalParkError> {
        self.properties.get(index).copied().flatten().ok_or(OriginalParkError::OutsideDeclaredScope)
    }
    fn end(&mut self,balls:&[Option<OriginalParkBall>;8],index:usize,friction:f64)->Result<(),OriginalParkError> {
        let mut prop=self.properties(index)?;prop.calculate_end(body(balls,index)?,friction)?;
        self.properties[index]=Some(prop);Ok(())
    }
    fn remove_impact(&mut self,balls:&[Option<OriginalParkBall>;8],index:usize,friction:f64,time:f64,collider:i64,depth:usize)->Result<(),OriginalParkError> {
        // Each recursive removal deletes later stored impacts. Twenty-two
        // locations in each of eight bodies bound a well-founded chain by 176.
        if depth>176 { return Err(OriginalParkError::OutsideDeclaredScope); }
        let ball=body(balls,index)?;
        if !ball.is_free { return Ok(()); }
        if self.properties[index].is_none() { return Err(OriginalParkError::OutsideDeclaredScope); }
        let mut prop=self.properties(index)?;let invalidated=prop.remove_impact(ball,friction,time,collider)?;
        self.properties[index]=Some(prop);let mut i=0;
        while i<invalidated.len() {
            let (time,id)=get(&invalidated,i)?;let mut j=0;
            while j<8 {
                if balls[j].is_some_and(|b|b.in_park && b.is_free && b.id==id) { self.remove_impact(balls,j,friction,time,ball.id,depth+1)?;break; }j+=1;
            }i+=1;
        }
        Ok(())
    }
    fn add_impact(&mut self,balls:&[Option<OriginalParkBall>;8],index:usize,friction:f64,mut impact:Impact)->Result<(),OriginalParkError> {
        let ball=body(balls,index)?;let mut prop=self.properties(index)?;
        if prop.t_end<=impact.time { return Ok(()); }
        let speed2=impact.velocity_change.norm_squared();
        // The source float multiplication rounds before conversion to double.
        let fraction=f64::from(25.0_f32*ball.max_velocity*ball.max_velocity)/speed2;
        if fraction<1.0 { impact.velocity_change=impact.velocity_change.scale(fraction); }
        let invalidated=prop.add_impact(ball,friction,impact)?;
        self.properties[index]=Some(prop);let mut i=0;
        while i<invalidated.len() {
            let (time,id)=get(&invalidated,i)?;let mut j=0;
            while j<8 {
                if balls[j].is_some_and(|b|b.in_park && b.is_free && b.id==id) { self.remove_impact(balls,j,friction,time,ball.id,0)?;break; }j+=1;
            }i+=1;
        }
        Ok(())
    }
    fn next_collision(&mut self,balls:&[Option<OriginalParkBall>;8],index:usize,friction:f64,
        neighbors:&BoundedList<usize,8>,static_indices:&BoundedList<usize,16>,statics:&BoundedList<StaticCollider,16>)
        ->Result<Option<CollisionItem>,OriginalParkError> {
        let ball=body(balls,index)?;let prop=self.properties(index)?;
        let (p0,_)=prop.position(ball,friction,prop.t_start)?;let (p1,_)=prop.position(ball,friction,prop.t_end)?;
        let mut selected:Option<CollisionItem>=None;let mut i=0;
        if !matches!(ball.mode,OriginalMode::Missile|OriginalMode::Mushroom) {
            while i<static_indices.len() {
                let collider=get(static_indices,i)?;let object=get(statics,collider)?;
                if let Some((time,normal))=object.check(p0,p1,ball.radius) {
                    if !prop.ignore_collision(object.id(),time)? {
                        let time=prop.t_start+(prop.t_end-prop.t_start)*time;
                        if selected.is_none_or(|old|time<old.time) { selected=Some(CollisionItem{time,normal,collider,is_static:true}); }
                    }
                }
                if selected.is_some_and(|s|s.time==prop.t_start) { break; }i+=1;
            }
        }
        if !selected.is_some_and(|s|s.time==prop.t_start) {
            i=0;while i<neighbors.len() {
                let collider=get(neighbors,i)?;let neighbor=body(balls,collider)?;
                if !neighbor.is_free && !neighbor_ignore(ball,neighbor) {
                    let time=two_sphere_time(p0,p1,neighbor.position,neighbor.position,f64::from(ball.radius)+f64::from(neighbor.radius));
                    if time>=0.0 && !prop.ignore_collision(neighbor.id,time)? {
                        let actual=prop.t_start+time*(prop.t_end-prop.t_start);
                        // Retain the source's comparison of the fraction time
                        // with the already-absolute previous collision time.
                        if selected.is_none_or(|old|time<old.time) {
                            let normal=p0.add(p1.sub(p0).scale(time)).sub(neighbor.position).normalize();
                            selected=Some(CollisionItem{time:actual,normal,collider,is_static:false});
                        }
                    }
                }
                if selected.is_some_and(|s|s.time==prop.t_start) { break; }i+=1;
            }
        }
        if selected.is_some_and(|s|s.time==prop.t_start) { return Ok(selected); }
        i=0;
        while i<neighbors.len() {
            let collider=get(neighbors,i)?;let other=body(balls,collider)?;
            if other.is_free && (other.id>ball.id || ball.mode==OriginalMode::Missile) && !neighbor_ignore(ball,other) {
                self.ensure(balls,collider,friction)?;
                let other_prop=self.properties(collider)?;let end=prop.t_end.min(other_prop.t_end);
                if prop.t_start>=end { i+=1;continue; }
                let mut times=BoundedList::<f64,24>::new();push(&mut times,prop.t_start)?;let mut j=0;
                while j<other_prop.locations.len() {
                    let point=get(&other_prop.locations,j)?;
                    if point.time>prop.t_start && point.time<end { push(&mut times,point.time)?; }j+=1;
                }
                push(&mut times,end)?;
                let (mut bp0,_)=prop.position(ball,friction,get(&times,0)?)?;
                let (mut op0,_)=other_prop.position(other,friction,get(&times,0)?)?;j=0;
                while j+1<times.len() {
                    let start=get(&times,j)?;let end=get(&times,j+1)?;
                    let (bp1,_)=prop.position(ball,friction,end)?;let (op1,_)=other_prop.position(other,friction,end)?;
                    let time=two_sphere_time(bp0,bp1,op0,op1,f64::from(ball.radius)+f64::from(other.radius));
                    if time>=0.0 {
                        let actual=start+(end-start)*time;
                        if selected.is_none_or(|old|actual<old.time) {
                            if prop.ignore_collision(other.id,time)? { j+=1;continue; }
                            selected=Some(CollisionItem{time:actual,normal:Vec3::ZERO,collider,is_static:false});break;
                        }
                    }
                    if times.len()>2 { bp0=bp1;op0=op1; }j+=1;
                }
                if selected.is_some_and(|s|s.time==prop.t_start) { break; }
            }i+=1;
        }
        Ok(selected)
    }
    fn response(&mut self,balls:&mut [Option<OriginalParkBall>;8],index:usize,friction:f64,
        neighbors:&BoundedList<usize,8>,static_indices:&BoundedList<usize,16>,statics:&BoundedList<StaticCollider,16>)
        ->Result<bool,OriginalParkError> {
        self.ensure(balls,index,friction)?;let prop=self.properties(index)?;
        if prop.t_start>=prop.t_end { self.end(balls,index,friction)?;return Ok(false); }
        if get(&prop.locations,0)?.collider!=-1 { return Ok(false); }
        if neighbors.is_empty() && static_indices.is_empty() { return Ok(false); }
        let selected=self.next_collision(balls,index,friction,neighbors,static_indices,statics)?;
        if let Some(collision)=selected {
            let mut ball=body(balls,index)?;
            let static_collision=collision.is_static || !body(balls,collision.collider)?.is_free;
            if static_collision {
                let normal=collision.normal;
                if prop.t_start==collision.time {
                    let acceleration=f64::from(ball.max_velocity)*friction/(ball.mass*f64::from(ball.agility))-ball.last_g.dot(normal);
                    ball.last_c=normal.scale(acceleration);store(balls,index,ball)?;
                }
                let (_,velocity)=prop.position(ball,friction,collision.time)?;
                let impulse=velocity.scale(-2.0).dot(normal);
                // new CollisionImpact() value-initializes collider to zero,
                // not -1. The source's negative-impulse early return therefore
                // still adds a zero-velocity impact and recalculates the end
                // with the overlap acceleration. Preserve this behavior.
                let id=if impulse<0.0 { 0 } else if collision.is_static { get(statics,collision.collider)?.id() } else { body(balls,collision.collider)?.id };
                let change=if impulse<0.0 { Vec3::ZERO } else { normal.scale(impulse) };
                self.add_impact(balls,index,friction,Impact{time:collision.time,collider:id,velocity_change:change})?;
                self.end(balls,index,friction)?;return Ok(true);
            } else {
                let other_index=collision.collider;let mut other=body(balls,other_index)?;
                let other_prop=self.properties(other_index)?;
                let (p,v)=prop.position(ball,friction,collision.time)?;
                let (q,w)=other_prop.position(other,friction,collision.time)?;
                let mut normal=p.sub(q);let length=normal.length();
                if length>0.0 { normal=normal.scale(1.0/length); } else { normal.z=1.0; }
                let other_normal=normal.scale(-1.0);
                if collision.time==0.0 {
                    let acceleration=f64::from(ball.max_velocity)*friction/(ball.mass*f64::from(ball.agility))-ball.last_g.dot(normal);
                    ball.last_c=normal.scale(acceleration);store(balls,index,ball)?;
                    let acceleration=f64::from(other.max_velocity)*friction/(other.mass*f64::from(other.agility))-other.last_g.dot(other_normal);
                    other.last_c=other_normal.scale(acceleration);store(balls,other_index,other)?;
                }
                let impulse=w.sub(v).dot(other_normal)*-2.0;
                let negative=impulse<0.0;
                let impulse=if negative { 0.0 } else { ball.mass*other.mass*impulse/(ball.mass+other.mass) };
                self.add_impact(balls,other_index,friction,Impact{time:collision.time,collider:if negative { 0 } else { ball.id },
                    velocity_change:if negative { Vec3::ZERO } else { other_normal.scale(impulse/other.mass) }})?;
                self.add_impact(balls,index,friction,Impact{time:collision.time,collider:if negative { 0 } else { other.id },
                    velocity_change:if negative { Vec3::ZERO } else { normal.scale(impulse/ball.mass) }})?;
                self.end(balls,index,friction)?;return Ok(true);
            }
        }
        self.end(balls,index,friction)?;Ok(false)
    }
    pub fn calculate(&mut self,balls:&mut [Option<OriginalParkBall>;8],free:&BoundedList<usize,8>,dead:[bool;8],
        friction:f64,neighbors:&[BoundedList<usize,8>;8],static_indices:&[BoundedList<usize,16>;8],statics:&BoundedList<StaticCollider,16>)
        ->Result<(),OriginalParkError> {
        let mut i=0;
        while i<8 {
            if self.properties[i].is_some() { self.properties[i]=Some(CollisionProperties::initialize(body(balls,i)?,friction)?); }i+=1;
        }
        let mut current=*free;let mut iteration=0;
        loop {
            let mut next=BoundedList::new();i=0;
            while i<current.len() {
                let index=get(&current,i)?;let ball=body(balls,index)?;
                if !ball.is_moribund && !dead[index] && ball.is_massive {
                    if self.response(balls,index,friction,&neighbors[index],&static_indices[index],statics)? {
                        let mut j=0;
                        while j<neighbors[index].len() {
                            let neighbor=get(&neighbors[index],j)?;
                            if body(balls,neighbor)?.is_free { insert_free_index(&mut next,balls,neighbor)?; }j+=1;
                        }
                        insert_free_index(&mut next,balls,index)?;
                    }
                }i+=1;
            }
            if next.is_empty() { break; }
            iteration+=1;if iteration>=20 { break; }
            current=next;
        }
        Ok(())
    }
}
fn insert_free_index(list:&mut BoundedList<usize,8>,balls:&[Option<OriginalParkBall>;8],index:usize)->Result<(),OriginalParkError> {
    let id=body(balls,index)?.id;let mut i=0;
    while i<list.len() {
        let previous=get(list,i)?;if previous==index { return Ok(()); }
        if body(balls,previous)?.id>id { break; }i+=1;
    }
    if list.insert(i,index) { Ok(()) } else { Err(OriginalParkError::OutsideDeclaredScope) }
}
fn neighbor_ignore(b:OriginalParkBall,n:OriginalParkBall)->bool {
    (b.mode==OriginalMode::Missile && (n.id==b.owner_id || n.mode==OriginalMode::Miniball && n.owner_id==b.owner_id))
        || (matches!(n.mode,OriginalMode::Missile|OriginalMode::Mushroom) && b.id==n.owner_id)
        || (n.mode==OriginalMode::Missile && b.id==n.follow_id)
        || (n.is_space_junk && !b.is_space_junk) || !n.is_massive
}

fn capsule_check(c:OriginalCapsule,p0:Vec3,p1:Vec3,radius:f32)->Option<(f64,Vec3)> {
    let bounding=f64::from((c.ab.length() as f32)*0.5+c.radius)+f64::from(radius);
    let path=p1.sub(p0);let path2=path.dot(path);let separation=c.position.sub(p0);let end=c.position.sub(p1);
    let closest=separation.dot(path);
    if closest<0.0 && bounding*bounding<separation.dot(separation) || closest>path2 && bounding*bounding<end.dot(end) { return None; }
    let distance2=path2*separation.norm_squared()-closest*closest;
    if distance2>path2*bounding*bounding { return None; }
    let length2=c.ab.norm_squared();
    if length2<1e-10 {
        let time=two_sphere_time(p0,p1,c.position,c.position,bounding);
        if time<0.0 { return None; }
        let normal=p0.add(path.scale(time)).sub(c.position);let length=normal.length();
        return Some((time,if length<1e-10 { Vec3{x:0.0,y:0.0,z:1.0} } else { normal.scale(1.0/length) }));
    }
    let radius=f64::from(radius)+f64::from(c.radius);let reciprocal=1.0/length2;let length=length2.sqrt();
    let ao=p0.sub(c.a);let parallel=ao.dot(c.ab)/length;
    let perpendicular_path=path.sub(c.ab.scale(path.dot(c.ab)).scale(reciprocal));
    let perpendicular_ao=ao.sub(c.ab.scale(ao.dot(c.ab)).scale(reciprocal));
    let constant=perpendicular_ao.dot(perpendicular_ao)-radius*radius;
    if constant<=0.0 {
        if parallel>=0.0 && parallel<=length { return Some((0.0,perpendicular_ao.normalize())); }
        let center=if parallel<0.0 { c.a } else { c.b };
        let time=two_sphere_time(p0,p1,center,center,radius);
        return if time>=0.0 { Some((time,p0.add(path.scale(time)).sub(center).normalize())) } else { None };
    }
    let a=perpendicular_path.dot(perpendicular_path);if a<1e-10 { return None; }
    let b=2.0*perpendicular_path.dot(perpendicular_ao);let determinant=b*b-4.0*a*constant;
    if determinant<0.0 { return None; }
    let second=(-b-determinant.sqrt())*0.5/a;
    if second>=0.0 && second<=1.0 {
        let pc=p0.add(path.scale(second)).sub(c.a);let height=pc.dot(c.ab)/length;
        if height>=0.0 && height<=length { return Some((second,pc.sub(c.ab.scale(pc.dot(c.ab)).scale(reciprocal)).normalize())); }
        let center=if height<0.0 { c.a } else { c.b };let time=two_sphere_time(p0,p1,center,center,radius);
        if time>=0.0 { return Some((time,p0.add(path.scale(time)).sub(center).normalize())); }
    }
    None
}
