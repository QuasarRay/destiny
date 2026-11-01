// Copyright © 2014 CCP ehf. Original Partition, Box and sorting semantics.
// Bevy ECS port of Partition.h, Partition.cpp, and Partition_Blue.cpp.
use std::collections::{BTreeMap, BTreeSet};

use bevy::{
    math::DVec3,
    prelude::{App, Component, Entity, Plugin, Resource, World},
};
use blueexposure::{blue_function, BlueAppExt, BlueModule};
use destiny_partitionable::{BoxKey, Partitionable};
use pyo3::{exceptions::PyTypeError, prelude::*};

const LEVEL_COUNT: usize = 8;
const GRID_UNIT: f64 = 480.0;

/// Box membership metadata only: eventually supplied by the Rust Ball port.
/// The remaining common state lives in the existing Partitionable component.
#[derive(Component, Clone, Debug)]
pub struct PartitionBall {
    pub owner_id: i64,
    pub mode: BallMode,
    pub cloaked: bool,
    pub massive: bool,
    pub free: bool,
    pub global: bool,
    pub harmonic: i64,
    pub corporation_id: i64,
    pub alliance_id: i64,
    pub followed: Option<Entity>,
    pub proximity_only_interactives: bool,
}
impl Default for PartitionBall {
    fn default() -> Self {
        Self {
            owner_id: 0,
            mode: BallMode::Other,
            cloaked: false,
            massive: false,
            free: false,
            global: false,
            harmonic: -1,
            corporation_id: -1,
            alliance_id: -1,
            followed: None,
            proximity_only_interactives: false,
        }
    }
}

/// Only modes distinguished by the original Partition filtering algorithm.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BallMode {
    #[default]
    Other,
    Missile,
    Field,
}

/// Parent ball ID is needed for the deterministic static-collidable sort order.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PartitionStatic {
    pub parent_ball_id: i64,
}

mod box;
pub use box::{BoxObject, PartitionBox};

/// Bevy World-local equivalent of Partition's level maps and grid geometry.
#[derive(Resource, Debug)]
pub struct Partition {
    levels: Vec<BTreeMap<BoxKey, Entity>>,
    pub level_width: Vec<f64>,
    pub level_grid: Vec<i64>,
}

impl Default for Partition {
    fn default() -> Self {
        let bigbox = ((1_u64 << 16) as f64) * GRID_UNIT * 0.25;
        Self {
            levels: (0..LEVEL_COUNT).map(|_| BTreeMap::new()).collect(),
            level_width: (0..LEVEL_COUNT)
                .map(|i| bigbox / ((1_u64 << (2 * i)) as f64))
                .collect(),
            level_grid: (0..LEVEL_COUNT)
                .map(|i| 1_i64 << (2 * i + 23))
                .collect(),
        }
    }
}

impl Partition {
    #[must_use]
    pub fn number_of_levels(&self) -> usize {
        self.levels.len()
    }

    #[must_use]
    pub fn get_box_key(&self, position: DVec3, level: i32) -> (i64, i64, i64) {
        let Some(i) = self.valid_level(level) else { return (0, 0, 0); };
        let w = self.level_width[i];
        let offset = 0.5 * w * self.level_grid[i] as f64;
        let fac = 1.0 / w;
        (
            ((position.x + offset) * fac) as i64,
            ((position.y + offset) * fac) as i64,
            ((position.z + offset) * fac) as i64,
        )
    }

    fn valid_level(&self, level: i32) -> Option<usize> {
        let i = usize::try_from(level).ok()?;
        (i < self.levels.len()).then_some(i)
    }

    fn key(&self, ix: i64, iy: i64, iz: i64, level: i32) -> Option<(usize, BoxKey)> {
        let i = self.valid_level(level)?;
        Some((i, BoxKey::new(ix, iy, iz, self.level_grid[i])))
    }

    #[must_use]
    pub fn check_box(&self, ix: i64, iy: i64, iz: i64, level: i32) -> Option<Entity> {
        let (i, key) = self.key(ix, iy, iz, level)?;
        self.levels[i].get(&key).copied()
    }

