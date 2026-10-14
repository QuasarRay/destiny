// Copyright © 2014-2023 CCP ehf. MIT license, retained from original Destiny.
//! Spherical-ball partition construction from Partition.cpp and Ball.cpp.
//!
//! This records every primary/adjacent top-level cell created by the source
//! insertion algorithm, including the 27-cell large-ball branch. Ancestor
//! keys use the original clamped grid and successive two-bit shifts. Connected
//! component labels implement Box::Traverse's 26-neighbour relation. Native
//! allocation, Python dictionary insertion order and pointer lifetime are
//! separate correspondence obligations. Static collidables are not represented
//! by this spherical partition; callers must reject that unimplemented scope.

use crate::original_bubbles::OriginalBubbleDictionary;
use crate::{BoundedList, Vec3};

pub(crate) const TOP_WIDTH: f64 = 7_864_320.0; // 480 * 4^7

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellKey {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl CellKey {
    pub fn parent(self) -> Self {
        Self {
            x: self.x >> 2,
            y: self.y >> 2,
            z: self.z >> 2,
        }
    }
    pub(crate) fn offset(self, x: i64, y: i64, z: i64, grid: i64) -> Self {
        Self {
            x: (self.x + x).clamp(0, grid - 1),
            y: (self.y + y).clamp(0, grid - 1),
            z: (self.z + z).clamp(0, grid - 1),
        }
    }
    fn adjacent(self, other: Self) -> bool {
        // These clamped nonnegative keys are at most 2^23-1 at level zero.
        (self.x - other.x).abs() <= 1
            && (self.y - other.y).abs() <= 1
            && (self.z - other.z).abs() <= 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PartitionCell {
    pub key: CellKey,
    pub bubble: i64,
}

pub(crate) struct OriginalBubbleProjection {
    pub cells: BoundedList<PartitionCell, 64>,
    pub dictionary: OriginalBubbleDictionary,
    pub next_bubble: i64,
}

impl Default for OriginalBubbleProjection {
    fn default() -> Self {
        Self {
            cells: BoundedList::new(),
            dictionary: OriginalBubbleDictionary::default(),
            next_bubble: 0,
        }
    }
}

/// Source double-to-int64 conversion is only defined within this checked
/// interval. Rust's saturating cast never conceals an out-of-domain C++ cast.
fn coordinate(value: f64, width: f64, grid: i64) -> Option<i64> {
    let scaled = (value + 0.5 * width * (grid as f64)) * (1.0 / width);
    if !scaled.is_finite()
        || scaled < -9_223_372_036_854_775_808.0
        || scaled >= 9_223_372_036_854_775_808.0
    {
        return None;
    }
    Some((scaled as i64).clamp(0, grid - 1))
}

pub fn primary_key(position: Vec3, level: u32) -> Option<CellKey> {
    if level >= 8 {
        return None;
    }
    let width = TOP_WIDTH / ((1_u64 << (2 * level)) as f64);
    let grid = 1_i64 << (23 + 2 * level);
    Some(CellKey {
        x: coordinate(position.x, width, grid)?,
        y: coordinate(position.y, width, grid)?,
        z: coordinate(position.z, width, grid)?,
    })
}

pub(crate) fn box_width(level: u32) -> Option<f64> {
    (level < 8).then(|| TOP_WIDTH / ((1_u64 << (2 * level)) as f64))
}

pub(crate) fn box_low(key: CellKey, level: u32) -> Option<Vec3> {
    let width = box_width(level)?;
    let grid = 1_i64 << (23 + 2 * level);
    let center = width * (grid as f64) * 0.5;
    Some(Vec3 {
        x: (key.x as f64) * width - center,
        y: (key.y as f64) * width - center,
        z: (key.z as f64) * width - center,
    })
}

pub(crate) fn box_snapshot(key: CellKey, level: u32) -> Option<(f64, (f64, f64, f64))> {
    let lo = box_low(key, level)?;
    let hi = box_low(
        CellKey {
            x: key.x + 1,
            y: key.y + 1,
            z: key.z + 1,
        },
        level,
    )?;
    let center = lo.add(hi.sub(lo).scale(0.5));
    Some((hi.x - lo.x, (center.x, center.y, center.z)))
}

pub(crate) fn grid_center(position: Vec3, level: u32) -> Option<(f64, f64, f64)> {
    let width = box_width(level)?;
    let grid = 1_i64 << (23 + 2 * level);
    let raw = |value: f64| {
        let scaled = (value + 0.5 * width * (grid as f64)) * (1.0 / width);
        if !scaled.is_finite()
            || scaled < -9_223_372_036_854_775_808.0
            || scaled >= 9_223_372_036_854_775_808.0
        {
            return None;
        }
        // GetBoxCenter deliberately does not use Key's grid clamp.
        Some((scaled as i64) as f64 * width + width * 0.5 - width * (grid as f64) * 0.5)
    };
    Some((raw(position.x)?, raw(position.y)?, raw(position.z)?))
}

pub(crate) fn add_ancestor_cells<const N: usize>(
    result: &mut BoundedList<CellKey, 64>,
    keys: &BoundedList<CellKey, N>,
    level: u32,
    wanted: u32,
) -> bool {
    if wanted > level {
        return true;
    }
    let mut i = 0;
    while i < keys.capacity() && i < keys.len() {
        if let Some(mut key) = keys.get(i) {
            let mut l = level;
            while l > wanted {
                key = key.parent();
                l -= 1;
            }
            let mut existing = false;
            let mut j = 0;
            while j < result.capacity() && j < result.len() {
                existing |= result.get(j) == Some(key);
                j += 1;
            }
            if !existing && !result.push(key) {
                return false;
            }
        }
        i += 1;
    }
    true
}

fn add_top(cells: &mut BoundedList<PartitionCell, 64>, mut key: CellKey, level: u32) -> bool {
    let mut ancestor = 0;
    while ancestor < level {
        key = key.parent();
        ancestor += 1;
    }
    let mut i = 0;
    while i < cells.capacity() && i < cells.len() {
        if cells.get(i).is_some_and(|cell| cell.key == key) {
            return true;
        }
        i += 1;
    }
    cells.push(PartitionCell { key, bubble: -1 })
}

impl OriginalBubbleProjection {
    pub fn reinsert_master_sphere(
        &mut self,
        new: SphericalCellKeys,
        old: Option<SphericalCellKeys>,
        others: &BoundedList<SphericalCellKeys, 8>,
    ) -> Option<i64> {
        let mut primary = new.keys.get(0)?;
        let mut level = 0;
        while level < new.level {
            primary = primary.parent();
            level += 1;
        }
        if old == Some(new) {
            return self.bubble_for(primary);
        }
        let existing = self.bubble_for(primary).unwrap_or(-1);
        // Deleting this ball's boxes releases a top cell only if no other
        // object/child remains. Static-object scopes are rejected by callers.
        let mut retained = BoundedList::new();
        let mut i = 0;
        while i < others.capacity() && i < others.len() {
            if let Some(sphere) = others.get(i) {
                let mut j = 0;
                while j < sphere.keys.capacity() && j < sphere.keys.len() {
                    if let Some(key) = sphere.keys.get(j) {
                        if !add_top(&mut retained, key, sphere.level) {
                            return None;
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        i = 0;
        while i < retained.capacity() && i < retained.len() {
            if let Some(mut cell) = retained.get(i) {
                cell.bubble = self.bubble_for(cell.key).unwrap_or(-1);
                if !retained.set(i, cell) {
                    return None;
                }
            }
            i += 1;
        }
        let mut bubble = existing;
        if bubble == -1 {
            // Source InsertInBoxes checks neighbours after its early deletion,
            // before inserting the new sphere's secondary boxes.
            let mut x = -1;
            'neighbors: while x <= 1 {
                let mut y = -1;
                while y <= 1 {
                    let mut z = -1;
                    while z <= 1 {
                        if x != 0 || y != 0 || z != 0 {
                            let key = primary.offset(x, y, z, 1_i64 << 23);
                            let mut j = 0;
                            while j < retained.capacity() && j < retained.len() {
                                if let Some(cell) = retained.get(j) {
                                    if cell.key == key && cell.bubble != -1 {
                                        bubble = cell.bubble;
                                        break 'neighbors;
                                    }
                                }
                                j += 1;
                            }
                        }
                        z += 1;
                    }
                    y += 1;
                }
                x += 1;
            }
            if bubble == -1 {
                bubble = self.next_bubble;
                self.next_bubble = self.next_bubble.checked_add(1)?;
            }
        }
        i = 0;
        while i < new.keys.capacity() && i < new.keys.len() {
            if let Some(key) = new.keys.get(i) {
                if !add_top(&mut retained, key, new.level) {
                    return None;
                }
            }
            i += 1;
        }
        i = 0;
        while i < retained.capacity() && i < retained.len() {
            if let Some(mut cell) = retained.get(i) {
                if cell.key == primary {
                    cell.bubble = bubble;
                    if !retained.set(i, cell) {
                        return None;
                    }
                }
            }
            i += 1;
        }
        self.cells = retained;
        Some(bubble)
    }

    pub fn insert_new_master_sphere(
        &mut self,
        position: Vec3,
        radius: f32,
        inflated: f64,
    ) -> Option<i64> {
        let key = self.insert_sphere(position, radius, inflated)?;
        let mut bubble = self.bubble_for(key)?;
        if bubble == -1 {
            // Box::NearbyBubbles chooses the first labelled neighbour in the
            // exact x/y/z order. Secondary cells are not seed filled here.
            let mut x = -1;
            'neighbors: while x <= 1 {
                let mut y = -1;
                while y <= 1 {
                    let mut z = -1;
                    while z <= 1 {
                        if x != 0 || y != 0 || z != 0 {
                            if let Some(found) = self.bubble_for(key.offset(x, y, z, 1_i64 << 23)) {
                                if found != -1 {
                                    bubble = found;
                                    break 'neighbors;
                                }
                            }
                        }
                        z += 1;
                    }
                    y += 1;
                }
                x += 1;
            }
            if bubble == -1 {
                bubble = self.next_bubble;
                self.next_bubble = self.next_bubble.checked_add(1)?;
            }
            let mut i = 0;
            while i < self.cells.capacity() && i < self.cells.len() {
                if let Some(mut cell) = self.cells.get(i) {
                    if cell.key == key {
                        cell.bubble = bubble;
                        if !self.cells.set(i, cell) {
                            return None;
                        }
                    }
                }
                i += 1;
            }
        }
        Some(bubble)
    }

    pub fn insert_sphere(&mut self, position: Vec3, radius: f32, inflated: f64) -> Option<CellKey> {
        let sphere = spherical_cell_keys(position, radius, inflated)?;
        let mut i = 0;
        while i < sphere.keys.capacity() && i < sphere.keys.len() {
            if let Some(key) = sphere.keys.get(i) {
                if !add_top(&mut self.cells, key, sphere.level) {
                    return None;
                }
            }
            i += 1;
        }
        let mut top = sphere.keys.get(0)?;
        i = 0;
        while i < (sphere.level as usize) {
            top = top.parent();
            i += 1;
        }
        Some(top)
    }

    pub fn label_component(&mut self, key: CellKey, bubble: i64) -> bool {
        let mut found = false;
        let mut i = 0;
        while i < self.cells.capacity() && i < self.cells.len() {
            if let Some(mut cell) = self.cells.get(i) {
                if cell.key == key {
                    found = true;
                    cell.bubble = bubble;
                    assert!(self.cells.set(i, cell));
                }
            }
            i += 1;
        }
        if !found {
            return false;
        }
        // Fixed-point seed fill is equivalent to recursive traversal on this
        // finite undirected adjacency graph; all cells remain in the graph.
        let mut changed = true;
        let mut pass = 0;
        while changed && pass < self.cells.capacity() && pass < self.cells.len() {
            changed = false;
            i = 0;
            while i < self.cells.capacity() && i < self.cells.len() {
                if let Some(mut candidate) = self.cells.get(i) {
                    if candidate.bubble == -1 {
                        let mut j = 0;
                        while j < self.cells.capacity() && j < self.cells.len() {
                            if self.cells.get(j).is_some_and(|c| {
                                c.bubble == bubble && c.key.adjacent(candidate.key)
                            }) {
                                candidate.bubble = bubble;
                                assert!(self.cells.set(i, candidate));
                                changed = true;
                                break;
                            }
                            j += 1;
                        }
                    }
                }
                i += 1;
            }
            pass += 1;
        }
        !changed
    }

    pub fn bubble_for(&self, key: CellKey) -> Option<i64> {
        let mut i = 0;
        while i < self.cells.capacity() && i < self.cells.len() {
            if let Some(cell) = self.cells.get(i) {
                if cell.key == key {
                    return Some(cell.bubble);
                }
            }
            i += 1;
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphericalCellKeys {
    pub level: u32,
    pub keys: BoundedList<CellKey, 27>,
}

pub fn spherical_cell_keys(
    position: Vec3,
    radius: f32,
    inflated: f64,
) -> Option<SphericalCellKeys> {
    point_cell_keys(position, radius, inflated, true)
}

/// Capsule sampling has the same point/modifier combinations but lacks the
/// sphere's 27-cell large-radius branch. This is an explicit source algorithm
/// choice; it does not substitute a zero radius or discard neighbours.
pub(crate) fn capsule_sample_keys(position: Vec3, radius: f32) -> Option<SphericalCellKeys> {
    point_cell_keys(position, radius, f64::from(radius), false)
}

pub(crate) fn partition_level(radius: f64) -> u32 {
    let mut level = 0_u32;
    let mut i = 0_u32;
    while i < 8 {
        let width = TOP_WIDTH / ((1_u64 << (2 * i)) as f64);
        if 3.0 * radius > width {
            break;
        }
        level = i;
        i += 1;
    }
    level
}

fn point_cell_keys(
    position: Vec3,
    radius: f32,
    inflated: f64,
    allow_large_branch: bool,
) -> Option<SphericalCellKeys> {
    // GetBox compares the inflated radius but casts only the coordinates.
    // A NaN inflation therefore selects level seven and zero modifiers; it
    // does not make a finite-coordinate insertion an undefined integer cast.
    // Preserve infinities and NaNs in those comparisons. coordinate() retains
    // the actual C++ conversion-domain check for the position itself.
    let level = partition_level(inflated);
    let key = primary_key(position, level)?;
    let width = TOP_WIDTH / ((1_u64 << (2 * level)) as f64);
    let grid = 1_i64 << (23 + 2 * level);
    let center = width * (grid as f64) * 0.5;
    let lo = Vec3 {
        x: (key.x as f64) * width - center,
        y: (key.y as f64) * width - center,
        z: (key.z as f64) * width - center,
    };
    let hi = Vec3 {
        x: ((key.x + 1) as f64) * width - center,
        y: ((key.y + 1) as f64) * width - center,
        z: ((key.z + 1) as f64) * width - center,
    };
    let modifier = |low: f64, high: f64, p: f64| {
        if low + inflated - p > 0.0 {
            -1
        } else if high - inflated - p < 0.0 {
            1
        } else {
            0
        }
    };
    let mx = modifier(lo.x, hi.x, position.x);
    let my = modifier(lo.y, hi.y, position.y);
    let mz = modifier(lo.z, hi.z, position.z);
    let mut result = SphericalCellKeys {
        level,
        keys: BoundedList::new(),
    };
    if !result.keys.push(key) {
        return None;
    }
    if allow_large_branch && level == 0 && f64::from(radius) * 2.0 > TOP_WIDTH {
        let mut x = -1_i64;
        while x <= 1 {
            let mut y = -1_i64;
            while y <= 1 {
                let mut z = -1_i64;
                while z <= 1 {
                    if x != 0 || y != 0 || z != 0 {
                        add_sphere_key(&mut result, key.offset(x, y, z, grid))?;
                    }
                    z += 1;
                }
                y += 1;
            }
            x += 1;
        }
    } else {
        // Ball::InsertInBoxes inserts x/y/z followed by xy/xz/yz/xyz.
        let offsets = [
            (mx != 0, mx, 0, 0),
            (my != 0, 0, my, 0),
            (mz != 0, 0, 0, mz),
            (mx != 0 && my != 0, mx, my, 0),
            (mx != 0 && mz != 0, mx, 0, mz),
            (my != 0 && mz != 0, 0, my, mz),
            (mx != 0 && my != 0 && mz != 0, mx, my, mz),
        ];
        let mut j = 0;
        while j < offsets.len() {
            if let Some((active, x, y, z)) = offsets.get(j) {
                if *active {
                    add_sphere_key(&mut result, key.offset(*x, *y, *z, grid))?;
                }
            }
            j += 1;
        }
    }
    Some(result)
}

fn add_sphere_key(sphere: &mut SphericalCellKeys, key: CellKey) -> Option<()> {
    let mut i = 0;
    // SphericalCellKeys owns exactly 27 slots (the original 3x3x3 branch).
    // len() checks that private representation invariant. The literal guard
    // retains every admitted key and lets symbolic execution end this loop
    // independently of unrelated, larger scenario unwind limits.
    while i < 27 && i < sphere.keys.capacity() && i < sphere.keys.len() {
        if sphere.keys.get(i) == Some(key) {
            return Some(());
        }
        i += 1;
    }
    sphere.keys.push(key).then_some(())
}

pub(crate) fn spherical_candidates(left: SphericalCellKeys, right: SphericalCellKeys) -> bool {
    footprints_touch(&left.keys, left.level, &right.keys, right.level)
}

pub(crate) fn footprints_touch<const L: usize, const R: usize>(
    left: &BoundedList<CellKey, L>,
    left_level: u32,
    right: &BoundedList<CellKey, R>,
    right_level: u32,
) -> bool {
    // Both producers choose an original partition level in 0..8. Check that
    // caller obligation rather than assuming it or clipping a level.
    assert!(left_level < 8 && right_level < 8);
    let mut i = 0;
    while i < L && i < left.capacity() && i < left.len() {
        if let Some(left_key) = left.get(i) {
            let mut j = 0;
            while j < R && j < right.capacity() && j < right.len() {
                if let Some(mut r) = right.get(j) {
                    let mut l = left_key;
                    let mut ll = left_level;
                    let mut rl = right_level;
                    let mut shifts = 0;
                    while shifts < 8 && ll > rl {
                        l = l.parent();
                        ll -= 1;
                        shifts += 1;
                    }
                    shifts = 0;
                    while shifts < 8 && rl > ll {
                        r = r.parent();
                        rl -= 1;
                        shifts += 1;
                    }
                    assert!(ll == rl);
                    if l == r {
                        return true;
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod insertion_regressions {
    use super::{TOP_WIDTH, spherical_cell_keys};
    use crate::Vec3;
    #[test]
    fn nonfinite_inflation_preserves_source_comparison_branches() {
        let point = Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let nan =
            spherical_cell_keys(point, 0.0, f64::NAN).expect("finite coordinates are defined");
        assert_eq!(nan.level, 7);
        assert_eq!(nan.keys.len(), 1);
        let infinite =
            spherical_cell_keys(point, 0.0, f64::INFINITY).expect("finite coordinates are defined");
        assert_eq!(infinite.level, 0);
        assert_eq!(infinite.keys.len(), 8);
        assert_eq!(TOP_WIDTH, 7_864_320.0);
        assert!(
            spherical_cell_keys(
                Vec3 {
                    x: f64::NAN,
                    ..point
                },
                0.0,
                0.0
            )
            .is_none()
        );
    }
}
