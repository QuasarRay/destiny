// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
// Copyright © 2015-2023 CCP ehf. Original Destiny MIT license retained.
//! Source spherical-cell components and insertion-ordered bubble dictionaries.
//! Native collections have no proof projection's eight-ball/64-cell cap. Static
//! geometry, source master insertion and engine/Python relations remain callers.
use destiny_original_spec::{Vec3, original_partition_primary_key, original_spherical_cell_keys};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Delta {
    Bubble(i64),
    Balls(Vec<i64>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Updates {
    pub dictionaries: [Vec<(i64, Delta)>; 4],
    pub transitions: Vec<(i64, i64, i64)>,
    pub invalid_user_index: Option<usize>,
}
fn set(map: &mut Vec<(i64, Delta)>, id: i64, value: Delta) {
    if let Some(entry) = map.iter_mut().find(|entry| entry.0 == id) {
        entry.1 = value;
    } else {
        map.push((id, value));
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Ball {
    pub id: i64,
    pub old: i64,
    pub new: i64,
    pub interactive: bool,
    pub global: bool,
    pub cloaked: bool,
    pub troll: bool,
}

#[derive(Debug, serde :: Serialize, serde :: Deserialize, Clone, Copy)]
struct Entry {
    bubble: i64,
    id: i64,
    action: i8,
}

#[derive(Debug, serde :: Serialize, serde :: Deserialize, Default, Clone)]
pub(crate) struct Tracking {
    groups: Vec<i64>,
    entries: Vec<Entry>,
    interactives: Vec<(i64, i64)>,
    pub partition: crate::generated_partition::Partition,
    #[serde(default)]
    keep_alive_registry: std::collections::BTreeSet<i64>,
    #[serde(default)]
    keep_alive_groups: Vec<(i64, std::collections::BTreeSet<i64>)>,
}

pub(crate) type LabelAssignments = Vec<(i64, i64)>;

pub(crate) fn initialize(
    tracking: Tracking,
    spheres: &[(i64, Vec3, f32, f64)],
) -> Result<(Tracking, LabelAssignments), &'static str> {
    let mut tracking = Tracking {
        partition: tracking.partition,
        ..Tracking::default()
    };
    let mut cells = tracking.partition.unlabeled_top_cells();
    let mut primary = Vec::with_capacity(spheres.len());
    for (id, position, radius, inflated) in spheres {
        let footprint = original_spherical_cell_keys(*position, *radius, *inflated)
            .ok_or("source spherical partition input is outside the defined cast domain")?;
        let mut first = None;
        for index in 0..footprint.keys.len() {
            let mut key = footprint
                .keys
                .get(index)
                .ok_or("missing source footprint key")?;
            for _ in 0..footprint.level {
                key = key.parent();
            }
            if first.is_none() {
                first = Some(key);
            }
            if !cells.iter().any(|(existing, _)| *existing == key) {
                cells.push((key, -1));
            }
        }
        primary.push((*id, first.ok_or("empty source sphere footprint")?));
    }
    let mut labels = Vec::with_capacity(primary.len());
    let mut next = 0_i64;
    for (id, key) in primary {
        let root = cells
            .iter()
            .position(|(candidate, _)| *candidate == key)
            .ok_or("missing primary cell")?;
        if cells[root].1 == -1 {
            cells[root].1 = next;
            let mut work = vec![root];
            while let Some(from) = work.pop() {
                let from_key = cells[from].0;
                for (index, cell) in cells.iter_mut().enumerate() {
                    if cell.1 == -1
                        && from_key.x.abs_diff(cell.0.x) <= 1
                        && from_key.y.abs_diff(cell.0.y) <= 1
                        && from_key.z.abs_diff(cell.0.z) <= 1
                    {
                        cell.1 = next;
                        work.push(index);
                    }
                }
            }
            next = next.checked_add(1).ok_or("source bubble ID exhausted")?;
        }
        labels.push((id, cells[root].1));
    }
    #[cfg(kani)]
    kani::cover!(
        true,
        "actual native source spherical component initialization completed"
    );
    tracking.partition.apply_top_labels(cells, next);
    Ok((tracking, labels))
}

impl Tracking {
    pub fn bubble_at(&self, position: Vec3) -> Result<Option<i64>, &'static str> {
        let key = original_partition_primary_key(position, 0)
            .ok_or("undefined source query coordinate cast")?;
        Ok(self
            .partition
            .bubble(crate::generated_partition::Address::from_cell(key, 0)))
    }
    pub fn transition(&mut self, ball: Ball) {
        if ball.new != -1 {
            if !self.groups.contains(&ball.new) {
                self.groups.push(ball.new);
            }
            if let Some(entry) = self
                .entries
                .iter_mut()
                .find(|entry| entry.bubble == ball.new && entry.id == ball.id)
            {
                if entry.action == -1 {
                    entry.action = 0;
                }
            } else {
                self.entries.push(Entry {
                    bubble: ball.new,
                    id: ball.id,
                    action: 1,
                });
            }
        }
        if ball.old != -1
            && let Some(index) = self
                .entries
                .iter()
                .position(|entry| entry.bubble == ball.old && entry.id == ball.id)
        {
            match self.entries[index].action {
                0 => self.entries[index].action = -1,
                1 => {
                    self.entries.remove(index);
                }
                _ => {}
            }
        }
        if ball.interactive {
            if ball.new != -1 && !self.interactives.contains(&(ball.new, ball.id)) {
                self.interactives.push((ball.new, ball.id));
            }
            self.interactives
                .retain(|entry| *entry != (ball.old, ball.id));
        }
        self.update_keep_alive(ball.id, ball.old, ball.new);
    }
    pub fn interactive(&mut self, id: i64, bubble: i64, enabled: bool) {
        if bubble == -1 {
            return;
        }
        if enabled {
            if !self.interactives.contains(&(bubble, id)) {
                self.interactives.push((bubble, id));
            }
        } else {
            self.interactives.retain(|entry| *entry != (bubble, id));
        }
    }
    pub fn copy(&mut self) {
        self.entries.retain(|entry| entry.action != -1);
        for entry in &mut self.entries {
            if entry.action == 1 {
                entry.action = 0;
            }
        }
        self.groups
            .retain(|bubble| self.entries.iter().any(|entry| entry.bubble == *bubble));
    }
    fn entry(&self, bubble: i64, id: i64) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.bubble == bubble && entry.id == id)
    }
    fn populate(&self, bubble: i64, balls: &[Ball]) -> (Vec<i64>, Vec<i64>) {
        let mut additions = Vec::new();
        let mut deletions = Vec::new();
        for entry in &self.entries {
            if entry.bubble != bubble || entry.id < -(1_i64 << 30) {
                continue;
            }
            let ball = balls.iter().find(|ball| ball.id == entry.id);
            let eligible = if entry.action != -1 {
                ball.is_some_and(|ball| !ball.global && !ball.cloaked)
            } else {
                !ball.is_some_and(|ball| ball.cloaked)
            };
            if eligible {
                if entry.action == 1 {
                    additions.push(entry.id);
                } else if entry.action == -1 {
                    deletions.push(entry.id);
                }
            }
        }
        (additions, deletions)
    }
    pub fn updates(&self, balls: &mut [Ball], users: &[Option<i64>]) -> Updates {
        self.updates_with_aliases(balls, users, [0, 1, 2, 3])
            .expect("canonical dictionary aliases")
    }
    pub fn updates_with_aliases(
        &self,
        balls: &mut [Ball],
        users: &[Option<i64>],
        aliases: [usize; 4],
    ) -> Result<Updates, &'static str> {
        if aliases
            .iter()
            .enumerate()
            .any(|(index, alias)| *alias > index || aliases[*alias] != *alias)
        {
            return Err("invalid dictionary alias relation");
        }
        let mut maps: [Vec<(i64, Delta)>; 4] = std::array::from_fn(|_| Vec::new());
        let mut transitions = Vec::new();
        let mut error = None;
        for bubble in &self.groups {
            if self.interactives.iter().any(|(key, _)| key == bubble) {
                let (adds, dels) = self.populate(*bubble, balls);
                set(&mut maps[aliases[2]], *bubble, Delta::Balls(adds));
                set(&mut maps[aliases[3]], *bubble, Delta::Balls(dels));
            }
        }
        for (index, user) in users.iter().enumerate() {
            let Some(id) = *user else {
                error = Some(index);
                break;
            };
            let Some(ball) = balls.iter().find(|ball| ball.id == id).copied() else {
                continue;
            };
            if ball.old == ball.new && ball.new != -1 && self.groups.contains(&ball.new) {
                if maps[aliases[2]].iter().any(|entry| entry.0 == ball.new)
                    && maps[aliases[3]].iter().any(|entry| entry.0 == ball.new)
                {
                    set(&mut maps[aliases[0]], id, Delta::Bubble(ball.new));
                    set(&mut maps[aliases[1]], id, Delta::Bubble(ball.new));
                } else {
                    let (adds, dels) = self.populate(ball.new, balls);
                    set(&mut maps[aliases[0]], id, Delta::Balls(adds));
                    set(&mut maps[aliases[1]], id, Delta::Balls(dels));
                }
            } else {
                let mut adds = Vec::new();
                let mut dels = Vec::new();
                for entry in &self.entries {
                    if entry.bubble == ball.new && entry.id >= -(1_i64 << 30) {
                        let other = balls.iter().find(|other| other.id == entry.id);
                        if !other.is_some_and(|other| other.global || other.cloaked) {
                            let was_in =
                                self.groups.contains(&ball.old) && self.entry(ball.old, entry.id);
                            if entry.action >= 0 && !was_in {
                                adds.push(entry.id);
                            } else if entry.action == -1
                                && (was_in || !self.groups.contains(&ball.old))
                            {
                                dels.push(entry.id);
                            }
                        }
                    }
                }
                for entry in &self.entries {
                    if entry.bubble == ball.old && entry.id >= -(1_i64 << 30) {
                        let other = balls.iter().find(|other| other.id == entry.id);
                        let eligible = if entry.action != -1 {
                            other.is_some_and(|other| !other.global && !other.cloaked)
                        } else {
                            !other.is_some_and(|other| other.cloaked)
                        };
                        let is_in =
                            self.groups.contains(&ball.new) && self.entry(ball.new, entry.id);
                        if eligible && !is_in && entry.action <= 0 {
                            dels.push(entry.id);
                        }
                    }
                }
                set(&mut maps[aliases[0]], id, Delta::Balls(adds));
                set(&mut maps[aliases[1]], id, Delta::Balls(dels));
                if !ball.troll {
                    transitions.push((id, ball.old, ball.new));
                }
            }
            if let Some(ball) = balls.iter_mut().find(|ball| ball.id == id) {
                ball.old = ball.new;
            }
        }
        #[cfg(kani)]
        kani::cover!(
            true,
            "actual native ordered bubble update construction completed"
        );
        // Distinct outputs already own the correct maps. Only aliases need
        // repeated DTO copies; avoid cloning the ordinary source call's maps.
        let dictionaries = if aliases == [0, 1, 2, 3] {
            maps
        } else {
            std::array::from_fn(|index| maps[aliases[index]].clone())
        };
        Ok(Updates {
            dictionaries,
            transitions,
            invalid_user_index: error,
        })
    }
}