    /// Equivalent to GetBox(i,j,k,level), using World entities in place of BoxAllocator.
    /// This creates the coarser parents on demand just as Box::SetParent does.
    pub fn get_box(
        &mut self, world: &mut World, ix: i64, iy: i64, iz: i64, level: i32,
    ) -> Option<Entity> {
        let (i, key) = self.key(ix, iy, iz, level)?;
        if let Some(entity) = self.levels[i].get(&key).copied() {
            return Some(entity);
        }
        let parent = if i == 0 {
            None
        } else {
            self.get_box(world, key.ix >> 2, key.iy >> 2, key.iz >> 2, level - 1)
        };
        let width = self.level_width[i];
        let center_offset = 0.5 * width * self.level_grid[i] as f64;
        let lo = DVec3::new(
            key.ix as f64 * width - center_offset,
            key.iy as f64 * width - center_offset,
            key.iz as f64 * width - center_offset,
        );
        let entity = world.spawn(PartitionBox {
            key,
            level: i,
            lo,
            hi: lo + DVec3::splat(width),
            parent,
            children: BTreeMap::new(),
            balls: BTreeMap::new(),
            statics: BTreeMap::new(),
            bubble: -1,
            handled: false,
        }).id();
        self.levels[i].insert(key, entity);
        if let Some(parent) = parent {
            debug_assert!(Self::add_child(world, parent, entity), "new Box must attach to parent");
        }
        Some(entity)
    }

    /// Box::AddChildren. No pointer-order sorting or silent key replacement.
    ///
    /// Only accepts the real parent of a directly subordinate spatial cell;
    /// arbitrary reparenting would corrupt the Partition hierarchy.
    pub fn add_child(world: &mut World, parent: Entity, child: Entity) -> bool {
        let (child_key, child_level, child_parent) = match world.get::<PartitionBox>(child) {
            Some(b) => (b.key, b.level, b.parent),
            None => return false,
        };
        let Some(p) = world.get::<PartitionBox>(parent) else { return false };
        if child_parent != Some(parent)
            || child_level != p.level + 1
            || (child_key.ix >> 2, child_key.iy >> 2, child_key.iz >> 2)
                != (p.key.ix, p.key.iy, p.key.iz)
        {
            return false;
        }
        use std::collections::btree_map::Entry;
        match world.get_mut::<PartitionBox>(parent).unwrap().children.entry(child_key) {
            Entry::Vacant(slot) => { slot.insert(child); true }
            Entry::Occupied(_) => false,
        }
    }

    #[must_use]
    pub fn choose_level(&self, radius: f64) -> usize {
        let mut level = 0;
        for i in 0..self.number_of_levels() {
            if 3.0 * radius > self.level_width[i] {
                break;
            }
            level = i;
        }
        level
    }

    pub fn box_for_radius(
        &mut self, world: &mut World, radius: f64, position: DVec3, create: bool,
    ) -> Option<Entity> {
        let level = self.choose_level(radius) as i32;
        let (x,y,z) = self.get_box_key(position, level);
        if create {
            self.get_box(world,x,y,z,level)
        } else {
            self.check_box(x,y,z,level)
        }
    }

    #[must_use]
    pub fn existing_top_box(&self, position: DVec3) -> Option<Entity> {
        let level = self.choose_level(10_000_000.0) as i32;
        let (x,y,z) = self.get_box_key(position, level);
        self.check_box(x,y,z,level)
    }

    #[must_use]
    pub fn get_box_center(&self, level: i32, position: DVec3) -> Option<DVec3> {
        let i = self.valid_level(level)?;
        let width = self.level_width[i];
        let (x,y,z) = self.get_box_key(position, level);
        let offset = width * 0.5 - width * self.level_grid[i] as f64 * 0.5;
        Some(DVec3::new(
            x as f64 * width + offset,
            y as f64 * width + offset,
            z as f64 * width + offset,
        ))
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.levels.iter().map(BTreeMap::len).sum()
    }

