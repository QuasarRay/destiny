// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
// @generated: source partition state rules and unchanged numeric kernels.
// Bindings: verification/migration/partition_source_evidence.json; partition_rules.json.
use destiny_original_spec::{
    OriginalParkBall, OriginalPartitionCellKey as CellKey, Vec3,
    original_partition_box_snapshot as box_snapshot,
};
use serde::{Deserialize, Serialize};

#[derive(bevy::prelude::Resource, Default)]
pub(crate) struct CollisionContext {
    pub is_master: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Address {
    pub key: [i64; 3],
    pub level: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoxRecord {
    pub address: Address,
    pub bubble: i64,
    pub object: Option<serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub id: i64,
    pub primary: Address,
    pub modifiers: [i64; 3],
    pub owned: Vec<Address>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Partition {
    pub boxes: Vec<BoxRecord>,
    pub placements: Vec<Placement>,
    pub next_bubble: i64,
    #[serde(default)]
    pub undefined_coordinates: Vec<i64>,
}
#[derive(Debug, Clone, Copy)]
pub struct PreparedInsertion {
    pub label: Option<i64>,
    pub transition: Option<i64>,
}
pub fn inflated_radius(b: &OriginalParkBall, friction: f64, dt: f64) -> f64 {
    if b.is_free {
        let m = b.mass * f64::from(b.agility);
        let a = friction * f64::from(b.speed_fraction) * f64::from(b.max_velocity) / m;
        let v = b.velocity.length();
        f64::from(b.radius) + dt * ((m * a - (m * a - v * friction) * b.time_factor) / friction)
    } else {
        f64::from(b.radius)
    }
}
impl Address {
    #[doc = "src/Partition.h:66-102"]
    pub fn from_cell(key: CellKey, level: u32) -> Address {
        Address {
            key: [key.x, key.y, key.z],
            level,
        }
    }
}
impl Address {
    #[doc = "src/Partition.h:66-102"]
    pub fn cell(self) -> CellKey {
        CellKey {
            x: self.key[0_usize],
            y: self.key[1_usize],
            z: self.key[2_usize],
        }
    }
}
impl Address {
    #[doc = "src/Box.cpp:98-108"]
    pub fn parent(self) -> Option<Address> {
        if (self.level) == (0_u32) {
            return None;
        }
        Some(Address::from_cell(
            ((self).cell()).parent(),
            (self.level) - (1_u32),
        ))
    }
}
impl Address {
    #[doc = "src/Partition.cpp:183-198"]
    pub fn top(self) -> Address {
        let mut current = self;
        for _level in 0_u32..self.level {
            current = ((current).parent()).expect("source level has a parent");
        }
        current
    }
}
impl Address {
    #[doc = "src/Partition.cpp:200-222, src/Partition.h:66-102"]
    pub fn offset(self, x: i64, y: i64, z: i64) -> Address {
        let grid = (1_i64) << ((23_u32) + ((2_u32) * (self.level)));
        Address::from_cell(((self).cell()).offset(x, y, z, grid), self.level)
    }
}
impl Partition {
    #[doc = "src/Partition.cpp:64-85, src/Box.cpp:15-45, src/Box.cpp:98-108"]
    pub fn ensure(&mut self, address: Address) {
        if ((self.boxes).iter()).any(|entry| (entry.address) == (address)) {
            return;
        }
        if let Some(parent) = (address).parent() {
            (self).ensure(parent);
        }
        (self.boxes).push(BoxRecord {
            address,
            bubble: -1_i64,
            object: None,
        });
    }
}
impl Partition {
    #[doc = "src/Box.cpp:50-61, src/Box.cpp:144-185"]
    pub fn remove_empty(&mut self, address: Address) {
        let Some(index) = ((self.boxes).iter()).position(|entry| (entry.address) == (address))
        else {
            return;
        };
        if ((self.placements).iter()).any(|placement| (placement.owned).contains(&address)) {
            return;
        }
        if ((self.boxes).iter()).any(|entry| ((entry.address).parent()) == (Some(address))) {
            return;
        }
        (self.boxes).remove(index);
        if let Some(parent) = (address).parent() {
            (self).remove_empty(parent);
        }
    }
}
impl Partition {
    #[doc = "src/Ball.cpp:812-826, src/Box.cpp:168-185"]
    pub fn remove(&mut self, id: i64) {
        (self.undefined_coordinates).retain(|candidate| (*(candidate)) != (id));
        let Some(index) = ((self.placements).iter()).position(|placement| (placement.id) == (id))
        else {
            return;
        };
        let previous = (self.placements).remove(index);
        for address in previous.owned {
            (self).remove_empty(address);
        }
    }
}
impl Partition {
    #[doc = "src/Ballpark.cpp:3115-3187, src/Ball.cpp:2355-2536, src/Box.cpp:278-282"]
    pub fn reinsert(
        &mut self,
        id: i64,
        primary: Address,
        modifiers: [i64; 3usize],
        owned: Vec<Address>,
    ) -> bool {
        (self).ensure(primary);
        if let Some(previous) = ((self.placements).iter()).find(|placement| (placement.id) == (id))
            && (((previous.primary) == (primary)) && ((previous.modifiers) == (modifiers)))
        {
            return false;
        }
        (self).remove(id);
        (self).ensure(primary);
        let mut owned = owned;
        (owned).sort();
        (owned).dedup();
        for address in &owned {
            (self).ensure(*(address));
        }
        (self.placements).push(Placement {
            id,
            primary,
            modifiers,
            owned,
        });
        true
    }
}
impl Partition {
    #[doc = "src/Thunkers.cpp:2260-2300"]
    pub fn ball_box(&self, id: i64) -> Option<(f64, (f64, f64, f64))> {
        let placement = ((self.placements).iter()).find(|placement| (placement.id) == (id))?;
        box_snapshot((placement.primary).cell(), placement.primary.level)
    }
}
impl Partition {
    #[doc = "src/Thunkers.cpp:2012-2047"]
    pub fn bubble(&self, address: Address) -> Option<i64> {
        (((self.boxes).iter()).find(|entry| (entry.address) == ((address).top())))
            .map(|entry| entry.bubble)
    }
}
impl Partition {
    #[doc = "src/Ball.cpp:2425-2430, src/Ballpark.cpp:3155-3157"]
    pub fn set_bubble(&mut self, address: Address, bubble: i64) {
        if let Some(entry) =
            ((self.boxes).iter_mut()).find(|entry| (entry.address) == ((address).top()))
        {
            entry.bubble = bubble;
        }
    }
}
impl Partition {
    #[doc = "src/Box.cpp:252-275"]
    pub fn nearby_bubble(&self, address: Address) -> i64 {
        for x in -1_i64..2_i64 {
            for y in -1_i64..2_i64 {
                for z in -1_i64..2_i64 {
                    if ((x) != (0_i64)) || (((y) != (0_i64)) || ((z) != (0_i64))) {
                        let neighbor = (address).offset(x, y, z);
                        if let Some(bubble) = (self).bubble(neighbor)
                            && ((bubble) != (-1_i64))
                        {
                            return bubble;
                        }
                    }
                }
            }
        }
        -1_i64
    }
}
impl Partition {
    #[doc = "src/Ballpark.cpp:3115-3187"]
    pub fn prepare_master(&mut self, id: i64, primary: Address) -> i64 {
        (self).ensure(primary);
        let top = (primary).top();
        let mut bubble = ((self).bubble(top)).unwrap_or(-1_i64);
        if (bubble) == (-1_i64) {
            (self).remove(id);
            bubble = (self).nearby_bubble(top);
            if (bubble) == (-1_i64) {
                bubble = self.next_bubble;
                self.next_bubble += 1_i64;
            }
            (self).set_bubble(top, bubble);
        }
        bubble
    }
}
impl Partition {
    #[doc = "src/Partition.cpp:224-237, src/Ballpark.cpp:6565-6622"]
    pub fn unlabeled_top_cells(&self) -> Vec<(CellKey, i64)> {
        let mut cells = Vec::new();
        for entry in &self.boxes {
            if (entry.address.level) == (0_u32) {
                (cells).push(((entry.address).cell(), -1_i64));
            }
        }
        cells
    }
}
impl Partition {
    #[doc = "src/Partition.cpp:224-237, src/Ballpark.cpp:6565-6622"]
    pub fn apply_top_labels(&mut self, cells: Vec<(CellKey, i64)>, next: i64) {
        for entry in &mut self.boxes {
            entry.bubble = -1_i64;
        }
        for entry in cells {
            (self).set_bubble(Address::from_cell(entry.0, 0_u32), entry.1);
        }
        self.next_bubble = next;
    }
}
impl Partition {
    #[doc = "src/Ballpark.cpp:3115-3187"]
    pub fn prepare_insert(
        &mut self,
        id: i64,
        primary: Address,
        master: bool,
        current: i64,
    ) -> PreparedInsertion {
        if !(master) {
            return PreparedInsertion {
                label: None,
                transition: None,
            };
        }
        let label = (self).prepare_master(id, primary);
        let mut transition = None;
        if (label) != (current) {
            transition = Some(label);
        }
        PreparedInsertion {
            label: Some(label),
            transition,
        }
    }
}
impl Partition {
    #[doc = "src/Ball.cpp:2355-2536, src/Ballpark.cpp:3115-3187"]
    pub fn finish_insert(
        &mut self,
        id: i64,
        primary: Address,
        modifiers: [i64; 3usize],
        owned: Vec<Address>,
        prepared: PreparedInsertion,
    ) {
        (self).reinsert(id, primary, modifiers, owned);
        if let Some(label) = prepared.label {
            (self).set_bubble(primary, label);
        }
    }
}
#[doc = "src/Ballpark.cpp:599-612"]
pub fn moved_for_partition(previous: Vec3, current: Vec3) -> bool {
    ((((current).sub(previous)).norm_squared()).partial_cmp(&0.1_f64))
        != (Some(std::cmp::Ordering::Less))
}
#[doc = "src/Ballpark.cpp:4880-4933"]
pub fn became_fixed(previous: bool, current: bool) -> bool {
    (previous) && (!(current))
}
#[doc = "src/Ballpark.cpp:429-615"]
pub fn tick_reinsert(
    previous: Vec3,
    current: Vec3,
    free: bool,
    pending: bool,
    mushroom: bool,
) -> bool {
    (free) && ((!(pending)) && ((!(mushroom)) && (moved_for_partition(previous, current))))
}
impl Partition {
    #[doc = "src/Partition.h:66-102, src/Box.cpp:15-61, src/Ball.cpp:812-835, logical source_partition_v1 representation invariants"]
    pub fn valid_for_balls(&self, ids: &std::collections::HashSet<i64>) -> bool {
        if (self.next_bubble) < (0_i64) {
            return false;
        }
        let mut addresses = std::collections::BTreeSet::new();
        for entry in &self.boxes {
            if (entry.address.level) >= (8_u32) {
                return false;
            }
            if !((addresses).insert(entry.address)) {
                return false;
            }
            for coordinate in &entry.address.key {
                if ((*(coordinate)) < (0_i64))
                    || ((*(coordinate))
                        >= ((1_i64) << ((23_u32) + ((2_u32) * (entry.address.level)))))
                {
                    return false;
                }
            }
            if ((entry.bubble) < (-1_i64))
                || (((entry.bubble) >= (0_i64)) && ((entry.bubble) >= (self.next_bubble)))
            {
                return false;
            }
        }
        for entry in &self.boxes {
            if let Some(parent) = (entry.address).parent()
                && (!((addresses).contains(&parent)))
            {
                return false;
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for placement in &self.placements {
            if !((ids).contains(&placement.id)) {
                return false;
            }
            if !((seen).insert(placement.id)) {
                return false;
            }
            if !((placement.owned).contains(&placement.primary)) {
                return false;
            }
            let mut owned_seen = std::collections::BTreeSet::new();
            for address in &placement.owned {
                if (address.level) != (placement.primary.level) {
                    return false;
                }
                if !((addresses).contains(address)) {
                    return false;
                }
                if !((owned_seen).insert(address)) {
                    return false;
                }
            }
            for modifier in &placement.modifiers {
                if ((*(modifier)) < (-1_i64)) || ((*(modifier)) > (1_i64)) {
                    return false;
                }
            }
        }
        for id in &self.undefined_coordinates {
            if (!((ids).contains(id))) || (!((seen).insert(*(id)))) {
                return false;
            }
        }
        ((seen).len()) == ((ids).len())
    }
}
impl Partition {
    #[doc = "src/Ballpark.cpp:5896-5953"]
    pub fn clear_owners(&mut self) {
        (self.undefined_coordinates).clear();
        let mut ids: Vec<i64> =
            (((self.placements).iter()).map(|placement| placement.id)).collect();
        (ids).sort_unstable();
        for id in ids {
            (self).remove(id);
        }
    }
}
impl Partition {
    #[doc = "src/Partition.cpp:42-60,108-139 (undefined out-of-range float-to-i64); retained Rust finite-coordinate extension, no Original equivalence claimed"]
    pub fn unplace_undefined(&mut self, id: i64) {
        (self).remove(id);
        (self.undefined_coordinates).push(id);
    }
}
#[doc = "src/Partition.cpp:417-461; symmetric broad-phase admission for the retained Avian extension. Directed source kernel filters remain independent."]
pub fn source_pair_filter(
    is_master: bool,
    left_bubble: i64,
    right_bubble: i64,
    left_global: bool,
    right_global: bool,
) -> bool {
    (!(is_master)) || ((left_global) || ((right_global) || ((left_bubble) == (right_bubble))))
}