impl Tracking {
    pub(crate) fn from_partition(partition: crate::generated_partition::Partition) -> Self {
        Self {
            partition,
            ..Self::default()
        }
    }
}

impl Tracking {
    #[doc = "src/Ballpark_Blue.cpp:224-240, src/Ballpark.cpp:6755; raw registration set mutation has no automatic group update"]
    pub fn register_keep_alive(&mut self, id: i64, enabled: bool) {
        if enabled {
            (self.keep_alive_registry).insert(id);
        } else {
            (self.keep_alive_registry).remove(&id);
        }
    }
}

impl Tracking {
    #[doc = "src/Ballpark.cpp:6770-6785"]
    pub fn keep_alive_add(&mut self, id: i64, bubble: i64) {
        if (bubble) == (-1_i64) {
            return;
        }
        if let Some(index) =
            ((self.keep_alive_groups).iter()).position(|entry| (entry.0) == (bubble))
        {
            (self.keep_alive_groups[index].1).insert(id);
        }
        if !(((self.keep_alive_groups).iter()).any(|entry| (entry.0) == (bubble))) {
            (self.keep_alive_groups).push((bubble, std::collections::BTreeSet::from([id])));
        }
    }
}

impl Tracking {
    #[doc = "src/Ballpark.cpp:6787-6803"]
    pub fn keep_alive_remove(&mut self, id: i64, bubble: i64) {
        if (bubble) == (-1_i64) {
            return;
        }
        if let Some(index) =
            ((self.keep_alive_groups).iter()).position(|entry| (entry.0) == (bubble))
        {
            (self.keep_alive_groups[index].1).remove(&id);
            if (self.keep_alive_groups[index].1).is_empty() {
                (self.keep_alive_groups).remove(index);
            }
        }
    }
}