    #[must_use]
    pub fn topmost(world: &World, box_entity: Option<Entity>) -> Option<Entity> {
        let mut entity = box_entity?;
        while let Some(parent) = world.get::<PartitionBox>(entity)?.parent {
            entity = parent;
        }
        Some(entity)
    }

    #[must_use]
    pub fn check_offset_box(
        &self, world: &World, box_entity: Option<Entity>, offset: (i64,i64,i64),
    ) -> Option<Entity> {
        let b = world.get::<PartitionBox>(box_entity?)?;
        let x = b.key.ix.checked_add(offset.0)?;
        let y = b.key.iy.checked_add(offset.1)?;
        let z = b.key.iz.checked_add(offset.2)?;
        self.check_box(x,y,z,b.level as i32)
    }

    pub fn get_offset_box(
        &mut self, world: &mut World, box_entity: Option<Entity>, offset: (i64,i64,i64),
    ) -> Option<Entity> {
        let b = world.get::<PartitionBox>(box_entity?)?;
        let (k,l) = (b.key,b.level);
        self.get_box(
            world,
            k.ix.checked_add(offset.0)?,
            k.iy.checked_add(offset.1)?,
            k.iz.checked_add(offset.2)?,
            l as i32,
        )
    }

    pub fn clear_traversal_flag(&self, world: &mut World) {
        for entity in self.levels[0].values().copied() {
            if let Some(mut b) = world.get_mut::<PartitionBox>(entity) {
                b.bubble = -1;
            }
        }
    }

