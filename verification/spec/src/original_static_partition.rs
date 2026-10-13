// Copyright © 2015 CCP ehf. MIT license, retained from original Destiny.
//! Capsule.cpp:449 and OrientedBox.cpp:127 static insertion footprints.
//! Capsule sampling retains binary32 length/decrement and every point. OBB
//! neighbours use the original AABB bounds, not a bounding-sphere proxy.
//! 64 capsule samples / 128 unique cells bound these catalog scenarios.
//! Capacity/conversion/nonterminating scopes return errors, never omission.
//! Allocation, native hash iteration and pointer/refcount correspondence remain.

use crate::{BoundedList,OriginalParkError,Vec3};
use crate::original_park::{OriginalCapsule,OriginalOrientedBox};
use crate::original_partition::{CellKey,box_low,box_width,primary_key,partition_level,capsule_sample_keys,TOP_WIDTH};

#[derive(Clone, Copy)]
pub(crate) struct StaticCellKeys {
    pub level:u32,
    pub primary:CellKey,
    pub keys:BoundedList<CellKey,128>,
    pub owned_keys:BoundedList<CellKey,128>,
}

fn add(list:&mut BoundedList<CellKey,128>,key:CellKey) -> Result<(),OriginalParkError> {
    let mut i=0;
    while i<list.len() { if list.get(i)==Some(key) { return Ok(()); } i+=1; }
    if list.push(key) { Ok(()) } else { Err(OriginalParkError::OutsideDeclaredScope) }
}

pub(crate) fn capsule_keys(capsule:&OriginalCapsule) -> Result<StaticCellKeys,OriginalParkError> {
    if capsule.radius<=0.0 { return Err(OriginalParkError::OutsideDeclaredScope); }
    let level=partition_level(f64::from(capsule.radius));
    let Some(primary)=primary_key(capsule.position,level) else { return Err(OriginalParkError::OutsideDeclaredScope); };
    let mut result=StaticCellKeys { level,primary,keys:BoundedList::new(),owned_keys:BoundedList::new() };
    // GetBox(center) creates this cell even if no sampled object is inserted in
    // it. The active-box query includes these allocated unowned cells as well.
    add(&mut result.keys,primary)?;
    let mut position=capsule.a;let delta=capsule.b.sub(capsule.a);
    let mut length=delta.length() as f32;
    let direction=delta.normalize().scale(f64::from(capsule.radius)).scale(2.0);
    let mut sample=0;
    while length>=0.0 {
        if sample>=64 { return Err(OriginalParkError::OutsideDeclaredScope); }
        let Some(keys)=capsule_sample_keys(position,capsule.radius) else { return Err(OriginalParkError::OutsideDeclaredScope); };
        let mut i=0;
        while i<keys.keys.len() { if let Some(key)=keys.keys.get(i) { add(&mut result.keys,key)?;add(&mut result.owned_keys,key)?; } i+=1; }
        position=position.add(direction);
        length-=capsule.radius*2.0_f32;
        sample+=1;
    }
    Ok(result)
}

pub(crate) fn oriented_box_keys(object:&OriginalOrientedBox) -> Result<StaticCellKeys,OriginalParkError> {
    let size=object.aabb.high.sub(object.aabb.low);
    // std::max(dx,std::max(dy,dz)) keeps its first argument for unordered
    // comparisons. Do not replace this nesting by a different NaN policy.
    let yz=if size.y<size.z { size.z } else { size.y };
    let longest=if size.x<yz { yz } else { size.x };
    let level=partition_level(longest/2.0);
    let Some(primary)=primary_key(object.position,level) else { return Err(OriginalParkError::OutsideDeclaredScope); };
    let Some(lo)=box_low(primary,level) else { return Err(OriginalParkError::OutsideDeclaredScope); };
    let Some(width)=box_width(level) else { return Err(OriginalParkError::OutsideDeclaredScope); };
    let hi=lo.add(Vec3 { x:width,y:width,z:width });
    let modifier=|low:f64,high:f64,aabb_low:f64,aabb_high:f64| {
        if low-aabb_low>0.0 { -1 } else if high-aabb_high<0.0 { 1 } else { 0 }
    };
    let mx=modifier(lo.x,hi.x,object.aabb.low.x,object.aabb.high.x);
    let my=modifier(lo.y,hi.y,object.aabb.low.y,object.aabb.high.y);
    let mz=modifier(lo.z,hi.z,object.aabb.low.z,object.aabb.high.z);
    let grid=1_i64<<(23+2*level);
    let mut result=StaticCellKeys { level,primary,keys:BoundedList::new(),owned_keys:BoundedList::new() };add(&mut result.keys,primary)?;
    if level==0 && f64::from(object.bounding_radius)*2.0>TOP_WIDTH {
        let mut x=-1;while x<=1 { let mut y=-1;while y<=1 { let mut z=-1;while z<=1 {
            if x!=0 || y!=0 || z!=0 { add(&mut result.keys,primary.offset(x,y,z,grid))?; }z+=1;
        }y+=1;}x+=1;}
    } else {
        let offsets=[(mx!=0,mx,0,0),(my!=0,0,my,0),(mz!=0,0,0,mz),
            (mx!=0 && my!=0,mx,my,0),(mx!=0 && mz!=0,mx,0,mz),
            (my!=0 && mz!=0,0,my,mz),(mx!=0 && my!=0 && mz!=0,mx,my,mz)];
        let mut i=0;
        while i<offsets.len() {
            if let Some((active,x,y,z))=offsets.get(i) { if *active { add(&mut result.keys,primary.offset(*x,*y,*z,grid))?; } }
            i+=1;
        }
    }
    result.owned_keys=result.keys;
    Ok(result)
}