impl Tracking {
    #[doc = "src/Ballpark.cpp:6745-6768; valid integer/set domain; Python allocation/type-error paths remain separate"]
    pub fn update_keep_alive(&mut self, id: i64, old: i64, new: i64) {
        if !((self.keep_alive_registry).contains(&id)) {
            return;
        }
        (self).keep_alive_add(id, new);
        (self).keep_alive_remove(id, old);
    }
}

impl Tracking {
    #[doc = "src/Ballpark_Blue.cpp:224-240; direct dictionary/set world-state replacement"]
    pub fn set_keep_alive_members(
        &mut self,
        bubble: i64,
        members: std::collections::BTreeSet<i64>,
    ) {
        if let Some(index) =
            ((self.keep_alive_groups).iter()).position(|entry| (entry.0) == (bubble))
        {
            self.keep_alive_groups[index].1 = members;
            return;
        }
        (self.keep_alive_groups).push((bubble, members));
    }
}

impl Tracking {
    #[doc = "src/Ballpark_Blue.cpp:214-240; direct valid integer/set membership replacement; empty-key/host alias/order correspondence remains open"]
    pub fn set_interactive_members(
        &mut self,
        bubble: i64,
        members: std::collections::BTreeSet<i64>,
    ) {
        (self.interactives).retain(|entry| (entry.0) != (bubble));
        for id in members {
            (self.interactives).push((bubble, id));
        }
    }
}

impl Tracking {
    #[doc = "src/Ballpark.cpp:6261-6277"]
    pub fn interactive_count(&self, master: bool, bubble: i64) -> usize {
        if !(master) {
            return 0_usize;
        }
        (((self.interactives).iter()).filter(|entry| (entry.0) == (bubble))).count()
    }
}

impl Tracking {
    #[doc = "src/Ballpark.cpp:6244-6255"]
    pub fn keep_alive_count(&self, bubble: i64) -> usize {
        ((((self.keep_alive_groups).iter()).find(|entry| (entry.0) == (bubble)))
            .map(|entry| (entry.1).len()))
        .unwrap_or(0_usize)
    }
}

impl Tracking {
    #[doc = "src/Ballpark.cpp:6216-6277; logical typed snapshot projection of the existing tracker"]
    pub fn liveness_state(&self) -> crate::generated_liveness::LivenessState {
        crate::generated_liveness::LivenessState {
            registered: (self.keep_alive_registry).clone(),
            keep_alive: (self.keep_alive_groups).clone(),
            interactives: (self.interactives).clone(),
        }
    }
}