    /// Bevy-safe iterative equivalent of Box::Traverse (26 connected neighbors).
    pub fn traverse(&self, world: &mut World, start: Entity, seed: i32) {
        let mut todo = vec![start];
        while let Some(entity) = todo.pop() {
            let Some(b) = world.get::<PartitionBox>(entity) else { continue };
            if b.bubble != -1 { continue; }
            let (key,level) = (b.key,b.level as i32);
            world.get_mut::<PartitionBox>(entity).unwrap().bubble = seed;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 { continue; }
                        if let (Some(x),Some(y),Some(z)) = (
                            key.ix.checked_add(dx), key.iy.checked_add(dy), key.iz.checked_add(dz)
                        ) {
                            if let Some(other) = self.check_box(x,y,z,level) { todo.push(other); }
                        }
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn nearby_bubbles(&self, world: &World, entity: Entity) -> Option<i32> {
        world.get::<PartitionBox>(entity)?;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if x == 0 && y == 0 && z == 0 { continue; }
                    if let Some(neighbor) = self.check_offset_box(world, Some(entity),(x,y,z)) {
                        if let Some(found) = world.get::<PartitionBox>(neighbor) {
                            if found.bubble != -1 { return Some(found.bubble); }
                        }
                    }
                }
            }
        }
        None
    }

    /// Returns (box width, minimum corners), matching C++ GetActiveBoxes.
    /// C++ Py_BuildValue("f") converts to single precision before Python sees it.
    #[must_use]
    pub fn active_boxes(&self, world: &World, level: i32) -> Option<(f32,Vec<[f32;3]>)> {
        let i = self.valid_level(level)?;
        if self.levels[i].is_empty() { return None; }
        let corners = self.levels[i].values().filter_map(|entity| {
            world.get::<PartitionBox>(*entity).map(|b| [
                b.lo.x as f32, b.lo.y as f32, b.lo.z as f32,
            ])
        }).collect();
        Some((self.level_width[i] as f32,corners))
    }

    /// Human-readable substitute for CCP_LOG_CH DumpPartition.
    #[must_use]
    pub fn dump(&self, world: &World) -> Vec<String> {
        let mut result = Vec::new();
        for (level, boxes) in self.levels.iter().enumerate() {
            for (key, entity) in boxes {
                if let Some(b) = world.get::<PartitionBox>(*entity) {
                    result.push(format!(
                        "Level {level}: {} balls and {} children in Box [{},{},{}]",
                        b.balls.len(), b.children.len(), key.ix,key.iy,key.iz,
                    ));
                }
            }
        }
        result
    }

    /// Add a concrete Ball reference, including the reciprocal Partitionable link.
    /// A separate future Ball port owns physics and any cached active-box flags.
    pub fn insert_ball(&self, world: &mut World, box_entity: Entity, ball: Entity) -> bool {
        let Some(b) = world.get::<PartitionBox>(box_entity) else {return false};
        let key = b.key;
        let Some(p) = world.get::<Partitionable>(ball) else {return false};
        let Some(meta) = world.get::<PartitionBall>(ball) else {return false};
        let order = ball_order(p.id, meta.owner_id);
        let inserted = {
            use std::collections::btree_map::Entry;
            match world.get_mut::<PartitionBox>(box_entity).unwrap().balls.entry(order) {
                Entry::Vacant(slot) => { slot.insert(ball); true }
                Entry::Occupied(_) => false,
            }
        };
        if inserted {
            world.get_mut::<Partitionable>(ball).unwrap().track_box(key, box_entity);
        }
        inserted
    }

    /// Add a static collider reference with the C++ (parent ID, local ID) order.
    pub fn insert_static(&self, world: &mut World, box_entity: Entity, object: Entity) -> bool {
        let Some(b) = world.get::<PartitionBox>(box_entity) else {return false};
        let key=b.key;
        let Some(p)=world.get::<Partitionable>(object) else{return false};
        let Some(meta)=world.get::<PartitionStatic>(object) else{return false};
        let order=(meta.parent_ball_id,p.id);
        let inserted={
            use std::collections::btree_map::Entry;
            match world.get_mut::<PartitionBox>(box_entity).unwrap().statics.entry(order) {
                Entry::Vacant(slot) => { slot.insert(object); true }
                Entry::Occupied(_) => false,
            }
        };
        if inserted {
            world.get_mut::<Partitionable>(object).unwrap().track_box(key,box_entity);
        }
        inserted
    }

    /// Like Box::RemoveBall / RemoveStaticCollidable, prune an empty box and
    /// then its empty ancestors. Live collidables must be detached before despawn.
    pub fn remove_ball(&mut self, world: &mut World, box_entity: Entity, ball: Entity) {
        if let Some(mut b)=world.get_mut::<PartitionBox>(box_entity) {
            b.balls.retain(|_,e| *e != ball);
            let key=b.key;
            if let Some(mut p)=world.get_mut::<Partitionable>(ball) {p.untrack_box(&key);}
        }
        self.prune_empty(world,box_entity);
    }
    pub fn remove_static(&mut self, world: &mut World, box_entity: Entity, object: Entity) {
        if let Some(mut b)=world.get_mut::<PartitionBox>(box_entity) {
            b.statics.retain(|_,e| *e != object);
            let key=b.key;
            if let Some(mut p)=world.get_mut::<Partitionable>(object) {p.untrack_box(&key);}
        }
        self.prune_empty(world,box_entity);
    }
    /// C++ Box::ReleaseBox / destructor. A nonempty Box cannot be released:
    /// doing so would leave dangling child, Ball, and StaticCollidable links.
    ///
    /// Removing the final child/member recursively prunes empty ancestors,
    /// matching Box::RemoveChildren / RemoveBall / RemoveStaticCollidable.
    pub fn release_box(&mut self, world: &mut World, entity: Entity) -> bool {
        let Some(b) = world.get::<PartitionBox>(entity) else { return false };
        if !b.is_empty() || self.levels.get(b.level).and_then(|m| m.get(&b.key)) != Some(&entity) {
            return false;
        }
        self.prune_empty(world, entity);
        true
    }

    fn prune_empty(&mut self, world: &mut World, mut entity: Entity) {
        loop {
            let Some(b)=world.get::<PartitionBox>(entity) else {break};
            if !b.is_empty() {break;}
            let (key,level,parent)=(b.key,b.level,b.parent);
            self.levels[level].remove(&key);
            let _ = world.despawn(entity);
            let Some(p)=parent else {break};
            if let Some(mut parent_box)=world.get_mut::<PartitionBox>(p) {
                if parent_box.children.get(&key) == Some(&entity) {
                    parent_box.children.remove(&key);
                }
            }
            entity=p;
        }
    }

    /// Matches the C++ child-first recursion (without recursive stack growth).
    fn explode_children(&self, world: &World, current: Entity, out: &mut Vec<Entity>) {
        let Some(b)=world.get::<PartitionBox>(current) else {return};
        for child in b.children.values() {
            self.explode_children(world,*child,out);
        }
        if b.contains_collidables() {out.push(current);}
    }

    /// All candidate filters from Partition::GetNearbyBalls.
    /// The caller supplies Ballpark::GetInteractiveCnt(source) explicitly
    /// because Ballpark is not yet ported.
    #[must_use]
    pub fn nearby_candidates(
        &self, world: &World, source: Entity, is_master: bool,
        filter: NearbyCriteria, interactive_count: usize,
    ) -> NearbyCandidates {
        let mut result=NearbyCandidates::default();
        let (Some(source_p),Some(source_b))=(
            world.get::<Partitionable>(source),
            world.get::<PartitionBall>(source),
        ) else {return result};
        if filter.exclude_inert_balls && interactive_count==0 {return result;}
        let mut todo=Vec::new();
        let mut seen_boxes=BTreeSet::new();
        for (_,starting) in source_p.boxes() {
            if world.get::<PartitionBox>(starting).is_none() {continue;}
            let mut ancestor=world.get::<PartitionBox>(starting).and_then(|b| b.parent);
            while let Some(parent)=ancestor {
                let Some(b)=world.get::<PartitionBox>(parent) else {break};
                if b.contains_collidables() && seen_boxes.insert(parent) {
                    todo.push(parent);
                }
                ancestor=b.parent;
            }
            let Some(b)=world.get::<PartitionBox>(starting) else {continue};
            for child in b.children.values().copied() {
                let mut exploded=Vec::new();
                self.explode_children(world,child,&mut exploded);
                for entity in exploded {
                    if seen_boxes.insert(entity) {todo.push(entity);}
                }
            }
            if (b.balls.len()>1 || !b.statics.is_empty()) && seen_boxes.insert(starting) {
                todo.push(starting);
            }
        }
        let mut seen_balls=BTreeSet::new();
        let mut seen_statics=BTreeSet::new();
        for box_entity in todo {
            let Some(b)=world.get::<PartitionBox>(box_entity) else {continue};
            for other in b.balls.values().copied() {
                if !seen_balls.insert(other) || other==source {continue;}
                let (Some(p),Some(info))=(
                    world.get::<Partitionable>(other),
                    world.get::<PartitionBall>(other),
                ) else {continue};
                if p.is_moribund {continue;}
                if filter.exclude_cloaked && (info.cloaked || !info.massive) {continue;}
                if filter.exclude_inert_balls {
                    if !p.is_interactive {continue;}
                } else {
                    if is_master && p.new_bubble != source_p.new_bubble && !info.global {continue;}
                    if filter.exclude_fixed_balls && !info.free {continue;}
                    if info.mode == BallMode::Field && !filter.exclude_harmonics_test {
                        if source_b.harmonic == -2 {continue;}
                        if info.harmonic != -1 && info.harmonic == source_b.harmonic {continue;}
                        if info.corporation_id != -1 && info.corporation_id == source_b.corporation_id {continue;}
                        if info.alliance_id != -1 && info.alliance_id == source_b.alliance_id {continue;}
                    }
                    if filter.exclude_missiles && info.mode == BallMode::Missile {continue;}
                }
                result.balls.push(other);
            }
            if !filter.exclude_fixed_balls {
                for other in b.statics.values().copied() {
                    if !seen_statics.insert(other) {continue;}
                    let Some(p)=world.get::<Partitionable>(other) else {continue};
                    if is_master && p.new_bubble != source_p.new_bubble {continue;}
                    result.static_collidables.push(other);
                }
            }
        }
        result
    }

    #[must_use]
    pub fn proximity_candidates(
        &self,world:&World,source:Entity,is_master:bool,interactive_count:usize,
    ) -> NearbyCandidates {
        let (Some(p),Some(b))=(
            world.get::<Partitionable>(source),world.get::<PartitionBall>(source)
        ) else {return NearbyCandidates::default()};
        let filter=NearbyCriteria {
            exclude_harmonics_test:true,
            exclude_fixed_balls:!b.cloaked,
            exclude_inert_balls:b.proximity_only_interactives,
            ..Default::default()
        };
        let _=p;
        self.nearby_candidates(world,source,is_master,filter,interactive_count)
    }

    #[must_use]
    pub fn collision_candidates(&self,world:&World,source:Entity,is_master:bool) -> NearbyCandidates {
        let (Some(p),Some(meta))=(
            world.get::<Partitionable>(source),world.get::<PartitionBall>(source)
        ) else {return NearbyCandidates::default()};
        if meta.mode == BallMode::Missile {
            if meta.owner_id < 0 {
                self.nearby_candidates(world,source,is_master,NearbyCriteria::CASE_3,1)
            } else {
                let mut found=NearbyCandidates::default();
                if let Some(target)=meta.followed {
                    if world.get::<Partitionable>(target).is_some_and(|t|t.new_bubble==p.new_bubble) {
                        found.balls.push(target);
                    }
                }
                found
            }
        } else {
            self.nearby_candidates(world,source,is_master,NearbyCriteria::CASE_1,1)
        }
    }
}

