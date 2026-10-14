// Copyright © 2015, 2023 CCP ehf. Original Destiny MIT license retained.
// Reference accounting derived from Carbon BlueExposure 2.1.1, CCP Games MIT.
//! Source RootRefLock/BlueClasses accounting and Destiny's concrete ownership
//! transitions, at the pinned behavioral/dependency revisions.
//!
//! Six tested classes, sixteen objects and four owned descriptors per object
//! bound these examples. Counts are computed from allocations and actual lock
//! releases, never set to expected test results. Registration/fresh global
//! interpreter state, atomic concurrency, native pointers, wrapper caches and
//! Python aliases remain executable correspondence obligations. This source
//! projection is specification evidence; no concrete runtime claim is made.

use crate::{BoundedList, OriginalBallInit, OriginalParkError, OriginalParkProjection, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalObjectClass {
    Ball,
    Park,
    ClientBall,
    MiniBall,
    MiniBox,
    MiniCapsule,
}
impl OriginalObjectClass {
    fn index(self) -> usize {
        match self {
            Self::Ball => 0,
            Self::Park => 1,
            Self::ClientBall => 2,
            Self::MiniBall => 3,
            Self::MiniBox => 4,
            Self::MiniCapsule => 5,
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct Object {
    class: OriginalObjectClass,
    locks: i32,
    children: BoundedList<usize, 4>,
}
#[derive(Default)]
struct ObjectRegistry {
    registered: [bool; 6],
    live: [u32; 6],
    objects: [Option<Object>; 16],
}
impl ObjectRegistry {
    fn allocate(&mut self, class: OriginalObjectClass) -> Result<usize, OriginalParkError> {
        if !self.registered[class.index()] {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut index = 0;
        while index < 16 {
            if self.objects[index].is_none() {
                break;
            }
            index += 1;
        }
        if index == 16 {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let count = self.live[class.index()]
            .checked_add(1)
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        self.live[class.index()] = count;
        // RootRefLock constructor InstIncrement(1), then Lock(): one initial
        // owning reference. UpdateObjectCount uses the final ClassType only.
        self.objects[index] = Some(Object {
            class,
            locks: 1,
            children: BoundedList::new(),
        });
        Ok(index)
    }
    fn lock(&mut self, index: usize) -> Result<(), OriginalParkError> {
        let mut object = self
            .objects
            .get(index)
            .copied()
            .flatten()
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        object.locks = object
            .locks
            .checked_add(1)
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        self.objects[index] = Some(object);
        Ok(())
    }
    fn unlock(&mut self, index: usize, depth: usize) -> Result<(), OriginalParkError> {
        if depth > 16 {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut object = self
            .objects
            .get(index)
            .copied()
            .flatten()
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        if object.locks <= 0 {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        object.locks -= 1;
        if object.locks == 0 {
            self.objects[index] = None;
            let mut i = 0;
            while i < object.children.len() {
                self.unlock(
                    object
                        .children
                        .get(i)
                        .ok_or(OriginalParkError::OutsideDeclaredScope)?,
                    depth + 1,
                )?;
                i += 1;
            }
            self.live[object.class.index()] = self.live[object.class.index()]
                .checked_sub(1)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        } else {
            self.objects[index] = Some(object);
        }
        Ok(())
    }
    fn add_child(&mut self, parent: usize, child: usize) -> Result<(), OriginalParkError> {
        let mut object = self
            .objects
            .get(parent)
            .copied()
            .flatten()
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        self.lock(child)?; // BlueList.Insert takes its own reference
        if !object.children.push(child) {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        self.objects[parent] = Some(object);
        self.unlock(child, 0) // original descriptor creator releases its initial reference
    }
    fn remove_children(&mut self, parent: usize) -> Result<(), OriginalParkError> {
        let mut object = self
            .objects
            .get(parent)
            .copied()
            .flatten()
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        let children = object.children;
        object.children = BoundedList::new();
        self.objects[parent] = Some(object);
        let mut i = 0;
        while i < children.len() {
            self.unlock(
                children
                    .get(i)
                    .ok_or(OriginalParkError::OutsideDeclaredScope)?,
                0,
            )?;
            i += 1;
        }
        Ok(())
    }
    fn live_count(&self, class: OriginalObjectClass) -> Option<i32> {
        if !self.registered[class.index()] {
            return None;
        }
        // PyLiveCount assigns its uint32 ClassInfo counter to C int. These
        // concrete counts are at most sixteen, so both conversions agree.
        Some(self.live[class.index()] as i32)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalBallLease {
    pub id: i64,
    object: usize,
}

#[derive(Default)]
pub struct OriginalLifetimeProjection {
    registry: ObjectRegistry,
    park: Option<Box<OriginalParkProjection>>,
    park_object: Option<usize>,
    park_owned_balls: BoundedList<OriginalBallLease, 16>,
}
impl OriginalLifetimeProjection {
    /// Source module registration publishes all six ClassInfo entries. This
    /// executes the typed registration relation; actual native plugin load and
    /// the initial global counts still need independent runtime evidence.
    pub fn register_destiny_classes(&mut self) {
        self.registry.registered = [true; 6];
    }
    #[must_use]
    pub fn live_count(&self, class: OriginalObjectClass) -> Option<i32> {
        self.registry.live_count(class)
    }
    #[must_use]
    pub fn has_park(&self) -> bool {
        self.park.is_some() && self.park_object.is_some()
    }
    #[must_use]
    pub fn native_class(&self, ball: OriginalBallLease) -> Option<OriginalObjectClass> {
        self.registry
            .objects
            .get(ball.object)
            .copied()
            .flatten()
            .map(|object| object.class)
    }
    /// Native class construction and returned owning wrapper, represented by a
    /// typed object lease. Actual Blue factory dispatch/NULL/exception paths
    /// and interpreter objects remain required boundary correspondence.
    pub fn create_detached_ball(
        &mut self,
        class: OriginalObjectClass,
    ) -> Result<OriginalBallLease, OriginalParkError> {
        if class != OriginalObjectClass::Ball && class != OriginalObjectClass::ClientBall {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let object = self.registry.allocate(class)?;
        self.registry.lock(object)?;
        self.registry.unlock(object, 0)?;
        Ok(OriginalBallLease { id: 0, object })
    }
    pub fn create_park(&mut self, master: bool) -> Result<(), OriginalParkError> {
        if self.park.is_some() {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let object = self.registry.allocate(OriginalObjectClass::Park)?;
        self.registry.lock(object)?; // Python wrapper's owning reference
        self.registry.unlock(object, 0)?; // factory releases its initial reference
        let mut park = Box::new(OriginalParkProjection::default());
        park.is_master = master;
        self.park = Some(park);
        self.park_object = Some(object);
        Ok(())
    }
    fn park_mut(&mut self) -> Result<&mut OriginalParkProjection, OriginalParkError> {
        self.park
            .as_deref_mut()
            .ok_or(OriginalParkError::OutsideDeclaredScope)
    }
    fn sync_park_ownership(&mut self) -> Result<(), OriginalParkError> {
        let park = self
            .park
            .as_deref()
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        let mut retained = BoundedList::new();
        let mut i = 0;
        while i < self.park_owned_balls.len() {
            let lease = self
                .park_owned_balls
                .get(i)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
            if park.contains(lease.id) {
                if !retained.push(lease) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            } else {
                self.registry.unlock(lease.object, 0)?;
            }
            i += 1;
        }
        self.park_owned_balls = retained;
        Ok(())
    }
    pub fn add_cleanup_ball(&mut self) -> Result<OriginalBallLease, OriginalParkError> {
        self.add_ball(OriginalBallInit {
            id: -1,
            mass: 0.0,
            radius: 500.0,
            max_velocity: 0.0,
            is_free: false,
            is_global: false,
            is_massive: false,
            is_interactive: false,
            is_space_junk: false,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            agility: 0.0,
            speed_fraction: 0.0,
        })
    }
    pub fn add_ball(
        &mut self,
        init: OriginalBallInit,
    ) -> Result<OriginalBallLease, OriginalParkError> {
        let park = self.park_mut()?;
        let class = if park.is_master {
            OriginalObjectClass::Ball
        } else {
            OriginalObjectClass::ClientBall
        };
        let id = park.add_original_ball_checked(init)?;
        let object = self.registry.allocate(class)?;
        self.registry.lock(object)?; // MapOfBalls owns the native ball
        self.registry.unlock(object, 0)?; // native AddBall's creator reference
        self.registry.lock(object)?; // BlueWrapObjectForPython retains the returned ball
        let lease = OriginalBallLease { id, object };
        if !self.park_owned_balls.push(lease) {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        Ok(lease)
    }
    pub fn add_mini_ball(&mut self, ball: OriginalBallLease) -> Result<(), OriginalParkError> {
        self.park_mut()?.add_mini_ball(ball.id, Vec3::ZERO, 10.0)?;
        let child = self.registry.allocate(OriginalObjectClass::MiniBall)?;
        self.registry.add_child(ball.object, child)?;
        // A non-free parent also adds a physical native Ball/ClientBall.
        let park = self
            .park
            .as_deref()
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        let parent = park.retained(ball.id);
        if !parent.is_free {
            let id = parent
                .mini_balls
                .get(
                    parent
                        .mini_balls
                        .len()
                        .checked_sub(1)
                        .ok_or(OriginalParkError::OutsideDeclaredScope)?,
                )
                .ok_or(OriginalParkError::OutsideDeclaredScope)?
                .id;
            let class = if park.is_master {
                OriginalObjectClass::Ball
            } else {
                OriginalObjectClass::ClientBall
            };
            let object = self.registry.allocate(class)?;
            self.registry.lock(object)?;
            self.registry.unlock(object, 0)?; // native park only, no Python handle
            if !self.park_owned_balls.push(OriginalBallLease { id, object }) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
        }
        Ok(())
    }
    pub fn add_mini_capsule(&mut self, ball: OriginalBallLease) -> Result<(), OriginalParkError> {
        self.park_mut()?.add_mini_capsule(
            ball.id,
            Vec3::ZERO,
            Vec3 {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            1.0,
        )?;
        let child = self.registry.allocate(OriginalObjectClass::MiniCapsule)?;
        self.registry.add_child(ball.object, child)
    }
    pub fn add_mini_box(&mut self, ball: OriginalBallLease) -> Result<(), OriginalParkError> {
        self.park_mut()?.add_mini_box(
            ball.id,
            Vec3 {
                x: -5.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 10.0,
            },
        )?;
        let child = self.registry.allocate(OriginalObjectClass::MiniBox)?;
        self.registry.add_child(ball.object, child)
    }
    pub fn remove_ball(&mut self, id: i64, delay: i32) -> Result<(), OriginalParkError> {
        let mut lease = None;
        let mut i = 0;
        while i < self.park_owned_balls.len() {
            if let Some(item) = self.park_owned_balls.get(i) {
                if item.id == id {
                    lease = Some(item);
                    break;
                }
            }
            i += 1;
        }
        let lease = lease.ok_or(OriginalParkError::OutsideDeclaredScope)?;
        let free = self
            .park
            .as_deref()
            .ok_or(OriginalParkError::OutsideDeclaredScope)?
            .retained(id)
            .is_free;
        self.park_mut()?.remove_ball(id, delay);
        // Source clears descriptor lists for a fixed parent before deciding
        // whether to retain a moribund ball for a delayed removal.
        if !free {
            self.registry.remove_children(lease.object)?;
        }
        self.sync_park_ownership()
    }
    pub fn evolve(&mut self) -> Result<(), OriginalParkError> {
        self.park_mut()?.evolve_two_spherical_bodies()?;
        self.sync_park_ownership()
    }
    pub fn clear_all(&mut self) -> Result<(), OriginalParkError> {
        self.park_mut()?.clear_all();
        self.sync_park_ownership()
    }
    pub fn drop_ball_reference(
        &mut self,
        ball: OriginalBallLease,
    ) -> Result<(), OriginalParkError> {
        self.registry.unlock(ball.object, 0)
    }
    pub fn drop_park_reference(&mut self) -> Result<(), OriginalParkError> {
        self.clear_all()?;
        let object = self
            .park_object
            .take()
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        self.park = None;
        self.registry.unlock(object, 0)
    }
    pub fn finish_scope(&mut self) -> Result<(), OriginalParkError> {
        if self.park.is_some() {
            self.drop_park_reference()?;
        }
        Ok(())
    }
}