/// Same flag meanings and preconfigured cases as C++ NearbyCriteria.
/// Important: C++ collisionCase3 excludes cloaked/nonmassive, NOT missiles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NearbyCriteria {
    pub exclude_fixed_balls: bool,
    pub exclude_inert_balls: bool,
    pub exclude_missiles: bool,
    pub exclude_cloaked: bool,
    pub exclude_harmonics_test: bool,
}
impl NearbyCriteria {
    pub const CASE_1: Self=Self {
        exclude_missiles:true,exclude_cloaked:true,
        exclude_fixed_balls:false,exclude_inert_balls:false,exclude_harmonics_test:false,
    };
    pub const CASE_3: Self=Self {
        exclude_cloaked:true,exclude_missiles:false,
        exclude_fixed_balls:false,exclude_inert_balls:false,exclude_harmonics_test:false,
    };
}
#[derive(Debug, Default, PartialEq, Eq)]
pub struct NearbyCandidates {
    pub balls: Vec<Entity>,
    pub static_collidables: Vec<Entity>,
}
fn ball_order(id:i64,owner:i64)->BallSortKey {
    if id<0 {(0,owner,id)} else {(1,0,id)}
}

/// Use Resource Scope because Bevy World must own both the index and its boxes.
pub fn get_or_create_box(
    world:&mut World,ix:i64,iy:i64,iz:i64,level:i32,
)->Option<Entity> {
    world.resource_scope(|world,mut part:bevy::prelude::Mut<'_,Partition>|{
        part.get_box(world,ix,iy,iz,level)
    })
}

/// Original Partition_Blue.cpp exposes only the GetBoxCenter module function.
/// The original constructs a new Partition and raises TypeError for invalid levels.
#[blue_function(
    module="destiny",
    name="GetBoxCenter",
    doc="Gives you back a partition centered coordinate"
)]
pub fn py_get_box_center(level:i32,x:f64,y:f64,z:f64)->PyResult<(f64,f64,f64)> {
    Partition::default().get_box_center(level,DVec3::new(x,y,z))
        .map(|v|(v.x,v.y,v.z))
        .ok_or_else(||PyTypeError::new_err("Illegal level"))
}

pub fn register_partition_python(module:&Bound<'_,pyo3::types::PyModule>)->PyResult<()> {
    BlueModule::new(module).function(PY_GET_BOX_CENTER_BLUE_FUNCTION)?;
    Ok(())
}

pub struct PartitionPlugin;
impl Plugin for PartitionPlugin {
    fn build(&self,app:&mut App) {
        app.init_resource::<Partition>()
            .register_blue_function(PY_GET_BOX_CENTER_BLUE_FUNCTION);
    }
}
