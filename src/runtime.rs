use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    io::{self, Write},
    time::Duration,
};

use avian3d::physics_transform::PhysicsTransformConfig;
use avian3d::prelude::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use bevy::{
    ecs::system::SystemParam,
    math::{DMat3, DQuat, DVec3, EulerRot},
    prelude::*,
    state::app::StatesPlugin,
    time::TimeUpdateStrategy,
};
use destiny_original_spec::{
    NO_FORMATION, OriginalMode, OriginalParkBall, Vec3 as OriginalVec3, adjust_time,
    assign_formation, clamp_speed_fraction, non_negative_setter_accepts, original_aligned_for_warp,
    original_follow_thrust_with_environment, original_goto_thrust, original_integrate_duration,
    original_new_orbit_thrust_with_environment, original_old_orbit_thrust_with_friction,
    original_warp_distance, positive_setter_accepts, surface_distance_from_center,
};
use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::{
    carbon_codec::decode_canonical,
    formation::{free_formation_slot, reserve_formation_slot},
    generated_registry::{GENERATED_TITLE_RECORDS, GeneratedTitleRecord},
};

const MAX_COLLISION_SUBSTEPS: u32 = 1_024;
const MAX_COORDINATE: f64 = 1.0e100;
const MAX_RADIUS: f64 = 1.0e50;
const MAX_MASS: f64 = 1.0e100;
const MAX_VELOCITY: f64 = 1.0e100;
const MAX_AGILITY: f64 = 1.0e100;
const MAX_CHILD_DESCRIPTOR_BYTES: usize = 48 * 1024 * 1024;
const MAX_PROXIMITY_WORK_PER_TICK: usize = 2_000_000;
const MAX_NETWORK_UPDATE_ROWS: usize = 100_000;
const MAX_NETWORK_RECIPIENTS_PER_ROW: usize = 100_000;
const MAX_NETWORK_EXPANDED_ROWS: usize = 200_000;
const MAX_CONFIGURED_SNAPSHOT_BYTES: usize = 32 * 1024 * 1024;
const MAX_CONFIGURED_SNAPSHOT_BALLS: usize = 100_000;
const MAX_CONFIGURED_CHILD_SHAPES_PER_BALL: usize = 4_096;
const MAX_CONFIGURED_OUTBOX_MESSAGES: usize = 10_000;
const MAX_CONFIGURED_OUTBOX_BYTES: usize = 48 * 1024 * 1024;
const MAX_FORMATION_DEFINITIONS: usize = 128;
const MAX_FORMATION_OFFSETS: usize = 100_000;

fn no_formation() -> u8 {
    NO_FORMATION
}

/// Parse JSON while rejecting duplicate object members at every depth.
///
/// Serde's typed structs reject duplicate typed fields, but descriptors and
/// protocol arguments intentionally contain `Value` subtrees, where ordinary
/// `serde_json` parsing otherwise keeps the last duplicate silently.
pub(crate) fn strict_json_from_slice<T: DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, serde_json::Error> {
    let StrictJsonValue(value) = serde_json::from_slice(bytes)?;
    serde_json::from_value(value)
}

struct StrictJsonValue(Value);

impl<'de> Deserialize<'de> for StrictJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(StrictJsonVisitor)
            .map(StrictJsonValue)
    }
}

struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object members")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("JSON numbers must be finite"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StrictJsonValue::deserialize(deserializer).map(|value| value.0)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
        while let Some(StrictJsonValue(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(<A::Error as serde::de::Error>::custom(format!(
                    "duplicate JSON object member {key:?}"
                )));
            }
            let StrictJsonValue(value) = object.next_value()?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompatOptions {
    pub is_master: bool,
    pub length_unit: f64,
    pub tick_interval_ms: f64,
    pub collision_substeps: u32,
    pub enable_network_components: bool,
    pub use_iterative_collision: bool,
    pub use_dynamical_orientation: bool,
    pub disable_dynamical_orientation_for_missiles: bool,
    pub use_new_orbit: bool,
    pub max_snapshot_bytes: usize,
    pub max_snapshot_balls: usize,
    pub max_child_shapes_per_ball: usize,
    pub max_outbox_messages: usize,
    pub max_outbox_bytes: usize,
}

impl Default for CompatOptions {
    fn default() -> Self {
        Self {
            is_master: false,
            length_unit: 1.0,
            tick_interval_ms: 1000.0,
            collision_substeps: 20,
            enable_network_components: cfg!(feature = "carbon-network"),
            use_iterative_collision: false,
            use_dynamical_orientation: false,
            disable_dynamical_orientation_for_missiles: false,
            use_new_orbit: false,
            max_snapshot_bytes: 32 * 1024 * 1024,
            max_snapshot_balls: 100_000,
            max_child_shapes_per_ball: 4_096,
            max_outbox_messages: 10_000,
            max_outbox_bytes: 48 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatRequest {
    pub title: String,
    #[serde(default = "default_operation")]
    pub operation: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub args: Vec<Value>,
    #[serde(default)]
    pub kwargs: serde_json::Map<String, Value>,
}

fn default_operation() -> String {
    "call".to_owned()
}

#[derive(Debug, Error)]
pub enum CompatError {
    #[error("{0}")]
    ValueError(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("ball {0} is not in the ballpark")]
    BallNotFound(i64),
    #[error("ball {0} already exists")]
    DuplicateBall(i64),
    #[error("unsupported Destiny title: {0}")]
    UnsupportedTitle(String),
    #[error("Bevy/Avian operation failed: {0}")]
    Engine(String),
}

impl CompatError {
    pub fn payload(&self) -> Value {
        match self {
            Self::ValueError(_) => json!({"code":"value_error","message":self.to_string()}),
            Self::BallNotFound(ball_id) => json!({
                "code": "ball_not_found",
                "message": self.to_string(),
                "ball_id": ball_id,
            }),
            Self::DuplicateBall(ball_id) => json!({
                "code": "duplicate_ball",
                "message": self.to_string(),
                "ball_id": ball_id,
            }),
            Self::UnsupportedTitle(title) => {
                let record = title_record(title);
                json!({
                    "code": "unsupported_title",
                    "message": self.to_string(),
                    "canonical_title": title,
                    "verdict": record.map_or("UNKNOWN", |row| row.verdict),
                    "material_difference": record.map_or("The title is not present in the audited registry.", |row| row.material_difference),
                    "mapping_items": record.map_or(&[][..], |row| row.mapping_items),
                })
            }
            Self::InvalidRequest(_) => {
                json!({"code": "invalid_request", "message": self.to_string()})
            }
            Self::Engine(_) => json!({"code": "engine_error", "message": self.to_string()}),
        }
    }
}

fn title_record(title: &str) -> Option<&'static GeneratedTitleRecord> {
    GENERATED_TITLE_RECORDS
        .iter()
        .find(|record| record.canonical_title == title)
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DestinyBallId(pub i64);

/// Lossless Destiny API mass. Avian 0.7 intentionally keeps its `Mass`
/// component as `f32` even when the solver uses `f64`, so the compatibility
/// layer stores the API value separately and only projects it into Avian.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DestinyMass(pub f64);

/// Delayed-destruction lifecycle state.  The component is intentionally
/// replicated/relevance-filtered as part of the ball entity lifecycle rather
/// than inferred from a local timer.
#[derive(Component, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DestinyPendingRemoval {
    pub due_tick: i64,
    pub reason: String,
}

/// Client presentation angular velocity.  Visual-only impulses never mutate
/// the authoritative, replicated Avian `AngularVelocity` component.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct DestinyPresentationAngularVelocity(pub DVec3);

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DestinyBallMetadata {
    pub radius: f64,
    pub is_free: bool,
    pub is_global: bool,
    pub is_massive: bool,
    pub is_interactive: bool,
    pub is_space_junk: bool,
    pub agility: f64,
    pub speed_fraction: f64,
    pub angular_agility: f64,
    pub is_cloaked: i32,
    pub new_bubble_id: i64,
    pub old_bubble_id: i64,
    pub effect_stamp: i64,
    #[serde(default = "no_formation")]
    pub formation_id: u8,
    #[serde(default)]
    pub formation_slots: u16,
    #[serde(default)]
    pub massive_before_cloak: Option<bool>,
    pub minis: Vec<Value>,
    #[serde(default)]
    pub mini_ids: Vec<i64>,
    #[serde(default)]
    pub owner_id: i64,
    pub sensors: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BallSnapshot {
    pub id: i64,
    pub mass: f64,
    pub radius: f64,
    pub max_velocity: f64,
    pub is_free: bool,
    pub is_global: bool,
    pub is_massive: bool,
    pub is_interactive: bool,
    pub is_space_junk: bool,
    pub position: Vec<f64>,
    pub velocity: Vec<f64>,
    pub agility: f64,
    pub speed_fraction: f64,
    pub max_angular_velocity: f64,
    pub angular_agility: f64,
    pub angular_velocity: Vec<f64>,
    pub rotation: Vec<f64>,
    pub is_cloaked: i32,
    pub new_bubble_id: i64,
    pub old_bubble_id: i64,
    pub effect_stamp: i64,
    #[serde(default = "no_formation")]
    pub formation_id: u8,
    #[serde(default)]
    pub formation_slots: u16,
    pub massive_before_cloak: Option<bool>,
    pub minis: Vec<Value>,
    #[serde(default)]
    pub mini_ids: Vec<i64>,
    #[serde(default)]
    pub owner_id: i64,
    pub sensors: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion: Option<MotionSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notifications: Option<RangeNotificationSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionSnapshot {
    pub mode: u8,
    pub destination: [f64; 3],
    pub min_range: f64,
    pub warp_factor: i32,
    pub warp_distance: f64,
    pub follow_id: Option<i64>,
    pub follow_range: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RangeNotificationSnapshot {
    pub range: f64,
    pub within: bool,
    pub target: Option<i64>,
    pub tracking_range: f64,
    pub tracking_within: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParkSnapshot {
    format: String,
    schema_version: u16,
    park: ParkSnapshotMetadata,
    balls: Vec<BallSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingRemovalSnapshot {
    ball_id: i64,
    due_tick: i64,
    reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParkSnapshotMetadata {
    is_master: bool,
    running: bool,
    tick_interval_ms: f64,
    friction: f64,
    current_time: i64,
    #[serde(default = "default_local_counter")]
    local_counter: i64,
    #[serde(default = "default_local_high_counter")]
    local_high_counter: i64,
    time: i64,
    ego: i64,
    collision_substeps: u32,
    use_iterative_collision: bool,
    use_dynamical_orientation: bool,
    disable_dynamical_orientation_for_missiles: bool,
    use_new_orbit: bool,
    #[serde(default)]
    formations: Vec<Vec<[f64; 3]>>,
    pending_removals: Vec<PendingRemovalSnapshot>,
    snapshot_semantics: String,
}

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CarbonNetworkOutbox {
    pub singlecasts: Vec<Value>,
    pub narrowcasts: Vec<Value>,
    pub batches: Vec<Value>,
    #[serde(skip)]
    pub queued_bytes: usize,
    #[serde(skip)]
    pub last_batch_id: i64,
    #[serde(skip)]
    pub last_batch_envelope: Option<Value>,
}

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProximityEventOutbox(pub Vec<Value>);

#[derive(Resource, Debug, Clone, Copy)]
pub struct DestinySpaceFriction(pub f64);

#[derive(Debug, Clone, Copy)]
struct CompatLimits {
    max_snapshot_bytes: usize,
    max_snapshot_balls: usize,
    max_child_shapes_per_ball: usize,
    max_outbox_messages: usize,
    max_outbox_bytes: usize,
}

#[derive(Debug)]
struct ParkMetadata {
    is_master: bool,
    current_time: i64,
    local_counter: i64,
    local_high_counter: i64,
    time: i64,
    ego: i64,
    network_components: bool,
    collision_substeps: u32,
    use_iterative_collision: bool,
    use_dynamical_orientation: bool,
    disable_dynamical_orientation_for_missiles: bool,
    use_new_orbit: bool,
    formations: Vec<Vec<[f64; 3]>>,
    pending_removals: HashMap<i64, i64>,
    limits: CompatLimits,
}

#[derive(Clone)]
struct EvolutionBallCheckpoint {
    entity: Entity,
    ball_id: DestinyBallId,
    metadata: DestinyBallMetadata,
    destiny_mass: DestinyMass,
    avian_mass: Mass,
    max_linear_speed: MaxLinearSpeed,
    max_angular_speed: MaxAngularSpeed,
    rigid_body: RigidBody,
    collider: Collider,
    collider_disabled: bool,
    gravity_scale: GravityScale,
    pending_removal: Option<DestinyPendingRemoval>,
    position: Position,
    rotation: Rotation,
    linear_velocity: LinearVelocity,
    angular_velocity: AngularVelocity,
    linear_damping: LinearDamping,
    transform: Transform,
}

struct EvolutionCheckpoint {
    follower_sets: HashMap<u64, HashSet<i64>>,
    balls: Vec<EvolutionBallCheckpoint>,
    current_time: i64,
    time: i64,
    fixed_time: Time<Fixed>,
    physics_time: Time<Physics>,
    substeps_time: Time<Substeps>,
    proximity_events: ProximityEventOutbox,
    network_outbox: CarbonNetworkOutbox,
    motion: HashMap<i64, MotionControl>,
    notifications: HashMap<i64, RangeNotifications>,
}

#[derive(Clone, Debug)]
struct MotionControl {
    mode: OriginalMode,
    destination: OriginalVec3,
    min_range: f64,
    warp_factor: i32,
    warp_distance: f64,
    follow_id: Option<i64>,
    follow_range: f32,
}

impl Default for MotionControl {
    fn default() -> Self {
        Self {
            mode: OriginalMode::Stop,
            destination: OriginalVec3::ZERO,
            min_range: 0.0,
            warp_factor: 20,
            warp_distance: -1.0,
            follow_id: None,
            follow_range: 10.0,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ModeCallback {
    pub(crate) ball_id: i64,
    generation: u64,
    pub(crate) old_mode: u8,
    pub(crate) new_mode: u8,
    new: OriginalMode,
}

#[derive(Clone, Debug)]
pub(crate) enum CallCallback {
    Mode(ModeCallback),
    Park {
        kind: u32,
        arguments: Value,
    },
    Ball {
        ball_id: i64,
        generation: u64,
        kind: u32,
        arguments: Value,
    },
    Remove {
        ball_id: i64,
        generation: u64,
        delay: i64,
        step: RemovalStep,
        arrival: Option<OriginalVec3>,
    },
    GotoPoint {
        ball_id: i64,
        generation: u64,
        point: OriginalVec3,
    },
    Follow {
        ball_id: i64,
        generation: u64,
        target: i64,
        target_generation: u64,
        kind: FollowKind,
    },
    RegisterFollower {
        id: i64,
        target: i64,
        target_generation: u64,
    },
    WarpStart {
        id: i64,
        generation: u64,
        point: OriginalVec3,
        min_range: f64,
        factor: i32,
    },
    EntityArrival {
        id: i64,
        generation: u64,
        point: OriginalVec3,
    },
    SetMode {
        id: i64,
        generation: u64,
        mode: OriginalMode,
        troll_delay: Option<i32>,
    },
    FormationChange {
        id: i64,
        generation: u64,
        formation: u8,
        remaining: VecDeque<(i64, u64)>,
    },
}

#[derive(Clone, Debug)]
pub(crate) enum FollowKind {
    Follow(f32),
    Orbit(f32),
    Missile(i64),
    Formation(i8),
}

#[derive(Clone, Debug)]
pub(crate) enum RemovalStep {
    Followers,
    NextFollower(VecDeque<i64>),
    Mark,
    Detach,
}

struct PendingTick {
    force: bool,
    results: Vec<PreparedMotion>,
}

#[derive(Clone, Copy)]
struct PreparedMotion {
    id: i64,
    position: OriginalVec3,
    velocity: OriginalVec3,
    acceleration: OriginalVec3,
    in_warp: bool,
}

#[derive(Default, Clone)]
struct RangeNotifications {
    range: f64,
    within: bool,
    target: Option<i64>,
    tracking_range: f64,
    tracking_within: bool,
}

// The enclosing operation owns this continuation. Nested calls cannot take
// its remaining work. Each candidate is inspected only when reached, after
// the preceding synchronous callback has returned.
pub(crate) struct NotificationPass {
    owners: VecDeque<(i64, Entity)>,
    owner: Option<(i64, Entity)>,
    stage: u8,
    sensors: VecDeque<bool>,
    sensor: Option<SensorPass>,
    work: usize,
}

struct SensorPass {
    cloak: bool,
    candidates: VecDeque<(i64, Entity)>,
    vanished: VecDeque<i64>,
}

fn mode_number(mode: OriginalMode) -> u8 {
    match mode {
        OriginalMode::Goto => 0,
        OriginalMode::Follow => 1,
        OriginalMode::Stop => 2,
        OriginalMode::Warp => 3,
        OriginalMode::Orbit => 4,
        OriginalMode::Missile => 5,
        OriginalMode::Formation => 12,
        OriginalMode::Mushroom => 6,
        OriginalMode::Boid => 7,
        OriginalMode::Troll => 8,
        OriginalMode::Miniball => 9,
        OriginalMode::Field => 10,
        OriginalMode::Rigid => 11,
    }
}

#[derive(SystemParam)]
struct DestinyBubbleCollisionHooks<'w, 's> {
    balls: Query<
        'w,
        's,
        (
            &'static DestinyBallMetadata,
            Option<&'static DestinyPendingRemoval>,
        ),
    >,
}

impl CollisionHooks for DestinyBubbleCollisionHooks<'_, '_> {
    fn filter_pairs(&self, collider1: Entity, collider2: Entity, _commands: &mut Commands) -> bool {
        let Ok([(left, left_pending), (right, right_pending)]) =
            self.balls.get_many([collider1, collider2])
        else {
            // Non-Destiny host colliders retain their host-defined behavior.
            return true;
        };
        if left_pending.is_some() || right_pending.is_some() {
            return false;
        }
        left.is_global
            || right.is_global
            || (left.new_bubble_id >= 0 && left.new_bubble_id == right.new_bubble_id)
    }
}

pub struct CompatRuntime {
    app: App,
    balls: HashMap<i64, Entity>,
    ball_generations: HashMap<i64, u64>,
    follower_sets: HashMap<u64, HashSet<i64>>,
    next_ball_generation: u64,
    retained_balls: HashMap<u64, RetainedBall>,
    park: ParkMetadata,
    terminal_error: Option<String>,
    motion: HashMap<i64, MotionControl>,
    mode_callbacks: VecDeque<CallCallback>,
    original_bubbles: Option<crate::bubbles::Tracking>,
    evolution_phase: u8,
    pending_tick: Option<PendingTick>,
    pending_tick_time: Option<i64>,
    range_notifications: HashMap<i64, RangeNotifications>,
}

struct RetainedBall {
    id: i64,
    attached: bool,
    moribund: bool,
    references: usize,
    detached: Option<BallSnapshot>,
    client: bool,
    identity_fields: [i64; 3],
}

type SweptBoundComponents = (
    &'static DestinyBallMetadata,
    &'static Collider,
    &'static Position,
    &'static Rotation,
    &'static LinearVelocity,
    &'static avian3d::collider_tree::ColliderTreeProxyKey,
    &'static mut ColliderAabb,
    &'static mut avian3d::collision::collider::EnlargedAabb,
);

fn prepare_destiny_swept_bounds(
    mut bodies: Query<SweptBoundComponents, Without<ColliderDisabled>>,
    time: Res<Time<Physics>>,
    length_unit: Res<PhysicsLengthUnit>,
    config: Res<NarrowPhaseConfig>,
    mut trees: ResMut<avian3d::collider_tree::ColliderTrees>,
    mut moved: ResMut<avian3d::collider_tree::MovedProxies>,
) {
    use avian3d::collision::collider::SimpleCollider;
    let dt = time.delta_secs_f64();
    for (metadata, collider, position, rotation, velocity, key, mut aabb, mut enlarged) in
        &mut bodies
    {
        if !metadata.is_free {
            continue;
        }
        // Root free colliders are spheres; rotation does not change their
        // bounds. Include the whole linear path before broad-phase selection,
        // also for newly added bodies and velocity changes before this tick.
        let end = position.0 + velocity.0 * dt;
        *aabb = collider
            .swept_aabb(position.0, *rotation, end, *rotation)
            .grow(DVec3::splat(length_unit.0 * config.contact_tolerance));
        if enlarged.update(&aabb, 0.0) {
            trees
                .tree_for_type_mut(key.tree_type())
                .resize_proxy_aabb(key.id(), enlarged.get().into());
            moved.insert(*key);
        }
    }
}

impl CompatRuntime {
    pub fn new(options: CompatOptions) -> Result<Self, CompatError> {
        Self::new_with_app(options, |_| {})
    }

    /// Builds the compatibility runtime while giving a Rust host one chance to
    /// install its complete Lightyear/Replicon link stack and other Bevy
    /// plugins before the app is finalized. Bevy forbids adding plugins after
    /// `App::finish`, so host integration must happen through this constructor
    /// rather than through [`Self::app_mut`].
    pub fn new_with_app<F>(options: CompatOptions, configure: F) -> Result<Self, CompatError>
    where
        F: FnOnce(&mut App),
    {
        if !options.length_unit.is_finite() || options.length_unit <= 0.0 {
            return Err(CompatError::InvalidRequest(
                "length_unit must be finite and positive".into(),
            ));
        }
        let tick_duration = tick_duration(options.tick_interval_ms)?;
        if options.collision_substeps == 0 || options.collision_substeps > MAX_COLLISION_SUBSTEPS {
            return Err(CompatError::InvalidRequest(format!(
                "collision_substeps must be in 1..={MAX_COLLISION_SUBSTEPS}"
            )));
        }
        if options.use_dynamical_orientation {
            return Err(CompatError::InvalidRequest(
                "use_dynamical_orientation is unsupported until the Destiny angular controller state is implemented".into(),
            ));
        }
        if !(1..=MAX_CONFIGURED_SNAPSHOT_BYTES).contains(&options.max_snapshot_bytes)
            || !(1..=MAX_CONFIGURED_SNAPSHOT_BALLS).contains(&options.max_snapshot_balls)
            || !(1..=MAX_CONFIGURED_CHILD_SHAPES_PER_BALL)
                .contains(&options.max_child_shapes_per_ball)
            || !(1..=MAX_CONFIGURED_OUTBOX_MESSAGES).contains(&options.max_outbox_messages)
            || !(1..=MAX_CONFIGURED_OUTBOX_BYTES).contains(&options.max_outbox_bytes)
        {
            return Err(CompatError::InvalidRequest(
                "compatibility limits must be positive and no larger than the hard resource ceilings"
                    .into(),
            ));
        }
        if options.disable_dynamical_orientation_for_missiles {
            return Err(CompatError::InvalidRequest(
                "disable_dynamical_orientation_for_missiles requires missile classification that this subset does not expose".into(),
            ));
        }
        #[cfg(not(feature = "carbon-network"))]
        if options.enable_network_components {
            return Err(CompatError::InvalidRequest(
                "enable_network_components requires the carbon-network Cargo feature".into(),
            ));
        }

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            StatesPlugin,
            PhysicsPlugins::default()
                .with_collision_hooks::<DestinyBubbleCollisionHooks>()
                .with_length_unit(options.length_unit),
        ));
        // Destiny's Evolve contract advances exactly one simulation tick per
        // call. This avoids coupling headless physics progress to wall-clock
        // time between calls to App::update.
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
        app.insert_resource(Time::<Fixed>::from_duration(tick_duration));
        // This driver advances explicit simulation ticks. Bevy's default
        // 250 ms virtual-time clamp would otherwise skip fixed physics steps
        // for the original 1000 ms Destiny tick.
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .set_max_delta(Duration::MAX);
        app.insert_resource(SubstepCount(if options.use_iterative_collision {
            options.collision_substeps
        } else {
            1
        }));
        app.init_resource::<CarbonNetworkOutbox>();
        app.init_resource::<ProximityEventOutbox>();
        app.add_systems(
            PhysicsSchedule,
            prepare_destiny_swept_bounds
                .after(avian3d::collider_tree::ColliderTreeSystems::UpdateAabbs)
                .before(BroadPhaseSystems::CollectCollisions),
        );
        app.insert_resource(DestinySpaceFriction(1_000_000.0));
        {
            let mut transform_config = app.world_mut().resource_mut::<PhysicsTransformConfig>();
            transform_config.transform_to_position = false;
            transform_config.position_to_transform = false;
        }
        app.world_mut().resource_mut::<Time<Physics>>().pause();

        #[cfg(feature = "carbon-network")]
        if options.enable_network_components {
            use crate::network::RoomPlugin;

            // This option adds only the mapped relevance components. A real
            // Carbon host must install its complete Lightyear server/link stack
            // and then `DestinyCarbonInteropPlugin` on this public App before
            // attaching links. Installing only Lightyear's Replicon bridge is
            // invalid: its systems require timeline/transport resources and
            // would panic in a standalone headless runtime.
            app.add_plugins(RoomPlugin);
        }

        // Product hosts add their complete link/authentication/transport stack
        // here, followed by `DestinyCarbonInteropPlugin`. The standalone ABI
        // passes a no-op configurator and retains an inspectable bounded outbox.
        configure(&mut app);

        // `CompatRuntime` is a headless, manually driven Bevy application. The
        // normal runner invokes these lifecycle hooks before its first update,
        // but callers of `App::update` must do so explicitly. Avian creates
        // several required diagnostics resources from `Plugin::finish`.
        app.finish();
        app.cleanup();

        // Run Bevy's startup schedule once while Avian's physics clock is
        // paused. Without this priming update, the first explicit `Evolve`
        // only initializes schedules and silently consumes the caller's tick.
        app.update();

        Ok(Self {
            app,
            balls: HashMap::new(),
            ball_generations: HashMap::new(),
            follower_sets: HashMap::new(),
            next_ball_generation: 1,
            retained_balls: HashMap::new(),
            terminal_error: None,
            motion: HashMap::new(),
            mode_callbacks: VecDeque::new(),
            original_bubbles: None,
            evolution_phase: 0,
            pending_tick: None,
            pending_tick_time: None,
            range_notifications: HashMap::new(),
            park: ParkMetadata {
                is_master: options.is_master,
                current_time: 0,
                local_counter: default_local_counter(),
                local_high_counter: default_local_high_counter(),
                time: 0,
                ego: 0,
                network_components: options.enable_network_components,
                collision_substeps: options.collision_substeps,
                use_iterative_collision: options.use_iterative_collision,
                use_dynamical_orientation: options.use_dynamical_orientation,
                disable_dynamical_orientation_for_missiles: options
                    .disable_dynamical_orientation_for_missiles,
                use_new_orbit: options.use_new_orbit,
                formations: Vec::new(),
                pending_removals: HashMap::new(),
                limits: CompatLimits {
                    max_snapshot_bytes: options.max_snapshot_bytes,
                    max_snapshot_balls: options.max_snapshot_balls,
                    max_child_shapes_per_ball: options.max_child_shapes_per_ball,
                    max_outbox_messages: options.max_outbox_messages,
                    max_outbox_bytes: options.max_outbox_bytes,
                },
            },
        })
    }

    pub fn dispatch(&mut self, request: CompatRequest) -> Result<Value, CompatError> {
        let (result, finish_tick, mut callbacks) = self.dispatch_yielding(request)?;
        while let Some(event) = callbacks.pop_front() {
            self.complete_mode_callback(event)?;
            self.prepend_callback_continuations(&mut callbacks);
        }
        if finish_tick {
            self.complete_callback_tick()?;
        }
        Ok(result)
    }

    pub(crate) fn dispatch_yielding(
        &mut self,
        request: CompatRequest,
    ) -> Result<(Value, bool, VecDeque<CallCallback>), CompatError> {
        let finish_tick = request.title == "destiny.Ballpark.Evolve" && request.operation == "call";
        let start = self.mode_callbacks.len();
        match self.dispatch_inner(request) {
            Ok(value) => {
                let callbacks =
                    crate::notification_batch::take_notifications(&mut self.mode_callbacks, start)
                        .ok_or_else(|| {
                            CompatError::Engine(
                                "operation removed its caller's pending notifications".into(),
                            )
                        })?;
                Ok((value, finish_tick, callbacks))
            }
            Err(error) => {
                self.mode_callbacks.truncate(start);
                Err(error)
            }
        }
    }

    fn dispatch_inner(&mut self, request: CompatRequest) -> Result<Value, CompatError> {
        if let Some(error) = &self.terminal_error {
            return Err(CompatError::Engine(format!(
                "runtime is unavailable after a terminal physics failure: {error}"
            )));
        }
        if !request.kwargs.is_empty() {
            return Err(CompatError::InvalidRequest(
                "keyword arguments are not supported".into(),
            ));
        }
        if !matches!(
            request.operation.as_str(),
            "call" | "construct" | "get" | "set"
        ) {
            return Err(CompatError::InvalidRequest(format!(
                "invalid operation {:?}",
                request.operation
            )));
        }
        if request.title != "destiny.Ballpark.__init__"
            && request.title.starts_with("destiny.Ballpark.")
            && !request.target.as_deref().is_some_and(is_park_target)
        {
            return Err(CompatError::InvalidRequest(
                "ballpark operation requires target park:0".into(),
            ));
        }
        match request.title.as_str() {
            "destiny.Ballpark.__init__" => {
                if request.operation != "construct" || request.target.is_some() {
                    return Err(CompatError::InvalidRequest(
                        "Ballpark construction requires operation=construct and a null target"
                            .into(),
                    ));
                }
                require_arity(&request.args, &[0, 1], "destiny.Ballpark.__init__")?;
                let is_master = request
                    .args
                    .first()
                    .map(|value| value_bool(Some(value), "isMaster"))
                    .transpose()?
                    .unwrap_or(false);
                self.clear_all()?;
                self.park.formations.clear();
                self.park.is_master = is_master;
                self.park.current_time = 0;
                self.park.time = 0;
                self.park.ego = 0;
                self.app.world_mut().resource_mut::<Time<Physics>>().pause();
                Ok(json!("park:0"))
            }
            title if title.starts_with("dbc.compat.") => {
                if !request.target.as_deref().is_some_and(is_park_target) {
                    return Err(CompatError::InvalidRequest(
                        "compatibility operation requires a park target".into(),
                    ));
                }
                self.dispatch_compat(title, &request.operation, &request.args)
            }
            title
                if title.starts_with("destiny.Ball.")
                    || title.starts_with("destiny.ClientBall.") =>
            {
                let ball_id = parse_ball_target(request.target.as_deref())?;
                if let Some(generation) = parse_ball_generation(request.target.as_deref())? {
                    let retained = self
                        .retained_balls
                        .get(&generation)
                        .filter(|ball| ball.id == ball_id && ball.references > 0)
                        .ok_or(CompatError::BallNotFound(ball_id))?;
                    if title.starts_with("destiny.ClientBall.") && !retained.client {
                        return Err(CompatError::UnsupportedTitle(title.into()));
                    }
                    if !retained.attached {
                        if request.operation == "get" {
                            require_arity(&request.args, &[0], title)?;
                            return retained_ball_property(
                                retained,
                                title.rsplit('.').next().unwrap_or(title),
                            );
                        }
                        if request.operation == "call" {
                            return retained_ball_call(
                                retained,
                                title.rsplit('.').next().unwrap_or(title),
                                &request.args,
                            );
                        }
                        return Err(CompatError::InvalidRequest(
                            "detached property mutation remains unimplemented".into(),
                        ));
                    }
                    if self.ball_generations.get(&ball_id) != Some(&generation) {
                        return Err(CompatError::BallNotFound(ball_id));
                    }
                }
                self.dispatch_ball(title, &request.operation, ball_id, &request.args)
            }
            title if title.starts_with("destiny.Ballpark.") => {
                self.dispatch_park(title, &request.operation, &request.args)
            }
            "destiny.net.server.NetworkInterface.singlecast" => {
                validate_network_request(&request, "singlecast")?;
                self.queue_network(
                    "singlecast",
                    request.args.first().cloned().unwrap_or_else(|| json!([])),
                )
            }
            "destiny.net.server.NetworkInterface.narrowcast" => {
                validate_network_request(&request, "narrowcast")?;
                self.queue_network(
                    "narrowcast",
                    request.args.first().cloned().unwrap_or_else(|| json!([])),
                )
            }
            "destiny.net.server.NetworkInterface.batch" => {
                validate_network_request(&request, "batch")?;
                self.queue_network(
                    "batch",
                    request.args.first().cloned().unwrap_or_else(|| json!({})),
                )
            }
            title => Err(CompatError::UnsupportedTitle(title.to_owned())),
        }
    }

    pub fn update(&mut self) -> Result<Value, CompatError> {
        if let Some(error) = &self.terminal_error {
            return Err(CompatError::Engine(format!(
                "runtime is unavailable after a terminal physics failure: {error}"
            )));
        }
        self.advance_one_tick(false)?;
        Ok(json!({"current_time": self.park.current_time}))
    }

    pub(crate) fn begin_callback_tick(&mut self, force: bool) -> Result<(), CompatError> {
        let phase = crate::evolution_phase::begin_tick(self.evolution_phase).ok_or_else(|| {
            CompatError::InvalidRequest("Evolve cannot reenter an active evolution".into())
        })?;
        let should_step = force || self.is_running();
        if should_step {
            self.validate_authoritative_world(true)?;
            // Preflight time before changing any control or entering the phase.
            self.park
                .current_time
                .checked_add(1)
                .ok_or_else(|| CompatError::InvalidRequest("current time overflow".into()))?;
        }
        self.evolution_phase = phase;
        let prepared = if should_step {
            self.prepare_motion_tick()
        } else {
            Ok(Vec::new())
        };
        match prepared {
            Ok(results) => {
                self.pending_tick = Some(PendingTick {
                    force: should_step,
                    results,
                });
                Ok(())
            }
            Err(error) => {
                self.evolution_phase = 0;
                Err(error)
            }
        }
    }

    pub(crate) fn take_mode_callback(&mut self) -> Option<CallCallback> {
        self.mode_callbacks.pop_front()
    }

    pub(crate) fn callback_is_live(&self, event: &CallCallback) -> bool {
        match event {
            CallCallback::Mode(event) => {
                self.ball_generations.get(&event.ball_id) == Some(&event.generation)
            }
            CallCallback::Ball {
                ball_id,
                generation,
                ..
            } => self.ball_generations.get(ball_id) == Some(generation),
            _ => true,
        }
    }

    pub(crate) fn prepend_callback_continuations(
        &mut self,
        callbacks: &mut VecDeque<CallCallback>,
    ) {
        crate::notification_batch::prepend(&mut self.mode_callbacks, callbacks);
    }

    pub(crate) fn enter_mode_callback(&mut self) -> bool {
        if let Some(phase) = crate::evolution_phase::enter_callback(self.evolution_phase) {
            self.evolution_phase = phase;
            true
        } else {
            false
        }
    }

    pub(crate) fn leave_mode_callback(&mut self, entered: bool) -> Result<(), CompatError> {
        if entered {
            self.evolution_phase = crate::evolution_phase::leave_callback(self.evolution_phase)
                .ok_or_else(|| CompatError::Engine("callback lost its evolution phase".into()))?;
        }
        Ok(())
    }

    pub(crate) fn complete_mode_callback(
        &mut self,
        event: CallCallback,
    ) -> Result<(), CompatError> {
        // Ball::SetMode sends the synchronous notification BEFORE storing mode.
        // Callback-time removal is deferred, so the ball remains live here.
        match event {
            CallCallback::Mode(event)
                if self.ball_generations.get(&event.ball_id) == Some(&event.generation) =>
            {
                self.motion.entry(event.ball_id).or_default().mode = event.new;
            }
            CallCallback::Remove {
                ball_id,
                generation,
                delay,
                step,
                arrival,
            } if self.ball_generations.get(&ball_id) == Some(&generation) => {
                self.resume_removal(ball_id, generation, delay, step, arrival)?;
            }
            CallCallback::GotoPoint {
                ball_id,
                generation,
                point,
            } if self.ball_generations.get(&ball_id) == Some(&generation) => {
                self.motion.entry(ball_id).or_default().destination = point;
                if self.metadata(ball_id)?.speed_fraction == 0.0 {
                    self.metadata_mut(ball_id)?.speed_fraction = 1.0;
                }
                self.queue_mode_change(ball_id, OriginalMode::Goto);
            }
            CallCallback::Follow {
                ball_id,
                generation,
                target,
                target_generation,
                kind,
            } if self.ball_generations.get(&ball_id) == Some(&generation)
                && self.ball_generations.get(&target) == Some(&target_generation) =>
            {
                let after_stop_check =
                    matches!(kind, FollowKind::Missile(_) | FollowKind::Formation(_));
                if after_stop_check
                    && (self.metadata(ball_id)?.new_bubble_id
                        != self.metadata(target)?.new_bubble_id
                        || self.metadata(target)?.is_cloaked != 0)
                {
                    return Ok(());
                }
                let mode = match kind {
                    FollowKind::Follow(range) | FollowKind::Orbit(range) => {
                        self.motion.entry(ball_id).or_default().follow_range = range;
                        if matches!(kind, FollowKind::Follow(_)) {
                            OriginalMode::Follow
                        } else {
                            OriginalMode::Orbit
                        }
                    }
                    FollowKind::Missile(owner) => {
                        let velocity = self
                            .app
                            .world()
                            .get::<LinearVelocity>(self.entity(ball_id)?)
                            .ok_or_else(|| CompatError::Engine("missing missile velocity".into()))?
                            .0;
                        let fields = crate::missile::follow_fields(
                            target,
                            owner,
                            self.park.current_time,
                            self.metadata(ball_id)?.radius as f32,
                            self.metadata(target)?.radius as f32,
                            OriginalVec3 {
                                x: velocity.x,
                                y: velocity.y,
                                z: velocity.z,
                            },
                        );
                        let control = self.motion.entry(ball_id).or_default();
                        control.follow_id = Some(fields.follow_id);
                        control.follow_range = fields.follow_range;
                        control.destination = fields.destination;
                        let mut metadata = self.metadata_mut(ball_id)?;
                        metadata.owner_id = fields.owner_id;
                        metadata.effect_stamp = fields.effect_stamp;
                        metadata.speed_fraction = f64::from(fields.speed_fraction);
                        OriginalMode::Missile
                    }
                    FollowKind::Formation(slot) => {
                        self.metadata_mut(ball_id)?.effect_stamp = i64::from(slot);
                        OriginalMode::Formation
                    }
                };
                self.motion.entry(ball_id).or_default().follow_id = Some(target);
                self.queue_mode_change(ball_id, mode);
                self.mode_callbacks
                    .push_back(CallCallback::RegisterFollower {
                        id: ball_id,
                        target,
                        target_generation,
                    });
            }
            CallCallback::RegisterFollower {
                id,
                target,
                target_generation,
            } if self.ball_generations.get(&target) == Some(&target_generation) => {
                self.follower_sets
                    .entry(target_generation)
                    .or_default()
                    .insert(id);
            }
            CallCallback::WarpStart {
                id,
                generation,
                point,
                min_range,
                factor,
            } if self.ball_generations.get(&id) == Some(&generation) => {
                let control = self.motion.entry(id).or_default();
                control.destination = point;
                control.min_range = min_range;
                control.warp_factor = factor;
                control.follow_range = 0.0;
                self.metadata_mut(id)?.effect_stamp = -1;
                self.metadata_mut(id)?.owner_id = i64::from(factor);
                self.queue_mode_change(id, OriginalMode::Warp);
            }
            CallCallback::EntityArrival {
                id,
                generation,
                point,
            } if self.ball_generations.get(&id) == Some(&generation) => {
                if self
                    .motion
                    .get(&id)
                    .is_some_and(|control| control.mode == OriginalMode::Goto)
                {
                    return Ok(());
                }
                self.mode_callbacks.push_back(CallCallback::Remove {
                    ball_id: id,
                    generation,
                    delay: 0,
                    step: RemovalStep::Followers,
                    arrival: Some(point),
                });
            }
            CallCallback::SetMode {
                id,
                generation,
                mode,
                troll_delay,
            } if self.ball_generations.get(&id) == Some(&generation) => {
                if let Some(delay) = troll_delay {
                    self.set_free(id, true)?;
                    self.set_interactive(id, true)?;
                    let stamp = self
                        .park
                        .current_time
                        .checked_add(i64::from(delay.max(1)))
                        .ok_or_else(|| {
                            CompatError::InvalidRequest("Troll delay overflows time".into())
                        })?;
                    self.metadata_mut(id)?.effect_stamp = stamp;
                }
                self.queue_mode_change(id, mode);
            }
            CallCallback::FormationChange {
                id,
                generation,
                formation,
                mut remaining,
            } if self.ball_generations.get(&id) == Some(&generation) => {
                if let Some((follower, follower_generation)) = remaining.pop_front() {
                    if self.ball_generations.get(&follower) == Some(&follower_generation) {
                        if formation == NO_FORMATION {
                            let control = self.motion.get(&follower).cloned().unwrap_or_default();
                            let action = crate::ball_lifecycle::follower_action(
                                true,
                                control.follow_id == Some(id),
                                mode_number(control.mode),
                                self.metadata(follower)?.is_interactive,
                            );
                            if action >= 2 {
                                if action == 3 {
                                    self.set_massive(follower, false)?;
                                }
                                let velocity = self
                                    .app
                                    .world()
                                    .get::<LinearVelocity>(self.entity(follower)?)
                                    .ok_or_else(|| {
                                        CompatError::Engine("missing follower velocity".into())
                                    })?
                                    .0;
                                self.begin_goto_direction(
                                    follower,
                                    OriginalVec3 {
                                        x: velocity.x,
                                        y: velocity.y,
                                        z: velocity.z,
                                    },
                                )?;
                            } else if action == 1 {
                                self.stop_command(follower)?;
                            } else if let Some(followers) = self.follower_sets.get_mut(&generation)
                            {
                                followers.remove(&follower);
                            }
                        } else {
                            self.stop_command(follower)?;
                        }
                    }
                    self.mode_callbacks
                        .push_back(CallCallback::FormationChange {
                            id,
                            generation,
                            formation,
                            remaining,
                        });
                } else {
                    let mut metadata = self.metadata_mut(id)?;
                    metadata.formation_id = formation;
                    if formation == NO_FORMATION {
                        metadata.formation_slots = 0;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn complete_callback_tick(&mut self) -> Result<(), CompatError> {
        self.advance_callback_tick()?;
        let mut callbacks = std::mem::take(&mut self.mode_callbacks);
        while let Some(event) = callbacks.pop_front() {
            self.complete_mode_callback(event)?;
            self.prepend_callback_continuations(&mut callbacks);
        }
        let mut pass = self.begin_notification_pass();
        while self.next_tick_notification(&mut pass)?.is_some() {}
        self.finish_callback_tick()
    }

    pub(crate) fn advance_callback_tick(&mut self) -> Result<(), CompatError> {
        let pending = self
            .pending_tick
            .take()
            .ok_or_else(|| CompatError::Engine("evolution continuation is missing".into()))?;
        let result = self.advance_one_tick_with_motion(pending.force, &pending.results);
        if result.is_err() {
            self.evolution_phase = 0;
        }
        result
    }

    pub(crate) fn finish_callback_tick(&mut self) -> Result<(), CompatError> {
        let stepped = self.pending_tick_time.is_some();
        if let Some(time) = self.pending_tick_time.take() {
            self.park.current_time = time;
        }
        self.evolution_phase = crate::evolution_phase::finish_tick(self.evolution_phase)
            .ok_or_else(|| {
                CompatError::Engine("evolution phase did not return from callbacks".into())
            })?;
        if stepped {
            self.remove_due_balls();
        }
        Ok(())
    }

    pub(crate) fn proximity_callback_is_live(&self, event: &Value) -> bool {
        let Some(owner) = event.get("owner_id").and_then(Value::as_i64) else {
            return false;
        };
        if self.park.pending_removals.contains_key(&owner) {
            return false;
        }
        let Ok(metadata) = self.metadata(owner) else {
            return false;
        };
        if event.get("kind").and_then(Value::as_str) == Some("range") {
            return self.range_notifications.contains_key(&owner);
        }
        let Some(index) = event.get("sensor_index").and_then(Value::as_u64) else {
            return false;
        };
        metadata.sensors.get(index as usize).is_some()
    }

    fn queue_mode_change(&mut self, id: i64, new: OriginalMode) {
        let old = self.motion.entry(id).or_default().mode;
        if old != new {
            self.mode_callbacks
                .push_back(CallCallback::Mode(ModeCallback {
                    ball_id: id,
                    generation: self.ball_generations[&id],
                    old_mode: mode_number(old),
                    new_mode: mode_number(new),
                    new,
                }));
        }
    }

    fn stop_command(&mut self, id: i64) -> Result<(), CompatError> {
        if !self.balls.contains_key(&id) {
            return Ok(());
        }
        let mode = self
            .motion
            .get(&id)
            .map_or(OriginalMode::Stop, |control| control.mode);
        if mode == OriginalMode::Stop {
            return Ok(());
        }
        if matches!(
            mode,
            OriginalMode::Follow
                | OriginalMode::Orbit
                | OriginalMode::Missile
                | OriginalMode::Formation
        ) {
            if let Some(generation) = self
                .motion
                .get(&id)
                .and_then(|control| control.follow_id)
                .and_then(|target| self.ball_generations.get(&target))
                .copied()
                && let Some(followers) = self.follower_sets.get_mut(&generation)
            {
                followers.remove(&id);
            }
            if self.park.is_master && mode == OriginalMode::Formation {
                let target = self.motion.get(&id).and_then(|control| control.follow_id);
                let slot = self.metadata(id)?.effect_stamp;
                if let Some(target) = target.filter(|target| self.balls.contains_key(target)) {
                    let count = self.formation_slot_count(target)?;
                    let mut metadata = self.metadata_mut(target)?;
                    metadata.formation_slots =
                        free_formation_slot(metadata.formation_slots, count, slot);
                }
            }
            let control = self.motion.entry(id).or_default();
            control.follow_id = None;
            control.follow_range = 0.0;
            let mut metadata = self.metadata_mut(id)?;
            metadata.effect_stamp = 0;
            metadata.owner_id = 0;
        }
        self.queue_mode_change(id, OriginalMode::Stop);
        Ok(())
    }

    fn followers(&self, target: i64) -> Vec<i64> {
        let mut ids = self
            .ball_generations
            .get(&target)
            .and_then(|generation| self.follower_sets.get(generation))
            .map(|followers| followers.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        ids.sort_unstable();
        ids
    }

    fn rebuild_follower_sets(&mut self) {
        self.follower_sets.clear();
        for (id, control) in &self.motion {
            if matches!(
                control.mode,
                OriginalMode::Follow
                    | OriginalMode::Orbit
                    | OriginalMode::Missile
                    | OriginalMode::Formation
            ) && let Some(generation) = control
                .follow_id
                .and_then(|target| self.ball_generations.get(&target))
                .copied()
            {
                self.follower_sets
                    .entry(generation)
                    .or_default()
                    .insert(*id);
            }
        }
    }

    fn begin_follow(&mut self, id: i64, target: i64, kind: FollowKind) -> Result<(), CompatError> {
        self.stop_command(id)?;
        self.mode_callbacks.push_back(CallCallback::Follow {
            ball_id: id,
            generation: self.ball_generations[&id],
            target,
            target_generation: self.ball_generations[&target],
            kind,
        });
        Ok(())
    }

    fn begin_goto_point(&mut self, id: i64, point: OriginalVec3) -> Result<(), CompatError> {
        if !self.balls.contains_key(&id) {
            return Ok(());
        }
        self.stop_command(id)?;
        self.mode_callbacks.push_back(CallCallback::GotoPoint {
            ball_id: id,
            generation: self.ball_generations[&id],
            point,
        });
        Ok(())
    }

    fn begin_goto_direction(
        &mut self,
        id: i64,
        direction: OriginalVec3,
    ) -> Result<(), CompatError> {
        if !self.balls.contains_key(&id) {
            return Ok(());
        }
        let position = self.position(id)?;
        let point = OriginalVec3 {
            x: position.x,
            y: position.y,
            z: position.z,
        }
        .add(direction.normalize().scale(1e17));
        self.begin_goto_point(id, point)
    }

    fn begin_warp(
        &mut self,
        id: i64,
        point: OriginalVec3,
        min_range: f64,
        factor: i32,
    ) -> Result<(), CompatError> {
        if !self.balls.contains_key(&id) {
            return Ok(());
        }
        let position = self.position(id)?;
        let distance_squared = (OriginalVec3 {
            x: position.x,
            y: position.y,
            z: position.z,
        })
        .sub(point)
        .norm_squared();
        let control = self.motion.get(&id).cloned().unwrap_or_default();
        let action = crate::warp_entry::command_action(
            distance_squared,
            control.mode == OriginalMode::Warp,
            control.destination == point,
        );
        if action == 1 {
            return self.begin_goto_point(id, point);
        }
        if action == 0 {
            return Ok(());
        }
        self.stop_command(id)?;
        self.mode_callbacks.push_back(CallCallback::WarpStart {
            id,
            generation: self.ball_generations[&id],
            point,
            min_range,
            factor,
        });
        Ok(())
    }

    fn finish_entity_warp(&mut self, id: i64, point: OriginalVec3) -> Result<(), CompatError> {
        let position = self.position(id)?;
        let control = self.motion.get(&id).cloned().unwrap_or_default();
        let (destination, velocity, effect, distance) = crate::warp_entry::arrival(
            point,
            OriginalVec3 {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            control.destination,
            control.min_range,
            self.park.current_time,
        )
        .ok_or_else(|| {
            CompatError::InvalidRequest("EntityWarpIn start time overflows i64".into())
        })?;
        self.set_massive(id, false)?;
        self.set_velocity(id, DVec3::new(velocity.x, velocity.y, velocity.z))?;
        self.metadata_mut(id)?.effect_stamp = effect;
        let control = self.motion.entry(id).or_default();
        control.destination = destination;
        control.warp_distance = distance;
        for sensor in &mut self.metadata_mut(id)?.sensors {
            if sensor.get("cloak_sensor").and_then(Value::as_bool) == Some(false) {
                sensor["active"] = json!(false);
            }
        }
        // Master PostEvent, dynamical SnapOrientation and complete original
        // sensor/engine correspondence remain required boundaries.
        Ok(())
    }

    fn original_motion_input(
        &self,
        id: i64,
        control: &MotionControl,
    ) -> Result<OriginalParkBall, CompatError> {
        let entity = self.entity(id)?;
        let metadata = self.metadata(id)?;
        let position = self.position(id)?;
        let velocity = self
            .app
            .world()
            .get::<LinearVelocity>(entity)
            .ok_or_else(|| CompatError::Engine("missing LinearVelocity".into()))?
            .0;
        let max_velocity = self
            .app
            .world()
            .get::<MaxLinearSpeed>(entity)
            .ok_or_else(|| CompatError::Engine("missing MaxLinearSpeed".into()))?
            .0;
        let mass = self
            .app
            .world()
            .get::<DestinyMass>(entity)
            .ok_or_else(|| CompatError::Engine("missing DestinyMass".into()))?
            .0;
        let agility = metadata.agility as f32;
        let effective_mass = mass * f64::from(agility);
        let friction = self.app.world().resource::<DestinySpaceFriction>().0;
        let dt = self
            .app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f64();
        let time_factor = if effective_mass <= 0.0 {
            0.0
        } else {
            (-friction * dt / effective_mass).exp()
        };
        Ok(OriginalParkBall {
            id,
            in_park: true,
            mode: control.mode,
            position: OriginalVec3 {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            velocity: OriginalVec3 {
                x: velocity.x,
                y: velocity.y,
                z: velocity.z,
            },
            mass,
            radius: metadata.radius as f32,
            time_factor,
            max_velocity: max_velocity as f32,
            agility,
            speed_fraction: metadata.speed_fraction as f32,
            goto: control.destination,
            effect_stamp: metadata.effect_stamp,
            owner_id: if control.mode == OriginalMode::Warp {
                i64::from(control.warp_factor)
            } else {
                metadata.owner_id
            },
            last_collision: control.warp_distance,
            follow_id: control.follow_id.unwrap_or(0),
            follow_range: control.follow_range,
            ..OriginalParkBall::default()
        })
    }

    fn prepare_motion_tick(&mut self) -> Result<Vec<PreparedMotion>, CompatError> {
        // Ordered controls retain the source's ID order. Numerical work is not
        // split across workers or reduced in a different floating-point order.
        let mut ids = self.balls.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        let dt = self
            .app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f64();
        let friction = self.app.world().resource::<DestinySpaceFriction>().0;
        let mut results = Vec::new();
        for id in ids {
            if !self.balls.contains_key(&id)
                || self.park.pending_removals.contains_key(&id)
                || !self.metadata(id)?.is_free
            {
                continue;
            }
            let mut control = self.motion.get(&id).cloned().unwrap_or_default();
            let mut ball = self.original_motion_input(id, &control)?;
            if control.mode == OriginalMode::Stop {
                if !self.park.use_dynamical_orientation {
                    // EvolveStop is distinct from the Stop command: the old
                    // source damps only Y before gradient/integration work.
                    let velocity = destiny_original_spec::original_stop_velocity(ball.velocity);
                    self.set_velocity(id, DVec3::new(velocity.x, velocity.y, velocity.z))?;
                }
                continue;
            }
            if control.mode == OriginalMode::Mushroom {
                continue;
            }
            if control.mode == OriginalMode::Warp && ball.effect_stamp < 0 {
                if original_aligned_for_warp(&ball) {
                    let offset = ball
                        .position
                        .sub(ball.goto)
                        .normalize()
                        .scale(control.min_range);
                    ball.goto = ball.goto.add(offset);
                    ball.effect_stamp = self.park.current_time;
                    ball.last_collision = ball.goto.sub(ball.position).length();
                    control.destination = ball.goto;
                    control.warp_distance = ball.last_collision;
                    self.metadata_mut(id)?.effect_stamp = ball.effect_stamp;
                    self.set_massive(id, false)?;
                } else {
                    ball.effect_stamp = ball.effect_stamp.checked_sub(1).ok_or_else(|| {
                        CompatError::InvalidRequest("warp alignment counter overflow".into())
                    })?;
                    self.metadata_mut(id)?.effect_stamp = ball.effect_stamp;
                }
                self.motion.insert(id, control.clone());
            }
            let in_warp = control.mode == OriginalMode::Warp && ball.effect_stamp >= 0;
            let mut acceleration = OriginalVec3::ZERO;
            let (position, velocity) = if in_warp {
                let elapsed = self
                    .park
                    .current_time
                    .checked_sub(ball.effect_stamp)
                    .ok_or_else(|| {
                        CompatError::InvalidRequest("warp elapsed time overflow".into())
                    })?;
                let (position, velocity, stop) = original_warp_distance(&ball, elapsed as f64 * dt);
                if stop {
                    self.set_massive(id, true)?;
                    self.queue_mode_change(id, OriginalMode::Stop);
                    original_integrate_duration(
                        position,
                        velocity,
                        OriginalVec3::ZERO,
                        ball.mass * f64::from(ball.agility),
                        friction,
                        ball.time_factor,
                        dt,
                        dt,
                    )
                } else {
                    (position, velocity)
                }
            } else {
                acceleration = if matches!(control.mode, OriginalMode::Follow | OriginalMode::Orbit)
                {
                    let target = control.follow_id.ok_or_else(|| {
                        CompatError::Engine("following mode lost its target".into())
                    })?;
                    if !self.balls.contains_key(&target) {
                        self.queue_mode_change(id, OriginalMode::Stop);
                        OriginalVec3::ZERO
                    } else {
                        let target_control = self.motion.get(&target).cloned().unwrap_or_default();
                        let other = self.original_motion_input(target, &target_control)?;
                        let acceleration = if control.mode == OriginalMode::Follow {
                            original_follow_thrust_with_environment(&mut ball, &other, friction, dt)
                        } else if self.park.use_new_orbit {
                            original_new_orbit_thrust_with_environment(
                                &mut ball,
                                &other,
                                self.park.current_time,
                                friction,
                                dt,
                            )
                            .map_err(|error| {
                                CompatError::InvalidRequest(format!(
                                    "original new orbit kernel rejected state: {error:?}"
                                ))
                            })?
                        } else {
                            original_old_orbit_thrust_with_friction(
                                &mut ball,
                                &other,
                                self.park.current_time,
                                friction,
                            )
                            .map_err(|error| {
                                CompatError::InvalidRequest(format!(
                                    "original orbit kernel rejected state: {error:?}"
                                ))
                            })?
                        };
                        control.destination = ball.goto;
                        self.motion.insert(id, control.clone());
                        acceleration
                    }
                } else if control.mode == OriginalMode::Goto {
                    original_goto_thrust(&ball, ball.goto, friction, dt, false)
                } else {
                    OriginalVec3::ZERO
                };
                original_integrate_duration(
                    ball.position,
                    ball.velocity,
                    acceleration,
                    ball.mass * f64::from(ball.agility),
                    friction,
                    ball.time_factor,
                    dt,
                    dt,
                )
            };
            if !solver_safe_vec3(
                DVec3::new(position.x, position.y, position.z),
                MAX_COORDINATE,
            ) || !solver_safe_vec3(DVec3::new(velocity.x, velocity.y, velocity.z), MAX_VELOCITY)
            {
                return Err(CompatError::InvalidRequest(
                    "motion result exceeds the solver-safe envelope".into(),
                ));
            }
            results.push(PreparedMotion {
                id,
                position,
                velocity,
                acceleration,
                in_warp,
            });
        }
        Ok(results)
    }

    fn advance_one_tick(&mut self, force: bool) -> Result<(), CompatError> {
        self.begin_callback_tick(force)?;
        let mut callbacks = std::mem::take(&mut self.mode_callbacks);
        while let Some(event) = callbacks.pop_front() {
            self.complete_mode_callback(event)?;
            self.prepend_callback_continuations(&mut callbacks);
        }
        self.complete_callback_tick()
    }

    fn prepare_iterative_sphere_tick(
        &self,
        motion: &[PreparedMotion],
    ) -> Result<Option<Vec<PreparedMotion>>, CompatError> {
        // In the pinned old-orientation iterative path, GetNextCollisionStatic
        // and GetNextCollisionFree test parent spheres, including parents with
        // mini balls. Capsules/boxes are separate static collidables attached
        // only to fixed parents, using translation rather than ship rotation.
        if self.park.use_dynamical_orientation {
            return Ok(None);
        }
        let dt = self
            .app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f64();
        let friction = self.app.world().resource::<DestinySpaceFriction>().0;
        if friction == 0.0
            || friction > f64::MAX.sqrt()
            || self
                .balls
                .keys()
                .any(|id| self.metadata(*id).is_ok_and(|m| m.new_bubble_id > 0))
        {
            // Keep the established native zero/extreme-friction and host-bubble
            // paths. Source arithmetic/partition correspondence for these
            // partitions remains explicit; they receive no original proof credit.
            return Ok(None);
        }
        let iterations = self.app.world().resource::<SubstepCount>().0 as usize;
        let prepared: HashMap<_, _> = motion.iter().map(|step| (step.id, *step)).collect();
        let mut ids = self.balls.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        let mut bodies = Vec::with_capacity(ids.len());
        let mut statics = Vec::new();
        for &id in &ids {
            let control = self.motion.get(&id).cloned().unwrap_or_default();
            let ball = self.original_motion_input(id, &control)?;
            let metadata = self.metadata(id)?;
            let step = prepared.get(&id);
            bodies.push(crate::iterative_collision::Body {
                id,
                position: ball.position,
                velocity: ball.velocity,
                acceleration: step.map_or(OriginalVec3::ZERO, |s| s.acceleration),
                collision_acceleration: OriginalVec3::ZERO,
                mass: ball.mass,
                agility: ball.agility,
                radius: ball.radius,
                max_velocity: ball.max_velocity,
                time_factor: ball.time_factor,
                is_free: metadata.is_free,
                is_massive: metadata.is_massive,
                is_cloaked: metadata.is_cloaked != 0,
                is_global: metadata.is_global,
                is_space_junk: metadata.is_space_junk,
                is_moribund: self.park.pending_removals.contains_key(&id),
                bubble: metadata.new_bubble_id,
                mode: control.mode,
                owner_id: ball.owner_id,
                follow_id: ball.follow_id,
            });
            if !metadata.is_free && !self.park.pending_removals.contains_key(&id) {
                for (mini_index, mini) in metadata.minis.iter().enumerate() {
                    let collider_id = metadata.mini_ids.get(mini_index).copied().unwrap_or(0);
                    let convert = |v: DVec3| OriginalVec3 {
                        x: v.x,
                        y: v.y,
                        z: v.z,
                    };
                    let shape = match mini.get("kind").and_then(Value::as_str) {
                        Some("capsule") => {
                            let a = ball
                                .position
                                .add(convert(value_vec3(mini.get("a"), "capsule a")?));
                            let b = ball
                                .position
                                .add(convert(value_vec3(mini.get("b"), "capsule b")?));
                            let position = a.add(b).scale(0.5);
                            destiny_original_spec::OriginalStaticCollisionShape::Capsule(
                                destiny_original_spec::OriginalCapsule {
                                    id: collider_id,
                                    parent_id: id,
                                    a,
                                    b,
                                    radius: value_f64(mini.get("radius"), "capsule radius")? as f32,
                                    ab: b.sub(a),
                                    position,
                                    old_position: position,
                                },
                            )
                        }
                        Some("box") => {
                            let values =
                                mini.get("basis").and_then(Value::as_array).ok_or_else(|| {
                                    CompatError::Engine("missing mini box basis".into())
                                })?;
                            let triple = |index: usize| -> Result<OriginalVec3, CompatError> {
                                Ok(OriginalVec3 {
                                    x: value_f64(values.get(index), "box x")?,
                                    y: value_f64(values.get(index + 1), "box y")?,
                                    z: value_f64(values.get(index + 2), "box z")?,
                                })
                            };
                            let shape = destiny_original_spec::BoxShape::new(
                                ball.position.add(triple(0)?),
                                triple(3)?,
                                triple(6)?,
                                triple(9)?,
                            );
                            let position = shape.center();
                            let mut aabb = destiny_original_spec::Aabb::new(
                                shape.corners[0],
                                shape.corners[0],
                            );
                            for &corner in &shape.corners[1..] {
                                aabb.update(corner);
                            }
                            destiny_original_spec::OriginalStaticCollisionShape::Box(
                                destiny_original_spec::OriginalOrientedBox {
                                    id: collider_id,
                                    parent_id: id,
                                    bounding_radius: shape.bounding_radius() as f32,
                                    position,
                                    old_position: position,
                                    aabb,
                                    shape,
                                },
                            )
                        }
                        Some("sphere") => continue,
                        _ => {
                            return Err(CompatError::Engine(
                                "unknown native mini geometry kind".into(),
                            ));
                        }
                    };
                    statics.push(crate::iterative_collision::StaticGeometry { owner: id, shape });
                }
            }
        }
        statics.sort_by_key(|geometry| geometry.shape.id());
        let results = if self.park.use_iterative_collision {
            crate::iterative_collision::resolve(
                &mut bodies,
                &statics,
                friction,
                dt,
                iterations,
                self.park.is_master,
            )
        } else {
            crate::noniterative_collision::resolve(
                &bodies,
                &statics,
                friction,
                dt,
                self.park.is_master,
            )
            .map_err(|error| {
                CompatError::Engine(format!("source gradient rejected scratch state: {error:?}"))
            })?
        };
        let mut publication = Vec::with_capacity(results.len());
        for (id, result) in ids.into_iter().zip(results) {
            if let Some(step) = prepared.get(&id).filter(|step| step.in_warp) {
                publication.push(*step);
            } else if let Some((position, velocity)) = result {
                if !solver_safe_vec3(
                    DVec3::new(position.x, position.y, position.z),
                    MAX_COORDINATE,
                ) || !solver_safe_vec3(
                    DVec3::new(velocity.x, velocity.y, velocity.z),
                    MAX_VELOCITY,
                ) {
                    return Err(CompatError::InvalidRequest(
                        "original sphere result exceeds solver-safe envelope".into(),
                    ));
                }
                publication.push(PreparedMotion {
                    id,
                    position,
                    velocity,
                    acceleration: OriginalVec3::ZERO,
                    in_warp: false,
                });
            }
        }
        Ok(Some(publication))
    }

    fn advance_one_tick_with_motion(
        &mut self,
        force: bool,
        motion: &[PreparedMotion],
    ) -> Result<(), CompatError> {
        let was_paused = !self.is_running();
        let should_step = force || !was_paused;
        let next_time = should_step
            .then(|| {
                self.park
                    .current_time
                    .checked_add(1)
                    .ok_or_else(|| CompatError::InvalidRequest("current time overflow".into()))
            })
            .transpose()?;
        let checkpoint = if should_step {
            self.validate_authoritative_world(true)?;
            Some(self.capture_evolution_checkpoint()?)
        } else {
            None
        };
        if should_step {
            // Pair filtering needs the standalone park's first compatibility
            // bubble before the solver runs. Assigning it only afterward made
            // every first-tick pair with sentinel -1 fail the collision hook.
            self.update_single_compatibility_bubble();
            self.configure_destiny_space_damping()?;
        }
        let iterative = if should_step {
            self.prepare_iterative_sphere_tick(motion)?
        } else {
            None
        };
        let mushrooms = if should_step {
            self.prepare_mushroom_tick()?
        } else {
            Vec::new()
        };
        if force && was_paused {
            self.app
                .world_mut()
                .resource_mut::<Time<Physics>>()
                .unpause();
        }
        self.app.update();
        if should_step {
            let step_result = if let Some(checkpoint) = checkpoint.as_ref() {
                self.correct_collision_free_analytic_motion(checkpoint)
                    .and_then(|()| {
                        for &PreparedMotion {
                            id,
                            position,
                            velocity,
                            ..
                        } in iterative.as_deref().unwrap_or(motion)
                        {
                            if self.balls.contains_key(&id)
                                && !self.park.pending_removals.contains_key(&id)
                                && !self
                                    .motion
                                    .get(&id)
                                    .is_some_and(|control| control.mode == OriginalMode::Mushroom)
                            {
                                self.set_position(
                                    id,
                                    DVec3::new(position.x, position.y, position.z),
                                )?;
                                self.set_velocity(
                                    id,
                                    DVec3::new(velocity.x, velocity.y, velocity.z),
                                )?;
                            }
                        }
                        for (id, radius) in &mushrooms {
                            match radius {
                                Some(radius) => self.set_radius(*id, f64::from(*radius))?,
                                None => self.schedule_remove_ball(*id, 0)?,
                            }
                        }
                        Ok(())
                    })
                    .and_then(|()| self.validate_authoritative_world(false))
            } else {
                self.validate_authoritative_world(false)
            };
            if let Err(error) = step_result {
                if let Some(checkpoint) = checkpoint.as_ref()
                    && let Err(rollback_error) = self.restore_evolution_checkpoint(checkpoint)
                {
                    self.app.world_mut().resource_mut::<Time<Physics>>().pause();
                    let message = format!(
                        "physics step failed ({error}); rollback also failed ({rollback_error})"
                    );
                    self.terminal_error = Some(message.clone());
                    return Err(CompatError::Engine(message));
                }
                self.app.world_mut().resource_mut::<Time<Physics>>().pause();
                // Avian contact manifolds, sleeping islands, and broad-phase
                // caches are intentionally not part of the logical checkpoint.
                // Even when every exposed authoritative component is restored,
                // continuing would claim a stronger rollback than we can prove.
                let message = format!(
                    "physics step rejected ({error}); authoritative state was restored but the runtime was disabled because solver caches are not rollback-safe"
                );
                self.terminal_error = Some(message.clone());
                return Err(CompatError::Engine(message));
            }
            self.sync_visual_transforms();
            self.update_single_compatibility_bubble();
            self.pending_tick_time = Some(next_time.ok_or_else(|| {
                CompatError::Engine("stepping update lost its validated time".into())
            })?);
        }
        if force && was_paused {
            self.app
                .world_mut()
                .resource_mut::<Time<Physics>>()
                .advance_by(Duration::ZERO);
            self.app.world_mut().resource_mut::<Time<Physics>>().pause();
        }
        Ok(())
    }

    /// The standalone ABI has no product bubble manager. After the first
    /// evolution it therefore creates one deterministic compatibility bubble
    /// for still-unassigned balls. A Rust host can assign real non-negative
    /// bubble IDs through its own systems; those values are never overwritten.
    fn update_single_compatibility_bubble(&mut self) {
        let entities = self.balls.values().copied().collect::<Vec<_>>();
        let world = self.app.world_mut();
        for entity in entities {
            let Some(mut metadata) = world.get_mut::<DestinyBallMetadata>(entity) else {
                continue;
            };
            if metadata.new_bubble_id == -1 {
                metadata.old_bubble_id = -1;
                metadata.new_bubble_id = 0;
            }
        }
    }

    fn prepare_mushroom_tick(&self) -> Result<Vec<(i64, Option<f32>)>, CompatError> {
        let mut ids = self
            .motion
            .iter()
            .filter_map(|(id, control)| {
                (control.mode == OriginalMode::Mushroom
                    && self.balls.contains_key(id)
                    && !self.park.pending_removals.contains_key(id))
                .then_some(*id)
            })
            .collect::<Vec<_>>();
        ids.sort_unstable();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let interval = self
            .app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f64()
            * 1000.0;
        if !interval.is_finite() || interval > i32::MAX as f64 {
            return Err(CompatError::InvalidRequest(
                "mushroom tick interval exceeds original i32 domain".into(),
            ));
        }
        let mut updates = Vec::new();
        for id in ids {
            let metadata = self.metadata(id)?;
            if metadata.is_free {
                let control = &self.motion[&id];
                let radius = crate::mushroom::radius_at_tick(
                    self.park.current_time,
                    metadata.effect_stamp,
                    interval as i32,
                    control.follow_range,
                    control.destination.x,
                )
                .map_err(|error| CompatError::InvalidRequest(error.into()))?;
                if radius.is_some_and(|value| {
                    !value.is_finite() || value < 0.0 || f64::from(value) > MAX_RADIUS
                }) {
                    return Err(CompatError::InvalidRequest(
                        "mushroom radius exceeds solver envelope".into(),
                    ));
                }
                updates.push((id, radius));
            }
        }
        Ok(updates)
    }

    fn source_bubble_ball(&self, id: i64) -> Result<crate::bubbles::Ball, CompatError> {
        let metadata = self.metadata(id)?;
        Ok(crate::bubbles::Ball {
            id,
            old: metadata.old_bubble_id,
            new: metadata.new_bubble_id,
            interactive: metadata.is_interactive,
            global: metadata.is_global,
            cloaked: metadata.is_cloaked != 0,
            troll: self
                .motion
                .get(&id)
                .is_some_and(|control| control.mode == OriginalMode::Troll),
        })
    }

    fn initialize_source_bubbles(&mut self) -> Result<(), CompatError> {
        if self.park.is_master {
            return Err(CompatError::InvalidRequest(
                "source master partition insertion remains unimplemented".into(),
            ));
        }
        let mut ids = self.balls.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        let mut spheres = Vec::with_capacity(ids.len());
        for id in ids {
            let metadata = self.metadata(id)?;
            if metadata.minis.iter().any(|mini| mini["kind"] != "sphere") {
                return Err(CompatError::InvalidRequest(
                    "source static-shape partition insertion remains unimplemented".into(),
                ));
            }
            let control = self.motion.get(&id).cloned().unwrap_or_default();
            let ball = self.original_motion_input(id, &control)?;
            let inflated = if metadata.is_free {
                let m = ball.mass * f64::from(ball.agility);
                let friction = self.app.world().resource::<DestinySpaceFriction>().0;
                let acceleration =
                    friction * f64::from(ball.speed_fraction) * f64::from(ball.max_velocity) / m;
                f64::from(ball.radius)
                    + (m * acceleration
                        - (m * acceleration - ball.velocity.length() * friction) * ball.time_factor)
                        / friction
            } else {
                f64::from(ball.radius)
            };
            spheres.push((id, ball.position, ball.radius, inflated));
        }
        let (mut tracking, labels) = crate::bubbles::initialize(&spheres)
            .map_err(|error| CompatError::InvalidRequest(error.into()))?;
        for (id, label) in labels {
            let mut ball = self.source_bubble_ball(id)?;
            ball.old = ball.new;
            ball.new = label;
            tracking.transition(ball);
            let mut metadata = self.metadata_mut(id)?;
            metadata.old_bubble_id = ball.old;
            metadata.new_bubble_id = label;
        }
        self.original_bubbles = Some(tracking);
        Ok(())
    }

    fn update_source_ball_bubble(&mut self, id: i64) -> Result<(), CompatError> {
        if !self.balls.contains_key(&id) {
            return Ok(());
        }
        let position = self.position(id)?;
        let mut label = self.metadata(id)?.new_bubble_id;
        let mut high = -1_i64;
        let mut ids = self.balls.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        for other in ids {
            let other_label = self.metadata(other)?.new_bubble_id;
            high = high.max(other_label);
            if other == id || other_label == -1 {
                continue;
            }
            let point = self.position(other)?;
            let dx = position.x - point.x;
            let dy = position.y - point.y;
            let dz = position.z - point.z;
            if dx * dx + dy * dy + dz * dz < 7_864_320.0 * 7_864_320.0 {
                label = other_label;
                break;
            }
        }
        if label == -1 {
            label = high
                .checked_add(1)
                .ok_or_else(|| CompatError::InvalidRequest("bubble ID overflow".into()))?;
        }
        // Source UpdateBallBubble changes neither oldBubble nor Python bubble
        // tracking dictionaries. Full InitializeBubbles is a different path.
        self.metadata_mut(id)?.new_bubble_id = label;
        Ok(())
    }

    fn source_bubble_updates(
        &mut self,
        users: &[Value],
        aliases: [usize; 4],
    ) -> Result<Value, CompatError> {
        let mut ids = self.balls.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        let mut balls = ids
            .into_iter()
            .map(|id| self.source_bubble_ball(id))
            .collect::<Result<Vec<_>, _>>()?;
        let users = users.iter().map(Value::as_i64).collect::<Vec<_>>();
        let tracking = self
            .original_bubbles
            .as_ref()
            .ok_or_else(|| CompatError::InvalidRequest("Bubbles not initialized yet".into()))?;
        let updates = if aliases == [0, 1, 2, 3] {
            tracking.updates(&mut balls, &users)
        } else {
            tracking
                .updates_with_aliases(&mut balls, &users, aliases)
                .map_err(|error| CompatError::InvalidRequest(error.into()))?
        };
        for ball in balls {
            self.metadata_mut(ball.id)?.old_bubble_id = ball.old;
        }
        let dictionaries = updates.dictionaries.map(|map| {
            map.into_iter()
                .map(|(id, delta)| {
                    let value = match delta {
                        crate::bubbles::Delta::Bubble(bubble) => json!(bubble),
                        crate::bubbles::Delta::Balls(ids) => json!(ids),
                    };
                    json!([id, value])
                })
                .collect::<Vec<_>>()
        });
        let payload = json!({"dictionaries":dictionaries,"transitions":updates.transitions,"invalid_user_index":updates.invalid_user_index});
        if updates.invalid_user_index.is_none() {
            self.mode_callbacks.push_back(CallCallback::Park {
                kind: 14,
                arguments: json!([payload["transitions"], payload["dictionaries"]]),
            });
        }
        Ok(payload)
    }

    /// Exposes the finalized compatibility app for inspection.
    pub fn app(&self) -> &App {
        &self.app
    }

    /// Exposes the finalized app for resource/system configuration. Plugins
    /// must be installed with [`Self::new_with_app`] before finalization.
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    pub fn world(&self) -> &World {
        self.app.world()
    }

    pub fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }

    fn queue_network(&mut self, expected: &str, updates: Value) -> Result<Value, CompatError> {
        let object = updates.as_object().ok_or_else(|| {
            CompatError::InvalidRequest("Carbon network envelope must be an object".into())
        })?;
        if object.len() != 5
            || !object.keys().all(|key| {
                matches!(
                    key.as_str(),
                    "protocol" | "schema_version" | "mode" | "batch_id" | "updates"
                )
            })
            || object.get("protocol").and_then(Value::as_str) != Some("destiny-carbon-update")
            || object.get("schema_version").and_then(Value::as_u64) != Some(2)
            || object.get("mode").and_then(Value::as_str) != Some(expected)
            || object
                .get("batch_id")
                .and_then(Value::as_i64)
                .is_none_or(|batch_id| batch_id <= 0)
            || if expected == "batch" {
                !object.get("updates").is_some_and(|value| {
                    value.as_object().is_some_and(|batch| {
                        batch.len() == 2
                            && batch.get("singlecasts").is_some_and(Value::is_array)
                            && batch.get("narrowcasts").is_some_and(Value::is_array)
                    })
                })
            } else {
                !object.get("updates").is_some_and(Value::is_array)
            }
        {
            return Err(CompatError::InvalidRequest(
                "invalid Carbon network protocol envelope".into(),
            ));
        }
        validate_queued_network_rows(object.get("updates").unwrap_or(&Value::Null), expected)?;
        let mut outbox = self.app.world_mut().resource_mut::<CarbonNetworkOutbox>();
        let batch_id = object
            .get("batch_id")
            .and_then(Value::as_i64)
            .ok_or_else(|| CompatError::InvalidRequest("invalid Carbon batch identifier".into()))?;
        if batch_id == outbox.last_batch_id {
            if outbox.last_batch_envelope.as_ref() == Some(&updates) {
                return Ok(Value::Null);
            }
            return Err(CompatError::InvalidRequest(
                "Carbon batch identifier conflicts with a different prior envelope".into(),
            ));
        }
        if batch_id < outbox.last_batch_id {
            return Err(CompatError::InvalidRequest(
                "Carbon batch identifiers must increase monotonically per runtime".into(),
            ));
        }
        if outbox
            .singlecasts
            .len()
            .saturating_add(outbox.narrowcasts.len())
            .saturating_add(outbox.batches.len())
            >= self.park.limits.max_outbox_messages
        {
            return Err(CompatError::InvalidRequest(
                "Carbon network outbox is full".into(),
            ));
        }
        let remaining = self
            .park
            .limits
            .max_outbox_bytes
            .saturating_sub(outbox.queued_bytes);
        let encoded_len = json_encoded_len(&updates, remaining).map_err(|error| {
            CompatError::InvalidRequest(format!("invalid or oversized Carbon envelope: {error}"))
        })?;
        outbox.queued_bytes += encoded_len;
        outbox.last_batch_envelope = Some(updates.clone());
        match expected {
            "singlecast" => outbox.singlecasts.push(updates),
            "narrowcast" => outbox.narrowcasts.push(updates),
            "batch" => outbox.batches.push(updates),
            _ => unreachable!("validated network mode"),
        }
        outbox.last_batch_id = batch_id;
        Ok(Value::Null)
    }

    pub(crate) fn is_running(&self) -> bool {
        !self.app.world().resource::<Time<Physics>>().is_paused()
    }

    pub(crate) fn current_time(&self) -> i64 {
        self.park.current_time
    }

    fn configure_destiny_space_damping(&mut self) -> Result<(), CompatError> {
        let friction = self.app.world().resource::<DestinySpaceFriction>().0;
        let dt = self
            .app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f64();
        let pending_removals = self
            .park
            .pending_removals
            .keys()
            .copied()
            .collect::<HashSet<_>>();
        let ball_capacity = self.balls.len();
        let plan = {
            let world = self.app.world_mut();
            let mut query = world.query::<(
                Entity,
                &DestinyBallId,
                &DestinyBallMetadata,
                &DestinyMass,
                &MaxLinearSpeed,
                &LinearVelocity,
            )>();
            let mut plan = Vec::with_capacity(ball_capacity);
            for (entity, ball_id, metadata, mass, max_speed, velocity) in query.iter(world) {
                let mut velocity = velocity.0;
                if pending_removals.contains(&ball_id.0) || !metadata.is_free {
                    plan.push((entity, velocity, 0.0));
                    continue;
                }
                let speed = stable_vec3_length(velocity);
                if speed > max_speed.0 && speed > 0.0 {
                    velocity = stable_vec3_normalize(velocity)
                        .map_or(DVec3::ZERO, |direction| direction * max_speed.0);
                }
                // Avian integrates LinearDamping inside its solver stages, so
                // collision impulses are not pre-scaled or post-scaled. The
                // coefficient matches Destiny's exponential free-flight decay
                // for Avian's implicit v/(1+d*dt) damping update.
                if mass.0 <= 0.0 {
                    // A zero-mass free body cannot accept Destiny's
                    // mass-scaled friction. Keep it finite and stationary.
                    plan.push((entity, DVec3::ZERO, 0.0));
                    continue;
                }
                let rate = if friction <= 0.0 {
                    0.0
                } else {
                    friction / mass.0 / metadata.agility
                };
                let coefficient = if rate == 0.0 {
                    0.0
                } else {
                    let substeps = if self.park.use_iterative_collision {
                        self.park.collision_substeps
                    } else {
                        1
                    } as f64;
                    let substep_dt = dt / substeps;
                    let exponent = rate * substep_dt;
                    if !exponent.is_finite() || exponent > f64::MAX.ln() {
                        // The exact Destiny solution has underflowed to zero
                        // velocity. Staging zero directly avoids manufacturing
                        // an infinite Avian damping coefficient; the analytic
                        // correction below still applies the finite integral
                        // displacement when the full-tick exponent is finite.
                        plan.push((entity, DVec3::ZERO, 0.0));
                        continue;
                    }
                    exponent.exp_m1() / substep_dt
                };
                if !coefficient.is_finite() || coefficient < 0.0 {
                    return Err(CompatError::InvalidRequest(
                        "space damping coefficient exceeds the solver-safe range".into(),
                    ));
                }
                plan.push((entity, velocity, coefficient));
            }
            plan
        };
        let world = self.app.world_mut();
        for (entity, velocity, damping) in plan {
            world
                .entity_mut(entity)
                .insert((LinearVelocity(velocity), LinearDamping(damping)));
        }
        Ok(())
    }

    fn capture_evolution_checkpoint(&self) -> Result<EvolutionCheckpoint, CompatError> {
        let world = self.app.world();
        let mut balls = Vec::with_capacity(self.balls.len());
        for entity in self.balls.values().copied() {
            balls.push(EvolutionBallCheckpoint {
                entity,
                ball_id: *world
                    .get::<DestinyBallId>(entity)
                    .ok_or_else(|| CompatError::Engine("missing DestinyBallId".into()))?,
                metadata: world
                    .get::<DestinyBallMetadata>(entity)
                    .ok_or_else(|| CompatError::Engine("missing DestinyBallMetadata".into()))?
                    .clone(),
                destiny_mass: *world
                    .get::<DestinyMass>(entity)
                    .ok_or_else(|| CompatError::Engine("missing DestinyMass".into()))?,
                avian_mass: *world
                    .get::<Mass>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Mass".into()))?,
                max_linear_speed: *world
                    .get::<MaxLinearSpeed>(entity)
                    .ok_or_else(|| CompatError::Engine("missing MaxLinearSpeed".into()))?,
                max_angular_speed: *world
                    .get::<MaxAngularSpeed>(entity)
                    .ok_or_else(|| CompatError::Engine("missing MaxAngularSpeed".into()))?,
                rigid_body: *world
                    .get::<RigidBody>(entity)
                    .ok_or_else(|| CompatError::Engine("missing RigidBody".into()))?,
                collider: world
                    .get::<Collider>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Collider".into()))?
                    .clone(),
                collider_disabled: world.get::<ColliderDisabled>(entity).is_some(),
                gravity_scale: *world
                    .get::<GravityScale>(entity)
                    .ok_or_else(|| CompatError::Engine("missing GravityScale".into()))?,
                pending_removal: world.get::<DestinyPendingRemoval>(entity).cloned(),
                position: *world
                    .get::<Position>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Position".into()))?,
                rotation: *world
                    .get::<Rotation>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Rotation".into()))?,
                linear_velocity: *world
                    .get::<LinearVelocity>(entity)
                    .ok_or_else(|| CompatError::Engine("missing LinearVelocity".into()))?,
                angular_velocity: *world
                    .get::<AngularVelocity>(entity)
                    .ok_or_else(|| CompatError::Engine("missing AngularVelocity".into()))?,
                linear_damping: *world
                    .get::<LinearDamping>(entity)
                    .ok_or_else(|| CompatError::Engine("missing LinearDamping".into()))?,
                transform: *world
                    .get::<Transform>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Transform".into()))?,
            });
        }
        Ok(EvolutionCheckpoint {
            follower_sets: self.follower_sets.clone(),
            balls,
            current_time: self.park.current_time,
            time: self.park.time,
            fixed_time: *world.resource::<Time<Fixed>>(),
            physics_time: *world.resource::<Time<Physics>>(),
            substeps_time: *world.resource::<Time<Substeps>>(),
            proximity_events: world.resource::<ProximityEventOutbox>().clone(),
            network_outbox: world.resource::<CarbonNetworkOutbox>().clone(),
            motion: self.motion.clone(),
            notifications: self.range_notifications.clone(),
        })
    }

    fn restore_evolution_checkpoint(
        &mut self,
        checkpoint: &EvolutionCheckpoint,
    ) -> Result<(), CompatError> {
        self.motion = checkpoint.motion.clone();
        self.follower_sets = checkpoint.follower_sets.clone();
        self.range_notifications = checkpoint.notifications.clone();
        let world = self.app.world_mut();
        *world.resource_mut::<Time<Fixed>>() = checkpoint.fixed_time;
        *world.resource_mut::<Time<Physics>>() = checkpoint.physics_time;
        *world.resource_mut::<Time<Substeps>>() = checkpoint.substeps_time;
        *world.resource_mut::<ProximityEventOutbox>() = checkpoint.proximity_events.clone();
        *world.resource_mut::<CarbonNetworkOutbox>() = checkpoint.network_outbox.clone();
        for ball in &checkpoint.balls {
            let mut entity = world.get_entity_mut(ball.entity).map_err(|_| {
                CompatError::Engine("physics step changed the authoritative entity set".into())
            })?;
            entity.insert((
                ball.ball_id,
                ball.metadata.clone(),
                ball.destiny_mass,
                ball.avian_mass,
                ball.max_linear_speed,
                ball.max_angular_speed,
                ball.rigid_body,
                ball.collider.clone(),
                ball.gravity_scale,
                ball.position,
                ball.rotation,
                ball.linear_velocity,
                ball.angular_velocity,
                ball.linear_damping,
                ball.transform,
            ));
            if ball.collider_disabled {
                entity.insert(ColliderDisabled);
            } else {
                entity.remove::<ColliderDisabled>();
            }
            if let Some(pending) = &ball.pending_removal {
                entity.insert(pending.clone());
            } else {
                entity.remove::<DestinyPendingRemoval>();
            }
        }
        self.park.current_time = checkpoint.current_time;
        self.park.time = checkpoint.time;
        self.sync_visual_transforms();
        Ok(())
    }

    fn correct_collision_free_analytic_motion(
        &mut self,
        checkpoint: &EvolutionCheckpoint,
    ) -> Result<(), CompatError> {
        let friction = self.app.world().resource::<DestinySpaceFriction>().0;
        let dt = self
            .app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f64();
        let world = self.app.world_mut();
        for initial in &checkpoint.balls {
            let Some((is_free, agility)) = world
                .get::<DestinyBallMetadata>(initial.entity)
                .map(|metadata| (metadata.is_free, metadata.agility))
            else {
                continue;
            };
            let Some(mass) = world.get::<DestinyMass>(initial.entity).map(|mass| mass.0) else {
                continue;
            };
            let has_contact = world
                .get::<CollidingEntities>(initial.entity)
                .is_some_and(|entities| !entities.is_empty());
            if !is_free || mass <= 0.0 || has_contact || initial.pending_removal.is_some() {
                continue;
            }
            let rate = if friction <= 0.0 {
                0.0
            } else {
                friction / mass / agility
            };
            let x = rate * dt;
            let decay = (-x).exp();
            let mut initial_velocity = initial.linear_velocity.0;
            let initial_speed = stable_vec3_length(initial_velocity);
            if initial_speed > initial.max_linear_speed.0 && initial_speed > 0.0 {
                initial_velocity = stable_vec3_normalize(initial_velocity)
                    .map_or(DVec3::ZERO, |direction| {
                        direction * initial.max_linear_speed.0
                    });
            }
            let expected_velocity = initial_velocity * decay;
            let actual_velocity = world
                .get::<LinearVelocity>(initial.entity)
                .ok_or_else(|| CompatError::Engine("missing LinearVelocity".into()))?
                .0;
            let difference = stable_vec3_length(actual_velocity - expected_velocity);
            let tolerance = 1.0e-10
                * (1.0
                    + stable_vec3_length(expected_velocity)
                        .max(stable_vec3_length(actual_velocity)));
            if difference > tolerance {
                // A solver impulse or host force changed this body. Its Avian
                // result is authoritative and must not receive a free-flight
                // displacement correction.
                continue;
            }
            let displacement_scale = if x.abs() <= f64::EPSILON {
                1.0
            } else {
                -(-x).exp_m1() / x
            };
            let position = initial.position.0 + initial_velocity * displacement_scale * dt;
            if !solver_safe_vec3(position, MAX_COORDINATE) {
                return Err(CompatError::InvalidRequest(
                    "analytic free-flight correction exceeded solver bounds".into(),
                ));
            }
            world
                .get_mut::<Position>(initial.entity)
                .ok_or_else(|| CompatError::Engine("missing Position".into()))?
                .0 = position;
        }
        Ok(())
    }

    fn validate_authoritative_world(&self, preflight: bool) -> Result<(), CompatError> {
        let dt = self
            .app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f64();
        let world = self.app.world();
        let friction = world.resource::<DestinySpaceFriction>().0;
        if !friction.is_finite() || friction < 0.0 {
            return Err(CompatError::InvalidRequest(
                "space friction is outside the authoritative solver range".into(),
            ));
        }
        for (ball_id, entity) in &self.balls {
            let snapshot = self.ball_snapshot(*ball_id)?;
            validate_ball_snapshot(&snapshot, self.park.limits.max_child_shapes_per_ball)?;
            let metadata = world
                .get::<DestinyBallMetadata>(*entity)
                .ok_or_else(|| CompatError::Engine("missing DestinyBallMetadata".into()))?;
            let component_id = world
                .get::<DestinyBallId>(*entity)
                .ok_or_else(|| CompatError::Engine("missing DestinyBallId".into()))?
                .0;
            let rigid_body = world
                .get::<RigidBody>(*entity)
                .ok_or_else(|| CompatError::Engine("missing RigidBody".into()))?;
            let rigid_body_matches = (metadata.is_free && matches!(rigid_body, RigidBody::Dynamic))
                || (!metadata.is_free && matches!(rigid_body, RigidBody::Static));
            let actual_avian_mass = world
                .get::<Mass>(*entity)
                .ok_or_else(|| CompatError::Engine("missing Mass".into()))?
                .0;
            let collider_disabled = world.get::<ColliderDisabled>(*entity).is_some();
            let gravity_scale = world
                .get::<GravityScale>(*entity)
                .ok_or_else(|| CompatError::Engine("missing GravityScale".into()))?
                .0;
            if world.get::<Collider>(*entity).is_none() {
                return Err(CompatError::Engine("missing Collider".into()));
            }
            let pending_component = world.get::<DestinyPendingRemoval>(*entity);
            let pending_matches = match (self.park.pending_removals.get(ball_id), pending_component)
            {
                (None, None) => true,
                (Some(due), Some(component)) => {
                    component.due_tick == *due && component.reason == "delayed"
                }
                _ => false,
            };
            if component_id != *ball_id
                || !rigid_body_matches
                || actual_avian_mass != avian_mass(snapshot.mass)
                || collider_disabled == metadata.is_massive
                || gravity_scale != 0.0
                || !pending_matches
            {
                return Err(CompatError::InvalidRequest(format!(
                    "ball {ball_id} has inconsistent authoritative components"
                )));
            }
            let position = world
                .get::<Position>(*entity)
                .ok_or_else(|| CompatError::Engine("missing Position".into()))?
                .0;
            let velocity = world
                .get::<LinearVelocity>(*entity)
                .ok_or_else(|| CompatError::Engine("missing LinearVelocity".into()))?
                .0;
            let angular = world
                .get::<AngularVelocity>(*entity)
                .ok_or_else(|| CompatError::Engine("missing AngularVelocity".into()))?
                .0;
            let rotation = world
                .get::<Rotation>(*entity)
                .ok_or_else(|| CompatError::Engine("missing Rotation".into()))?
                .0;
            if !solver_safe_vec3(position, MAX_COORDINATE)
                || !solver_safe_vec3(velocity, MAX_VELOCITY)
                || !solver_safe_vec3(angular, MAX_VELOCITY)
                || !rotation.is_finite()
                || (rotation.length_squared() - 1.0).abs() > 1.0e-9
            {
                return Err(CompatError::InvalidRequest(format!(
                    "ball {ball_id} exceeds authoritative solver bounds"
                )));
            }
            if preflight {
                let pending = self.park.pending_removals.contains_key(ball_id);
                let mut planned_velocity = velocity;
                let max_speed = world
                    .get::<MaxLinearSpeed>(*entity)
                    .ok_or_else(|| CompatError::Engine("missing MaxLinearSpeed".into()))?
                    .0;
                let speed = stable_vec3_length(planned_velocity);
                if speed > max_speed && speed > 0.0 {
                    planned_velocity = stable_vec3_normalize(planned_velocity)
                        .map_or(DVec3::ZERO, |direction| direction * max_speed);
                }
                let integration = if pending || !metadata.is_free {
                    0.0
                } else {
                    let mass = world
                        .get::<DestinyMass>(*entity)
                        .ok_or_else(|| CompatError::Engine("missing DestinyMass".into()))?
                        .0;
                    let exponent = if mass <= 0.0 {
                        f64::INFINITY
                    } else {
                        (friction / mass / metadata.agility) * dt
                    };
                    if friction <= 0.0 || exponent == 0.0 {
                        dt
                    } else if exponent.is_finite() {
                        -(-exponent).exp_m1() / exponent * dt
                    } else {
                        0.0
                    }
                };
                let projected = position + planned_velocity * integration;
                if !solver_safe_vec3(projected, MAX_COORDINATE) {
                    return Err(CompatError::InvalidRequest(format!(
                        "ball {ball_id} would leave the solver-safe coordinate envelope"
                    )));
                }
            }
        }
        Ok(())
    }

    fn sync_visual_transforms(&mut self) {
        let entities = self.balls.values().copied().collect::<Vec<_>>();
        let world = self.app.world_mut();
        for entity in entities {
            let Some(position) = world.get::<Position>(entity).map(|value| value.0) else {
                continue;
            };
            let rotation = world
                .get::<Rotation>(entity)
                .map(|value| value.0)
                .unwrap_or(DQuat::IDENTITY);
            if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                transform.translation = visual_vec3(position);
                transform.rotation = rotation.as_quat();
            }
        }
    }

    pub(crate) fn begin_notification_pass(&self) -> NotificationPass {
        let mut owners = self
            .balls
            .iter()
            .map(|(&id, &entity)| (id, entity))
            .collect::<Vec<_>>();
        owners.sort_unstable_by_key(|&(id, _)| id);
        NotificationPass {
            owners: owners.into(),
            owner: None,
            stage: 0,
            sensors: VecDeque::new(),
            sensor: None,
            work: 0,
        }
    }

    fn sensor_index(&self, owner: i64, cloak: bool) -> Option<usize> {
        self.metadata(owner)
            .ok()?
            .sensors
            .iter()
            .position(|sensor| {
                sensor
                    .get("cloak_sensor")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    == cloak
            })
    }

    fn begin_sensor_pass(
        &mut self,
        owner: i64,
        cloak: bool,
        work: &mut usize,
    ) -> Result<Option<SensorPass>, CompatError> {
        let Some(index) = self.sensor_index(owner, cloak) else {
            return Ok(None);
        };
        if self.metadata(owner)?.sensors[index]
            .get("active")
            .and_then(Value::as_bool)
            == Some(false)
        {
            return Ok(None);
        }
        let dt = self
            .app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f64();
        let sensor = &self.metadata(owner)?.sensors[index];
        let prior = sensor["elapsed"].as_f64().unwrap_or(0.0);
        let period = sensor["period"].as_f64().unwrap_or(2.0);
        let (elapsed, due) =
            crate::proximity::sensor_elapsed_step(prior, dt, period).ok_or_else(|| {
                CompatError::InvalidRequest("invalid sensor clock or timestep".into())
            })?;
        self.metadata_mut(owner)?.sensors[index]["elapsed"] = json!(elapsed);
        if !due {
            return Ok(None);
        }
        let metadata = self.metadata(owner)?;
        if metadata.new_bubble_id < 0 {
            return Ok(None);
        }
        let only = metadata.sensors[index]["only_interactives"]
            .as_bool()
            .unwrap_or(false);
        let mut candidates = Vec::new();
        for (&id, &entity) in &self.balls {
            let other = self.metadata(id)?;
            if crate::proximity::candidate_admitted(
                metadata.is_cloaked != 0,
                other.is_free,
                other.is_interactive,
                only,
                self.park.is_master,
                metadata.new_bubble_id == other.new_bubble_id,
                other.is_global,
            ) {
                candidates.push((id, entity));
            }
        }
        *work = work
            .checked_add(candidates.len())
            .ok_or_else(|| CompatError::InvalidRequest("proximity work overflow".into()))?;
        if *work > MAX_PROXIMITY_WORK_PER_TICK {
            return Err(CompatError::InvalidRequest(
                "proximity work exceeds the runtime limit; no completed tick is claimed".into(),
            ));
        }
        candidates.sort_unstable_by_key(|&(id, _)| id);
        let nearby = candidates.iter().map(|&(id, _)| id).collect::<HashSet<_>>();
        let mut vanished = metadata.sensors[index]["members"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_i64)
            .filter(|id| *id != owner && !nearby.contains(id))
            .collect::<Vec<_>>();
        vanished.sort_unstable();
        Ok(Some(SensorPass {
            cloak,
            candidates: candidates.into(),
            vanished: vanished.into(),
        }))
    }

    fn set_sensor_member(
        &mut self,
        owner: i64,
        index: usize,
        other: i64,
        inside: bool,
    ) -> Result<bool, CompatError> {
        let mut metadata = self.metadata_mut(owner)?;
        let sensor = metadata.sensors.get_mut(index).ok_or_else(|| {
            CompatError::Engine("sensor disappeared during owned transition".into())
        })?;
        let mut members = sensor["members"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_i64)
            .collect::<Vec<_>>();
        let prior = members.contains(&other);
        if prior == inside {
            return Ok(false);
        }
        if inside {
            members.push(other);
            members.sort_unstable();
        } else {
            members.retain(|&id| id != other);
        }
        sensor["members"] = json!(members);
        Ok(true)
    }

    fn sensor_event(&mut self, owner: i64, index: usize, other: i64, entering: bool) -> Value {
        let event = json!({"kind":"transition","owner_id":owner,"other_id":other,
            "sensor_index":index,"entering":entering,"tick":self.park.current_time.saturating_add(1)});
        // The diagnostic queue is bounded independently. Backpressure does not
        // cancel a synchronous original callback or roll back its membership.
        let limit = self.park.limits.max_outbox_messages;
        let mut outbox = self.app.world_mut().resource_mut::<ProximityEventOutbox>();
        if outbox.0.len() < limit {
            outbox.0.push(event.clone());
        } else if limit > 0 {
            let last = outbox.0.len() - 1;
            outbox.0[last] = json!({"kind":"overflow","tick":self.park.current_time.saturating_add(1),"reason":"event_backpressure"});
        }
        event
    }

    fn range_notification(
        &mut self,
        owner: i64,
        tracking: bool,
    ) -> Result<Option<Value>, CompatError> {
        let Some(state) = self.range_notifications.get(&owner).cloned() else {
            return Ok(None);
        };
        let position = self.position(owner)?;
        let radius = self.metadata(owner)?.radius;
        let (prior, inside, enabled, kind) = if tracking {
            let Some(target) = state.target.filter(|id| self.balls.contains_key(id)) else {
                return Ok(None);
            };
            if state.tracking_range < 0.00001 {
                return Ok(None);
            }
            let other = self.position(target)?;
            let delta = OriginalVec3 {
                x: position.x - other.x,
                y: position.y - other.y,
                z: position.z - other.z,
            };
            let inside = crate::proximity::strict_range_contains(
                delta.norm_squared(),
                state.tracking_range + radius,
            )
            .ok_or_else(|| CompatError::InvalidRequest("invalid target tracking state".into()))?;
            (state.tracking_within, inside, true, 13)
        } else {
            if state.range == 0.0 {
                return Ok(None);
            }
            let inside = match self.motion.get(&owner) {
                Some(control) if control.mode == OriginalMode::Goto => {
                    let delta = OriginalVec3 {
                        x: position.x,
                        y: position.y,
                        z: position.z,
                    }
                    .sub(control.destination);
                    crate::proximity::strict_range_contains(
                        delta.norm_squared(),
                        state.range + radius,
                    )
                    .ok_or_else(|| {
                        CompatError::InvalidRequest("invalid notification range state".into())
                    })?
                }
                Some(control)
                    if matches!(
                        control.mode,
                        OriginalMode::Follow | OriginalMode::Orbit | OriginalMode::Missile
                    ) =>
                {
                    if let Some(target) = control.follow_id.filter(|id| self.balls.contains_key(id))
                    {
                        let other = self.position(target)?;
                        let delta = OriginalVec3 {
                            x: position.x - other.x,
                            y: position.y - other.y,
                            z: position.z - other.z,
                        };
                        let hi = self.metadata(target)?.radius
                            + f64::from(control.follow_range)
                            + state.range
                            + radius;
                        crate::proximity::annular_range_contains(
                            delta.norm_squared(),
                            hi,
                            state.range,
                        )
                        .ok_or_else(|| {
                            CompatError::InvalidRequest(
                                "invalid follow notification range state".into(),
                            )
                        })?
                    } else {
                        false
                    }
                }
                _ => false,
            };
            (state.within, inside, true, 7)
        };
        let (within, send) = crate::proximity::notification_transition(prior, inside, enabled);
        let current = self
            .range_notifications
            .get_mut(&owner)
            .ok_or_else(|| CompatError::Engine("notification state lost under ownership".into()))?;
        if tracking {
            current.tracking_within = within;
        } else {
            current.within = within;
        }
        Ok(send.then(||json!({"kind":"range","owner_id":owner,"event_kind":kind,"arguments":[if inside {1}else{0}]})))
    }

    pub(crate) fn next_tick_notification(
        &mut self,
        pass: &mut NotificationPass,
    ) -> Result<Option<Value>, CompatError> {
        if self.pending_tick_time.is_none() {
            return Ok(None);
        }
        loop {
            if pass.owner.is_none() {
                pass.owner = crate::notification_batch::take_next(&mut pass.owners);
                pass.stage = 0;
                pass.sensor = None;
                pass.sensors.clear();
            }
            let Some((owner, entity)) = pass.owner else {
                return Ok(None);
            };
            if self.balls.get(&owner) != Some(&entity)
                || self.park.pending_removals.contains_key(&owner)
            {
                pass.owner = None;
                continue;
            }
            match pass.stage {
                0 => {
                    pass.stage = 1;
                    if let Some(event) = self.range_notification(owner, false)? {
                        return Ok(Some(event));
                    }
                }
                1 => {
                    pass.stage = 2;
                    if let Some(event) = self.range_notification(owner, true)? {
                        return Ok(Some(event));
                    }
                }
                2 => {
                    pass.stage = 3;
                    pass.sensors = self
                        .metadata(owner)?
                        .sensors
                        .iter()
                        .map(|sensor| {
                            sensor
                                .get("cloak_sensor")
                                .and_then(Value::as_bool)
                                .unwrap_or(false)
                        })
                        .collect();
                }
                3 => {
                    if pass.sensor.is_none() {
                        let Some(cloak) = crate::notification_batch::take_next(&mut pass.sensors)
                        else {
                            pass.owner = None;
                            continue;
                        };
                        pass.sensor = self.begin_sensor_pass(owner, cloak, &mut pass.work)?;
                        if pass.sensor.is_none() {
                            continue;
                        }
                    }
                    let sensor = pass
                        .sensor
                        .as_mut()
                        .ok_or_else(|| CompatError::Engine("sensor continuation lost".into()))?;
                    // Removal cancels remaining work. Replacement reuses the
                    // same logical sensor, as Ball::AddProximitySensor does.
                    let Some(index) = self.sensor_index(owner, sensor.cloak) else {
                        pass.sensor = None;
                        continue;
                    };
                    if let Some(other) = crate::notification_batch::take_next(&mut sensor.vanished)
                    {
                        if self.set_sensor_member(owner, index, other, false)? {
                            return Ok(Some(self.sensor_event(owner, index, other, false)));
                        }
                        continue;
                    }
                    let Some((other, other_entity)) =
                        crate::notification_batch::take_next(&mut sensor.candidates)
                    else {
                        pass.sensor = None;
                        continue;
                    };
                    if other == owner
                        || self.balls.get(&other) != Some(&other_entity)
                        || self.park.pending_removals.contains_key(&other)
                    {
                        continue;
                    }
                    // Keep the candidate list fixed, but read positions, radii,
                    // sensor range and membership after every prior callback.
                    let owner_position = self.position(owner)?;
                    let other_position = self.position(other)?;
                    let metadata = self.metadata(owner)?;
                    let reach = metadata.sensors[index]["range"].as_f64().unwrap_or(0.0)
                        + self.metadata(other)?.radius
                        + metadata.radius;
                    let delta = OriginalVec3 {
                        x: owner_position.x - other_position.x,
                        y: owner_position.y - other_position.y,
                        z: owner_position.z - other_position.z,
                    };
                    let inside =
                        crate::proximity::inclusive_range_contains(delta.norm_squared(), reach)
                            .ok_or_else(|| {
                                CompatError::InvalidRequest("invalid sensor geometry state".into())
                            })?;
                    if self.set_sensor_member(owner, index, other, inside)? {
                        return Ok(Some(self.sensor_event(owner, index, other, inside)));
                    }
                }
                _ => {
                    return Err(CompatError::Engine(
                        "invalid notification continuation stage".into(),
                    ));
                }
            }
        }
    }

    fn bubble_membership_value(&self) -> Result<Value, CompatError> {
        let mut eligible = Vec::new();
        let mut ids = self.balls.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        for ball_id in ids {
            if self.park.pending_removals.contains_key(&ball_id) {
                continue;
            }
            let metadata = self.metadata(ball_id)?;
            if metadata.new_bubble_id < 0 || metadata.is_cloaked != 0 {
                continue;
            }
            eligible.push((
                ball_id,
                metadata.new_bubble_id,
                metadata.is_interactive,
                metadata.is_global,
            ));
        }

        let mut interactives: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut bubbles = HashSet::new();
        let mut global_ids = Vec::new();
        for (ball_id, bubble_id, is_interactive, is_global) in &eligible {
            if *is_interactive {
                interactives.entry(*bubble_id).or_default().push(*ball_id);
                bubbles.insert(*bubble_id);
            }
            if *is_global {
                global_ids.push(*ball_id);
            } else {
                bubbles.insert(*bubble_id);
            }
        }

        let mut bubble_ids = bubbles.into_iter().collect::<Vec<_>>();
        bubble_ids.sort_unstable();
        let mut members: HashMap<i64, Vec<i64>> = HashMap::new();
        for bubble_id in bubble_ids {
            let mut row = global_ids.clone();
            row.extend(
                eligible
                    .iter()
                    .filter_map(|(ball_id, candidate_bubble, _, is_global)| {
                        (!*is_global && *candidate_bubble == bubble_id).then_some(*ball_id)
                    }),
            );
            row.sort_unstable();
            row.dedup();
            members.insert(bubble_id, row);
        }

        let mut interactive_rows = Map::new();
        let mut interactive_bubbles = interactives.keys().copied().collect::<Vec<_>>();
        interactive_bubbles.sort_unstable();
        for bubble_id in interactive_bubbles {
            let mut row = interactives.remove(&bubble_id).unwrap_or_default();
            row.sort_unstable();
            interactive_rows.insert(bubble_id.to_string(), json!(row));
        }
        let mut member_rows = Map::new();
        let mut member_bubbles = members.keys().copied().collect::<Vec<_>>();
        member_bubbles.sort_unstable();
        for bubble_id in member_bubbles {
            member_rows.insert(
                bubble_id.to_string(),
                json!(members.get(&bubble_id).cloned().unwrap_or_default()),
            );
        }
        let mut observer_rows = Map::new();
        for (ball_id, bubble_id, is_interactive, _) in eligible {
            if is_interactive {
                observer_rows.insert(
                    ball_id.to_string(),
                    json!(members.get(&bubble_id).cloned().unwrap_or_default()),
                );
            }
        }
        Ok(json!({
            "interactives": interactive_rows,
            "members": member_rows,
            "observers": observer_rows,
        }))
    }

    fn dispatch_compat(
        &mut self,
        title: &str,
        operation: &str,
        args: &[Value],
    ) -> Result<Value, CompatError> {
        if operation != "call" {
            return Err(CompatError::InvalidRequest(format!(
                "{title} requires operation=call, got {operation:?}",
            )));
        }
        match title {
            "dbc.compat.Runtime.Configure" => {
                require_arity(args, &[3, 5], title)?;
                let iterations = value_i64(args.first(), "collisionMaxIterations")?;
                let iterative = value_bool(args.get(1), "useIterativeCollision")?;
                let new_orbit = value_bool(args.get(2), "useNewOrbit")?;
                let dynamical = args
                    .get(3)
                    .map_or(Ok(self.park.use_dynamical_orientation), |value| {
                        value_bool(Some(value), "useDynamicalOrientation")
                    })?;
                let disable_missiles = args.get(4).map_or(
                    Ok(self.park.disable_dynamical_orientation_for_missiles),
                    |value| value_bool(Some(value), "disableDynamicalOrientationForMissiles"),
                )?;
                if !(1..=i64::from(MAX_COLLISION_SUBSTEPS)).contains(&iterations) {
                    return Err(CompatError::InvalidRequest(
                        "invalid collisionMaxIterations".into(),
                    ));
                }
                let mut count = self
                    .app
                    .world_mut()
                    .get_resource_mut::<SubstepCount>()
                    .ok_or_else(|| CompatError::Engine("missing SubstepCount".into()))?;
                count.0 = if iterative { iterations as u32 } else { 1 };
                self.park.collision_substeps = iterations as u32;
                self.park.use_iterative_collision = iterative;
                self.park.use_new_orbit = new_orbit;
                self.park.use_dynamical_orientation = dynamical;
                self.park.disable_dynamical_orientation_for_missiles = disable_missiles;
                Ok(Value::Null)
            }
            "dbc.compat.Ballpark.HasBall" => {
                require_arity(args, &[1], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                Ok(json!(self.balls.contains_key(&id)))
            }
            "dbc.compat.Ballpark.ObjectGeneration" => {
                require_arity(args, &[1], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                let generation = self
                    .ball_generations
                    .get(&id)
                    .ok_or(CompatError::BallNotFound(id))?;
                Ok(json!(generation))
            }
            "dbc.compat.Ballpark.NewDetachedBall" => {
                require_arity(args, &[1], title)?;
                let client = value_bool(args.first(), "client")?;
                let fields = crate::ball_defaults::detached_fields(client);
                let generation = self.next_ball_generation;
                let next = generation.checked_add(1).ok_or_else(|| {
                    CompatError::InvalidRequest("object generation exhausted".into())
                })?;
                let snapshot = BallSnapshot {
                    id: fields.id,
                    mass: fields.mass,
                    radius: f64::from(fields.radius),
                    max_velocity: f64::from(fields.max_velocity),
                    is_free: fields.is_free,
                    is_global: fields.is_global,
                    is_massive: fields.is_massive,
                    is_interactive: fields.is_interactive,
                    is_space_junk: fields.is_space_junk,
                    position: vec![fields.position.x, fields.position.y, fields.position.z],
                    velocity: vec![fields.velocity.x, fields.velocity.y, fields.velocity.z],
                    agility: f64::from(fields.agility),
                    speed_fraction: f64::from(fields.speed_fraction),
                    max_angular_velocity: f64::from(fields.max_angular_velocity),
                    angular_agility: f64::from(fields.rotational_agility),
                    angular_velocity: fields
                        .angular_velocity
                        .iter()
                        .map(|value| f64::from(*value))
                        .collect(),
                    rotation: fields
                        .rotation
                        .iter()
                        .map(|value| f64::from(*value))
                        .collect(),
                    is_cloaked: i32::from(fields.is_cloaked),
                    new_bubble_id: fields.new_bubble,
                    old_bubble_id: fields.old_bubble,
                    effect_stamp: fields.effect_stamp,
                    formation_id: NO_FORMATION,
                    formation_slots: fields.formation_slots,
                    massive_before_cloak: None,
                    minis: Vec::new(),
                    mini_ids: Vec::new(),
                    owner_id: fields.owner_id,
                    sensors: Vec::new(),
                    motion: Some(MotionSnapshot {
                        mode: mode_number(fields.mode),
                        destination: [fields.goto.x, fields.goto.y, fields.goto.z],
                        min_range: 0.0,
                        warp_factor: 20,
                        warp_distance: fields.last_collision,
                        follow_id: None,
                        follow_range: fields.follow_range,
                    }),
                    notifications: Some(RangeNotificationSnapshot {
                        range: fields.notification_range,
                        within: fields.within_range,
                        target: fields.tracking_id,
                        tracking_range: fields.tracking_range,
                        tracking_within: fields.within_tracking_range,
                    }),
                };
                self.retained_balls.insert(
                    generation,
                    RetainedBall {
                        id: 0,
                        attached: false,
                        moribund: false,
                        references: 1,
                        detached: Some(snapshot),
                        client,
                        identity_fields: [
                            fields.harmonic,
                            i64::from(fields.corporation_id),
                            i64::from(fields.alliance_id),
                        ],
                    },
                );
                self.next_ball_generation = next;
                Ok(json!(generation))
            }
            "dbc.compat.Ballpark.RetainBall" | "dbc.compat.Ballpark.ReleaseBall" => {
                require_arity(args, &[2], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                let generation = args[1]
                    .as_u64()
                    .ok_or_else(|| CompatError::InvalidRequest("generation must fit u64".into()))?;
                let event = if title.ends_with("RetainBall") { 0 } else { 1 };
                self.transition_retained_ball(id, generation, event)?;
                Ok(Value::Null)
            }
            "dbc.compat.Ballpark.ListBalls" => {
                require_arity(args, &[0], title)?;
                let mut ids: Vec<_> = self.balls.keys().copied().collect();
                ids.sort_unstable();
                Ok(Value::Array(
                    ids.into_iter()
                        .map(|id| json!(format!("ball:{id}")))
                        .collect(),
                ))
            }
            "dbc.compat.Ballpark.GetBall" => {
                require_arity(args, &[1], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                self.entity(id)?;
                Ok(json!(format!("ball:{id}")))
            }
            "dbc.compat.Ballpark.BubbleMembership" => {
                require_arity(args, &[0], title)?;
                self.bubble_membership_value()
            }
            "dbc.compat.Ballpark.CaptureSnapshot" => {
                require_arity(args, &[0, 1], title)?;
                let source_id = match args.first() {
                    Some(Value::Null) | None => None,
                    Some(value) if value.as_i64() == Some(-1) => None,
                    Some(value) => Some(value_i64(Some(value), "source_id")?),
                };
                let current_time = self.park.current_time;
                let snapshot = self.serialize_snapshot(None, source_id)?;
                Ok(json!({
                    "current_time": current_time,
                    "snapshot": snapshot,
                }))
            }
            "dbc.compat.Ballpark.Serialize" => {
                require_arity(args, &[1, 2], title)?;
                let ids = match args.first() {
                    Some(Value::Array(values)) => Some(
                        values
                            .iter()
                            .map(|value| value_i64(Some(value), "ball_id"))
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    Some(Value::Null) | None => None,
                    _ => {
                        return Err(CompatError::InvalidRequest(
                            "Serialize expects an ID array or null".into(),
                        ));
                    }
                };
                let source_id = match args.get(1) {
                    Some(Value::Null) | None => None,
                    Some(value) if value.as_i64() == Some(-1) => None,
                    Some(value) => Some(value_i64(Some(value), "source_id")?),
                };
                let encoded = self.serialize_snapshot(ids.as_deref(), source_id)?;
                Ok(json!(encoded))
            }
            "dbc.compat.Ballpark.Deserialize" => {
                require_arity(args, &[1, 2], title)?;
                let encoded = args.first().and_then(Value::as_str).ok_or_else(|| {
                    CompatError::InvalidRequest("Deserialize expects base64 data".into())
                })?;
                let partial = args
                    .get(1)
                    .map(|value| value_i64(Some(value), "partial"))
                    .transpose()?
                    .unwrap_or(0);
                if !matches!(partial, 0..=2) {
                    return Err(CompatError::InvalidRequest(
                        "partial must be 0, 1, or 2".into(),
                    ));
                }
                self.deserialize_snapshot(encoded, partial)?;
                Ok(Value::Null)
            }
            "dbc.compat.Network.DrainOutbox" => {
                require_arity(args, &[0], title)?;
                let mut outbox = self.app.world_mut().resource_mut::<CarbonNetworkOutbox>();
                let last_batch_id = outbox.last_batch_id;
                let last_batch_envelope = outbox.last_batch_envelope.take();
                let drained = std::mem::take(&mut *outbox);
                outbox.last_batch_id = last_batch_id;
                outbox.last_batch_envelope = last_batch_envelope;
                Ok(serde_json::to_value(drained)
                    .map_err(|error| CompatError::Engine(error.to_string()))?)
            }
            "dbc.compat.Proximity.DrainEvents" => {
                require_arity(args, &[0], title)?;
                let mut events = self.app.world_mut().resource_mut::<ProximityEventOutbox>();
                Ok(Value::Array(std::mem::take(&mut events.0)))
            }
            _ => Err(CompatError::UnsupportedTitle(title.to_owned())),
        }
    }

    fn entity(&self, ball_id: i64) -> Result<Entity, CompatError> {
        self.balls
            .get(&ball_id)
            .copied()
            .ok_or(CompatError::BallNotFound(ball_id))
    }

    fn transition_retained_ball(
        &mut self,
        id: i64,
        generation: u64,
        event: u8,
    ) -> Result<(), CompatError> {
        let ball = self
            .retained_balls
            .get_mut(&generation)
            .filter(|ball| ball.id == id)
            .ok_or(CompatError::BallNotFound(id))?;
        let (attached, moribund, references) =
            crate::ball_lifecycle::transition(ball.attached, ball.moribund, ball.references, event)
                .ok_or_else(|| {
                    CompatError::InvalidRequest("invalid retained ball lifecycle transition".into())
                })?;
        ball.attached = attached;
        ball.moribund = moribund;
        ball.references = references;
        if !attached && references == 0 {
            self.retained_balls.remove(&generation);
        }
        Ok(())
    }

    fn detach_retained_ball(&mut self, id: i64, removed: bool) -> Result<(), CompatError> {
        let generation = *self
            .ball_generations
            .get(&id)
            .ok_or(CompatError::BallNotFound(id))?;
        if self
            .retained_balls
            .get(&generation)
            .is_some_and(|ball| ball.references > 0)
        {
            let mut snapshot = self.ball_snapshot(id)?;
            if removed {
                if let Some(motion) = &mut snapshot.motion {
                    motion.mode = mode_number(OriginalMode::Stop);
                }
                snapshot.old_bubble_id = snapshot.new_bubble_id;
                snapshot.new_bubble_id = -1;
                if !snapshot.is_free {
                    snapshot.minis.clear();
                    snapshot.mini_ids.clear();
                }
            }
            self.retained_balls
                .get_mut(&generation)
                .expect("live generation")
                .detached = Some(snapshot);
        }
        self.transition_retained_ball(id, generation, if removed { 2 } else { 3 })?;
        self.ball_generations.remove(&id);
        Ok(())
    }

    fn clear_all(&mut self) -> Result<(), CompatError> {
        // Capture every retained object's state before removing components.
        let snapshots = self
            .ball_generations
            .iter()
            .filter_map(|(id, generation)| {
                self.retained_balls
                    .get(generation)
                    .filter(|ball| ball.references > 0)
                    .map(|_| {
                        self.ball_snapshot(*id)
                            .map(|snapshot| (*generation, snapshot))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (generation, snapshot) in snapshots {
            self.retained_balls
                .get_mut(&generation)
                .expect("live generation")
                .detached = Some(snapshot);
        }
        let identities = self
            .ball_generations
            .iter()
            .map(|(id, generation)| (*id, *generation))
            .collect::<Vec<_>>();
        for (id, generation) in identities {
            self.transition_retained_ball(id, generation, 3)?;
        }
        self.ball_generations.clear();
        self.original_bubbles = None;
        let balls: Vec<_> = self.balls.drain().map(|(_, entity)| entity).collect();
        self.park.pending_removals.clear();
        self.park.ego = 0;
        self.park.local_counter = default_local_counter();
        self.park.local_high_counter = default_local_high_counter();
        self.motion.clear();
        self.follower_sets.clear();
        self.range_notifications.clear();
        let world = self.app.world_mut();
        for entity in balls {
            let _ = world.despawn(entity);
        }
        world.resource_mut::<ProximityEventOutbox>().0.clear();
        *world.resource_mut::<CarbonNetworkOutbox>() = CarbonNetworkOutbox::default();
        Ok(())
    }

    fn remove_ball(&mut self, ball_id: i64) -> Result<(), CompatError> {
        self.remove_ball_internal(ball_id, true)
    }

    fn remove_ball_internal(
        &mut self,
        ball_id: i64,
        scrub_related_state: bool,
    ) -> Result<(), CompatError> {
        let children = self.metadata(ball_id)?.mini_ids.clone();
        for child in children {
            if self.balls.contains_key(&child)
                && self.metadata(child)?.owner_id == ball_id
                && self
                    .motion
                    .get(&child)
                    .is_some_and(|control| control.mode == OriginalMode::Miniball)
            {
                self.remove_ball_internal(child, scrub_related_state)?;
            }
        }
        let entity = self
            .balls
            .get(&ball_id)
            .copied()
            .ok_or(CompatError::BallNotFound(ball_id))?;
        if scrub_related_state {
            self.remove_from_sensor_members(ball_id)?;
        }
        let bubble_ball = self.source_bubble_ball(ball_id)?;
        if let Some(tracking) = &mut self.original_bubbles {
            tracking.transition(crate::bubbles::Ball {
                old: bubble_ball.new,
                new: -1,
                ..bubble_ball
            });
        }
        self.follower_sets.remove(&self.ball_generations[&ball_id]);
        self.detach_retained_ball(ball_id, true)?;
        self.balls.remove(&ball_id);
        self.motion.remove(&ball_id);
        self.range_notifications.remove(&ball_id);
        self.park.pending_removals.remove(&ball_id);
        if scrub_related_state && self.park.ego == ball_id {
            self.park.ego = 0;
        }
        let world = self.app.world_mut();
        let _ = world.despawn(entity);
        if scrub_related_state {
            world
                .resource_mut::<ProximityEventOutbox>()
                .0
                .retain(|event| {
                    event.get("owner_id").and_then(Value::as_i64) != Some(ball_id)
                        && event.get("other_id").and_then(Value::as_i64) != Some(ball_id)
                });
        }
        Ok(())
    }

    fn schedule_remove_ball(&mut self, ball_id: i64, delay: i64) -> Result<(), CompatError> {
        if delay < 0 {
            return Err(CompatError::InvalidRequest(
                "removal delay must be non-negative".into(),
            ));
        }
        if !self.balls.contains_key(&ball_id) {
            return Ok(());
        }
        self.park
            .current_time
            .checked_add(delay)
            .ok_or_else(|| CompatError::InvalidRequest("removal delay overflows time".into()))?;
        let entity = self.entity(ball_id)?;
        let world = self.app.world();
        if world.get::<LinearVelocity>(entity).is_none()
            || world.get::<AngularVelocity>(entity).is_none()
        {
            return Err(CompatError::Engine(
                "ball is missing LinearVelocity or AngularVelocity".into(),
            ));
        }
        self.stop_command(ball_id)?;
        self.mode_callbacks.push_back(CallCallback::Remove {
            ball_id,
            generation: self.ball_generations[&ball_id],
            delay,
            step: RemovalStep::Followers,
            arrival: None,
        });
        Ok(())
    }

    fn resume_removal(
        &mut self,
        ball_id: i64,
        generation: u64,
        delay: i64,
        step: RemovalStep,
        arrival: Option<OriginalVec3>,
    ) -> Result<(), CompatError> {
        // Ballpark::RemoveBall resumes only after each synchronous mode event.
        // The source copies an unordered follower-ID set, then resolves each
        // live follower again. Reentrant changes are therefore observed here.
        let next = match step {
            RemovalStep::Followers => RemovalStep::NextFollower(self.followers(ball_id).into()),
            RemovalStep::NextFollower(mut followers) => {
                if let Some(follower) = followers.pop_front() {
                    let live = self.balls.contains_key(&follower);
                    let follows = self
                        .motion
                        .get(&follower)
                        .is_some_and(|control| control.follow_id == Some(ball_id));
                    let mode = self
                        .motion
                        .get(&follower)
                        .map_or(OriginalMode::Stop, |control| control.mode);
                    let interactive = live && self.metadata(follower)?.is_interactive;
                    let action = crate::ball_lifecycle::follower_action(
                        live,
                        follows,
                        mode_number(mode),
                        interactive,
                    );
                    if action != 0 {
                        if action >= 2 {
                            if action == 3 {
                                self.set_massive(follower, false)?;
                            }
                            let velocity = self
                                .app
                                .world()
                                .get::<LinearVelocity>(self.entity(follower)?)
                                .ok_or_else(|| {
                                    CompatError::Engine("missing follower velocity".into())
                                })?
                                .0;
                            self.begin_goto_direction(
                                follower,
                                OriginalVec3 {
                                    x: velocity.x,
                                    y: velocity.y,
                                    z: velocity.z,
                                },
                            )?;
                        } else {
                            self.stop_command(follower)?;
                        }
                    } else if let Some(followers) = self.follower_sets.get_mut(&generation) {
                        followers.remove(&follower);
                    }
                    RemovalStep::NextFollower(followers)
                } else {
                    RemovalStep::Mark
                }
            }
            RemovalStep::Mark => {
                if let Some(point) = arrival {
                    return self.finish_entity_warp(ball_id, point);
                }
                self.transition_retained_ball(ball_id, generation, 5)?;
                if delay == 0 && self.evolution_phase == 0 {
                    self.mode_callbacks.push_back(CallCallback::Ball {
                        ball_id,
                        generation,
                        kind: 15,
                        arguments: json!([]),
                    });
                    RemovalStep::Detach
                } else {
                    return self.mark_pending_removal(ball_id, delay);
                }
            }
            RemovalStep::Detach => return self.remove_ball(ball_id),
        };
        self.mode_callbacks.push_back(CallCallback::Remove {
            ball_id,
            generation,
            delay,
            step: next,
            arrival,
        });
        Ok(())
    }

    fn mark_pending_removal(&mut self, ball_id: i64, delay: i64) -> Result<(), CompatError> {
        let due =
            self.park.current_time.checked_add(delay).ok_or_else(|| {
                CompatError::InvalidRequest("removal delay overflows time".into())
            })?;
        let entity = self.entity(ball_id)?;
        let mut metadata = self.metadata(ball_id)?.clone();
        metadata.effect_stamp = due;
        metadata.is_massive = false;
        self.app.world_mut().entity_mut(entity).insert((
            metadata,
            ColliderDisabled,
            DestinyPendingRemoval {
                due_tick: due,
                reason: "delayed".into(),
            },
        ));
        self.park.pending_removals.insert(ball_id, due);
        let generation = self.ball_generations[&ball_id];
        self.transition_retained_ball(ball_id, generation, 5)?;
        Ok(())
    }

    fn remove_due_balls(&mut self) {
        let now = self.park.current_time;
        let due = self
            .park
            .pending_removals
            .iter()
            .filter_map(|(ball_id, due)| (*due <= now).then_some(*ball_id))
            .collect::<Vec<_>>();
        for ball_id in due {
            let _ = self.remove_ball(ball_id);
        }
    }

    fn add_ball(&mut self, args: &[Value]) -> Result<String, CompatError> {
        self.add_ball_with_identity(args, None)
    }

    fn add_ball_with_identity(
        &mut self,
        args: &[Value],
        preserved_id: Option<i64>,
    ) -> Result<String, CompatError> {
        if preserved_id.is_none() && !crate::evolution_phase::add_allowed(self.evolution_phase) {
            return Err(CompatError::InvalidRequest(
                "Can not add ball during Evolve().".into(),
            ));
        }
        let expected = if self.park.use_dynamical_orientation {
            19
        } else {
            17
        };
        if args.len() != expected {
            return Err(CompatError::InvalidRequest(format!(
                "AddBall expects {expected} arguments when useDynamicalOrientation={}, got {}",
                self.park.use_dynamical_orientation,
                args.len(),
            )));
        }
        let requested_id = value_i64(args.first(), "srcId")?;
        let allocation = preserved_id
            .map(|id| (id, self.park.local_counter, self.park.local_high_counter))
            .or_else(|| {
                crate::collision_identity::allocate_collision_id(
                    self.park.is_master,
                    requested_id,
                    self.park.local_counter,
                    self.park.local_high_counter,
                )
            })
            .ok_or_else(|| {
                CompatError::InvalidRequest("collision identity counter exhausted".into())
            })?;
        let id = allocation.0;
        let mass = value_f64(args.get(1), "mass")?.max(0.0);
        let radius = value_f64(args.get(2), "radius")?.max(0.0);
        let max_velocity = value_f64(args.get(3), "maxVel")?.max(0.0);
        let is_free = value_bool(args.get(4), "isFree")?;
        let is_global = value_bool(args.get(5), "isGlobal")?;
        let is_massive = value_bool(args.get(6), "isMassive")?;
        let is_interactive = value_bool(args.get(7), "isInteractive")?;
        let is_space_junk = value_bool(args.get(8), "isSpaceJunk")?;
        let position = DVec3::new(
            value_f64(args.get(9), "x")?,
            value_f64(args.get(10), "y")?,
            value_f64(args.get(11), "z")?,
        );
        let velocity = DVec3::new(
            value_f64(args.get(12), "vx")?,
            value_f64(args.get(13), "vy")?,
            value_f64(args.get(14), "vz")?,
        );
        let mut agility = value_f64(args.get(15), "agility")?;
        if agility <= 0.0 {
            agility = 1.0;
        }
        let speed_fraction = value_f64(args.get(16), "speedFraction")?.clamp(0.0, 1.0);
        let max_angular_speed = if args.len() == 19 {
            value_f64(args.get(17), "maxAngularSpeed")?.max(0.0)
        } else {
            MAX_VELOCITY
        };
        let angular_agility = if args.len() == 19 {
            value_f64(args.get(18), "angularAgility")?.max(0.0)
        } else {
            0.0
        };
        if mass > MAX_MASS
            || radius > MAX_RADIUS
            || max_velocity > MAX_VELOCITY
            || max_angular_speed > MAX_VELOCITY
            || agility > MAX_AGILITY
            || angular_agility > MAX_AGILITY
            || !solver_safe_vec3(position, MAX_COORDINATE)
            || !solver_safe_vec3(velocity, MAX_VELOCITY)
        {
            return Err(CompatError::InvalidRequest(
                "AddBall arguments exceed the solver-safe envelope".into(),
            ));
        }
        self.park.local_counter = allocation.1;
        self.park.local_high_counter = allocation.2;
        if self.balls.contains_key(&id) {
            let existing_cloak = self.metadata(id)?.is_cloaked;
            let mut staged_metadata = self.metadata(id)?.clone();
            staged_metadata.radius = radius;
            staged_metadata.is_free = is_free;
            staged_metadata.is_global = is_global;
            staged_metadata.is_massive = existing_cloak == 0 && is_massive;
            staged_metadata.is_interactive = is_interactive;
            staged_metadata.is_space_junk = is_space_junk;
            staged_metadata.agility = agility;
            staged_metadata.speed_fraction = speed_fraction;
            staged_metadata.angular_agility = angular_agility;
            staged_metadata.effect_stamp = 0;
            if existing_cloak != 0 {
                staged_metadata.massive_before_cloak = Some(is_massive);
            }
            validate_ball_snapshot(
                &self.ball_snapshot(id)?,
                self.park.limits.max_child_shapes_per_ball,
            )?;
            root_collider_for_metadata(&staged_metadata)?;
            let entity = self.entity(id)?;
            let world = self.app.world();
            if world.get::<Transform>(entity).is_none()
                || world.get::<Rotation>(entity).is_none()
                || world.get::<RigidBody>(entity).is_none()
            {
                return Err(CompatError::Engine(
                    "ball is missing Transform, Rotation, or RigidBody".into(),
                ));
            }
            if let Some(direction) = stable_vec3_normalize(velocity) {
                normalized_quat(DQuat::from_rotation_arc(DVec3::X, direction))?;
            }
            self.set_mass(id, mass)?;
            self.set_radius(id, radius)?;
            self.set_max_velocity(id, max_velocity)?;
            self.set_max_angular_velocity(id, max_angular_speed)?;
            self.set_position(id, position)?;
            self.set_velocity(id, velocity)?;
            self.set_free(id, is_free)?;
            self.set_massive(
                id,
                if existing_cloak != 0 {
                    false
                } else {
                    is_massive
                },
            )?;
            let mut metadata = self.metadata_mut(id)?;
            metadata.is_global = is_global;
            metadata.is_interactive = is_interactive;
            metadata.is_space_junk = is_space_junk;
            metadata.agility = agility;
            metadata.speed_fraction = speed_fraction;
            metadata.angular_agility = angular_agility;
            metadata.effect_stamp = 0;
            if existing_cloak != 0 {
                metadata.massive_before_cloak = Some(is_massive);
            }
            self.park.pending_removals.remove(&id);
            self.app
                .world_mut()
                .entity_mut(entity)
                .remove::<DestinyPendingRemoval>();
            self.transition_retained_ball(id, self.ball_generations[&id], 4)?;
            return Ok(format!("ball:{id}"));
        }

        if self.balls.len() >= self.park.limits.max_snapshot_balls {
            return Err(CompatError::InvalidRequest(
                "live ball capacity exceeded".into(),
            ));
        }
        let generation = self.next_ball_generation;
        let following_generation = generation.checked_add(1).ok_or_else(|| {
            CompatError::InvalidRequest("ball object generation exhausted".into())
        })?;

        let rigid_body = if is_free {
            RigidBody::Dynamic
        } else {
            RigidBody::Static
        };
        let initial_rotation = if !self.park.use_dynamical_orientation {
            stable_vec3_normalize(velocity)
                .map(|direction| DQuat::from_rotation_arc(DVec3::X, direction))
                .unwrap_or(DQuat::IDENTITY)
        } else {
            DQuat::IDENTITY
        };
        let metadata = DestinyBallMetadata {
            radius,
            is_free,
            is_global,
            is_massive,
            is_interactive,
            is_space_junk,
            agility,
            speed_fraction,
            angular_agility,
            is_cloaked: 0,
            new_bubble_id: -1,
            old_bubble_id: -1,
            effect_stamp: 0,
            formation_id: NO_FORMATION,
            formation_slots: 0,
            massive_before_cloak: None,
            minis: Vec::new(),
            mini_ids: Vec::new(),
            owner_id: 0,
            sensors: Vec::new(),
        };
        // Bevy's tuple Bundle implementations are arity-limited. Keep the
        // component set identical, but insert it in two batches so this code
        // does not depend on a particular maximum tuple arity.
        let entity = self
            .app
            .world_mut()
            .spawn((
                DestinyBallId(id),
                metadata,
                rigid_body,
                Collider::sphere(radius),
                Transform::from_translation(visual_vec3(position))
                    .with_rotation(initial_rotation.as_quat()),
                GlobalTransform::default(),
                Position(position),
                Rotation(initial_rotation),
                LinearVelocity(velocity),
                AngularVelocity::ZERO,
            ))
            .id();
        self.app.world_mut().entity_mut(entity).insert((
            LinearDamping(0.0),
            DestinyMass(mass),
            DestinyPresentationAngularVelocity::default(),
            Mass(avian_mass(mass)),
            MaxLinearSpeed(max_velocity),
            MaxAngularSpeed(max_angular_speed),
            GravityScale(0.0),
            CollidingEntities::default(),
            ActiveCollisionHooks::FILTER_PAIRS,
        ));

        if !is_massive {
            self.app
                .world_mut()
                .entity_mut(entity)
                .insert(ColliderDisabled);
        }

        #[cfg(feature = "carbon-network")]
        if self.park.network_components {
            use crate::network::DestinyCarbonReplicationBundle;
            self.app
                .world_mut()
                .entity_mut(entity)
                .insert(DestinyCarbonReplicationBundle::hidden());
        }

        self.park.pending_removals.remove(&id);
        self.balls.insert(id, entity);
        self.next_ball_generation = following_generation;
        self.ball_generations.insert(id, generation);
        self.retained_balls.insert(
            generation,
            RetainedBall {
                id,
                attached: true,
                moribund: false,
                references: 0,
                detached: None,
                client: !self.park.is_master,
                identity_fields: [-1; 3],
            },
        );
        Ok(format!("ball:{id}"))
    }

    fn serialize_snapshot(
        &self,
        selected: Option<&[i64]>,
        source_id: Option<i64>,
    ) -> Result<String, CompatError> {
        validate_formations(&self.park.formations)?;
        if selected.is_some_and(|ids| ids.len() > self.park.limits.max_snapshot_balls) {
            return Err(CompatError::InvalidRequest(
                "snapshot selector exceeds the ball-count limit".into(),
            ));
        }
        let mut ids: Vec<i64> =
            selected.map_or_else(|| self.balls.keys().copied().collect(), |ids| ids.to_vec());
        ids.sort_unstable();
        ids.dedup();
        if source_id.is_some_and(|id| self.park.pending_removals.contains_key(&id)) {
            return Err(CompatError::InvalidRequest(
                "snapshot source is pending removal".into(),
            ));
        }
        let source = source_id.map(|id| self.metadata(id)).transpose()?;
        let mut selected_ids = Vec::new();
        for id in ids {
            if !self.balls.contains_key(&id)
                || (source_id.is_some() && self.park.pending_removals.contains_key(&id))
            {
                continue;
            }
            if let Some(source) = source {
                let candidate = self.metadata(id)?;
                if !(candidate.is_global || candidate.new_bubble_id == source.new_bubble_id)
                    || (candidate.is_cloaked != 0 && source_id != Some(id))
                {
                    continue;
                }
            }
            selected_ids.push(id);
        }
        if selected_ids.len() > self.park.limits.max_snapshot_balls {
            return Err(CompatError::InvalidRequest(
                "snapshot ball-count limit exceeded".into(),
            ));
        }
        let selected_set = selected_ids.iter().copied().collect::<HashSet<_>>();
        let live_ids = self.balls.keys().copied().collect::<HashSet<_>>();
        let mut balls = selected_ids
            .iter()
            .copied()
            .map(|id| self.ball_snapshot(id))
            .collect::<Result<Vec<_>, _>>()?;
        let mut descriptor_bytes = 0usize;
        for ball in &balls {
            validate_ball_snapshot(ball, self.park.limits.max_child_shapes_per_ball)?;
            for descriptor in ball.minis.iter().chain(&ball.sensors) {
                descriptor_bytes = descriptor_bytes
                    .checked_add(json_value_len(descriptor)?)
                    .ok_or_else(|| {
                        CompatError::InvalidRequest("child descriptor size overflow".into())
                    })?;
                if descriptor_bytes > MAX_CHILD_DESCRIPTOR_BYTES {
                    return Err(CompatError::InvalidRequest(
                        "snapshot child descriptor byte limit exceeded".into(),
                    ));
                }
            }
            for sensor in &ball.sensors {
                for member in sensor
                    .get("members")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let member = value_i64(Some(member), "sensor member")?;
                    if member == ball.id || !live_ids.contains(&member) {
                        return Err(CompatError::InvalidRequest(format!(
                            "ball {} has a self-referential or missing sensor member {member}",
                            ball.id
                        )));
                    }
                }
            }
        }
        for ball in &mut balls {
            for sensor in &mut ball.sensors {
                if let Some(members) = sensor.get_mut("members").and_then(Value::as_array_mut) {
                    members.retain(|member| {
                        member
                            .as_i64()
                            .is_some_and(|id| id != ball.id && selected_set.contains(&id))
                    });
                    members.sort_unstable_by_key(|member| member.as_i64().unwrap_or(i64::MAX));
                }
            }
        }
        let snapshot = ParkSnapshot {
            format: "destiny-bevy-compat-state-v3".into(),
            schema_version: 3,
            park: ParkSnapshotMetadata {
                is_master: self.park.is_master,
                running: self.is_running(),
                tick_interval_ms: self
                    .app
                    .world()
                    .resource::<Time<Fixed>>()
                    .timestep()
                    .as_secs_f64()
                    * 1000.0,
                friction: self.app.world().resource::<DestinySpaceFriction>().0,
                current_time: self.park.current_time,
                local_counter: self.park.local_counter,
                local_high_counter: self.park.local_high_counter,
                time: self.park.time,
                // Filtered/subset snapshots must be independently restorable;
                // never serialize an ego reference to an omitted ball.
                ego: if selected_set.contains(&self.park.ego) {
                    self.park.ego
                } else {
                    0
                },
                collision_substeps: self.park.collision_substeps,
                use_iterative_collision: self.park.use_iterative_collision,
                use_dynamical_orientation: self.park.use_dynamical_orientation,
                disable_dynamical_orientation_for_missiles: self
                    .park
                    .disable_dynamical_orientation_for_missiles,
                use_new_orbit: self.park.use_new_orbit,
                formations: self.park.formations.clone(),
                pending_removals: selected_ids
                    .iter()
                    .filter_map(|ball_id| {
                        self.park.pending_removals.get(ball_id).map(|due_tick| {
                            PendingRemovalSnapshot {
                                ball_id: *ball_id,
                                due_tick: *due_tick,
                                reason: "delayed".into(),
                            }
                        })
                    })
                    .collect(),
                snapshot_semantics: "logical-authoritative".into(),
            },
            balls,
        };
        let bytes = bounded_json_bytes(&snapshot, self.park.limits.max_snapshot_bytes).map_err(
            |error| CompatError::InvalidRequest(format!("snapshot byte limit exceeded: {error}")),
        )?;
        Ok(BASE64.encode(bytes))
    }

    fn ball_snapshot(&self, id: i64) -> Result<BallSnapshot, CompatError> {
        let entity = self.entity(id)?;
        let world = self.app.world();
        let metadata = world
            .get::<DestinyBallMetadata>(entity)
            .ok_or_else(|| CompatError::Engine("missing DestinyBallMetadata".into()))?;
        let position = world
            .get::<Position>(entity)
            .ok_or_else(|| CompatError::Engine("missing Position".into()))?
            .0;
        let rotation = world
            .get::<Rotation>(entity)
            .ok_or_else(|| CompatError::Engine("missing Rotation".into()))?
            .0;
        let velocity = world
            .get::<LinearVelocity>(entity)
            .ok_or_else(|| CompatError::Engine("missing LinearVelocity".into()))?
            .0;
        let angular_velocity = world
            .get::<AngularVelocity>(entity)
            .ok_or_else(|| CompatError::Engine("missing AngularVelocity".into()))?
            .0;
        let mass = world
            .get::<DestinyMass>(entity)
            .ok_or_else(|| CompatError::Engine("missing DestinyMass".into()))?
            .0;
        let max_velocity = world
            .get::<MaxLinearSpeed>(entity)
            .ok_or_else(|| CompatError::Engine("missing MaxLinearSpeed".into()))?
            .0;
        let max_angular_velocity = world
            .get::<MaxAngularSpeed>(entity)
            .ok_or_else(|| CompatError::Engine("missing MaxAngularSpeed".into()))?
            .0;
        Ok(BallSnapshot {
            id,
            mass,
            radius: metadata.radius,
            max_velocity,
            is_free: metadata.is_free,
            is_global: metadata.is_global,
            is_massive: metadata.is_massive,
            is_interactive: metadata.is_interactive,
            is_space_junk: metadata.is_space_junk,
            position: vec![position.x, position.y, position.z],
            velocity: vec![velocity.x, velocity.y, velocity.z],
            agility: metadata.agility,
            speed_fraction: metadata.speed_fraction,
            max_angular_velocity,
            angular_agility: metadata.angular_agility,
            angular_velocity: vec![angular_velocity.x, angular_velocity.y, angular_velocity.z],
            rotation: vec![rotation.x, rotation.y, rotation.z, rotation.w],
            is_cloaked: metadata.is_cloaked,
            new_bubble_id: metadata.new_bubble_id,
            old_bubble_id: metadata.old_bubble_id,
            effect_stamp: metadata.effect_stamp,
            formation_id: metadata.formation_id,
            formation_slots: metadata.formation_slots,
            massive_before_cloak: metadata.massive_before_cloak,
            minis: metadata.minis.clone(),
            mini_ids: metadata.mini_ids.clone(),
            owner_id: metadata.owner_id,
            sensors: metadata.sensors.clone(),
            motion: self.motion.get(&id).map(|control| MotionSnapshot {
                mode: mode_number(control.mode),
                destination: [
                    control.destination.x,
                    control.destination.y,
                    control.destination.z,
                ],
                min_range: control.min_range,
                warp_factor: control.warp_factor,
                warp_distance: control.warp_distance,
                follow_id: control.follow_id,
                follow_range: control.follow_range,
            }),
            notifications: self.range_notifications.get(&id).map(|state| {
                RangeNotificationSnapshot {
                    range: state.range,
                    within: state.within,
                    target: state.target,
                    tracking_range: state.tracking_range,
                    tracking_within: state.tracking_within,
                }
            }),
        })
    }

    fn deserialize_snapshot(&mut self, encoded: &str, partial: i64) -> Result<(), CompatError> {
        // Restore must not replace the live objects behind an operation-owned
        // callback batch. Validate this boundary before any entity is changed.
        if self.evolution_phase != 0 {
            return Err(CompatError::InvalidRequest(
                "cannot restore a snapshot during Evolve".into(),
            ));
        }
        let encoded_limit = self
            .park
            .limits
            .max_snapshot_bytes
            .checked_add(2)
            .and_then(|value| value.checked_div(3))
            .and_then(|value| value.checked_mul(4))
            .ok_or_else(|| CompatError::InvalidRequest("snapshot size limit overflowed".into()))?;
        if encoded.len() > encoded_limit {
            return Err(CompatError::InvalidRequest(
                "encoded snapshot byte limit exceeded".into(),
            ));
        }
        let bytes = BASE64
            .decode(encoded)
            .map_err(|error| CompatError::InvalidRequest(error.to_string()))?;
        if BASE64.encode(&bytes) != encoded {
            return Err(CompatError::InvalidRequest(
                "snapshot base64 must use the canonical encoding".into(),
            ));
        }
        if bytes.len() > self.park.limits.max_snapshot_bytes {
            return Err(CompatError::InvalidRequest(
                "snapshot byte limit exceeded".into(),
            ));
        }
        let snapshot: ParkSnapshot = strict_json_from_slice(&bytes)
            .map_err(|error| CompatError::InvalidRequest(format!("invalid snapshot: {error}")))?;
        if snapshot.format != "destiny-bevy-compat-state-v3" || snapshot.schema_version != 3 {
            return Err(CompatError::InvalidRequest(format!(
                "unsupported snapshot format {}",
                snapshot.format
            )));
        }
        if !matches!(partial, 0..=2) {
            return Err(CompatError::InvalidRequest(
                "partial must be 0, 1, or 2".into(),
            ));
        }
        if snapshot.balls.len() > self.park.limits.max_snapshot_balls {
            return Err(CompatError::InvalidRequest(
                "snapshot ball-count limit exceeded".into(),
            ));
        }
        let snapshot_tick_duration = tick_duration(snapshot.park.tick_interval_ms)?;
        if !snapshot.park.friction.is_finite()
            || snapshot.park.friction < 0.0
            || snapshot.park.current_time < 0
            || snapshot.park.collision_substeps == 0
            || snapshot.park.collision_substeps > MAX_COLLISION_SUBSTEPS
        {
            return Err(CompatError::InvalidRequest(
                "invalid snapshot park timing/controller values".into(),
            ));
        }
        if snapshot.park.use_dynamical_orientation
            || snapshot.park.disable_dynamical_orientation_for_missiles
        {
            return Err(CompatError::InvalidRequest(
                "snapshot enables an unsupported orientation or missile-orientation mode".into(),
            ));
        }
        if snapshot.park.snapshot_semantics != "logical-authoritative" {
            return Err(CompatError::InvalidRequest(
                "unsupported snapshot semantics".into(),
            ));
        }
        validate_formations(&snapshot.park.formations)?;
        if partial != 0 && snapshot.park.formations != self.park.formations {
            return Err(CompatError::InvalidRequest(
                "partial restore requires the same formation definitions".into(),
            ));
        }
        let mut ids = HashSet::new();
        let mut snapshot_descriptor_bytes = 0usize;
        for ball in &snapshot.balls {
            if !ids.insert(ball.id) {
                return Err(CompatError::InvalidRequest(format!(
                    "duplicate snapshot ball id {}",
                    ball.id
                )));
            }
            validate_ball_snapshot(ball, self.park.limits.max_child_shapes_per_ball)?;
            for descriptor in ball.minis.iter().chain(&ball.sensors) {
                snapshot_descriptor_bytes = snapshot_descriptor_bytes
                    .checked_add(json_value_len(descriptor)?)
                    .ok_or_else(|| {
                        CompatError::InvalidRequest("child descriptor size overflow".into())
                    })?;
            }
            if snapshot_descriptor_bytes > MAX_CHILD_DESCRIPTOR_BYTES {
                return Err(CompatError::InvalidRequest(
                    "snapshot child descriptor byte limit exceeded".into(),
                ));
            }
        }
        let incoming_by_id: HashMap<_, _> =
            snapshot.balls.iter().map(|ball| (ball.id, ball)).collect();
        let mut mini_owners = HashMap::new();
        for ball in &snapshot.balls {
            for (index, &mini_id) in ball.mini_ids.iter().enumerate() {
                if mini_id == 0 {
                    continue;
                }
                if mini_id == ball.id || mini_owners.insert(mini_id, ball.id).is_some() {
                    return Err(CompatError::InvalidRequest(
                        "snapshot has aliased/self mini identity".into(),
                    ));
                }
                if let Some(child) = incoming_by_id.get(&mini_id)
                    && (ball
                        .minis
                        .get(index)
                        .and_then(|mini| mini.get("kind"))
                        .and_then(Value::as_str)
                        != Some("sphere")
                        || child.owner_id != ball.id
                        || child
                            .motion
                            .as_ref()
                            .is_none_or(|control| control.mode != 9))
                {
                    return Err(CompatError::InvalidRequest(
                        "snapshot mini identity references unrelated ball".into(),
                    ));
                }
            }
            if let Some(motion) = &ball.motion
                && let Some(target) = motion.follow_id
                && (!(ids.contains(&target) || partial == 2 && self.balls.contains_key(&target))
                    || (matches!(motion.mode, 1 | 4) && target == ball.id))
            {
                return Err(CompatError::InvalidRequest(
                    "snapshot motion references missing/self target".into(),
                ));
            }
            if let Some(state) = &ball.notifications
                && let Some(target) = state.target
                && !ids.contains(&target)
                && !(partial == 2 && self.balls.contains_key(&target))
            {
                return Err(CompatError::InvalidRequest(
                    "snapshot tracking references a missing target".into(),
                ));
            }
            for sensor in &ball.sensors {
                let mut members = HashSet::new();
                for member in sensor
                    .get("members")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_i64)
                {
                    if member == ball.id || !ids.contains(&member) || !members.insert(member) {
                        return Err(CompatError::InvalidRequest(format!(
                            "ball {} has a duplicate, self-referential, or missing sensor member {member}",
                            ball.id
                        )));
                    }
                }
            }
        }
        let mut pending = HashMap::new();
        for row in &snapshot.park.pending_removals {
            if row.reason != "delayed"
                || !ids.contains(&row.ball_id)
                || row.due_tick < snapshot.park.current_time
                || pending.insert(row.ball_id, row.due_tick).is_some()
            {
                return Err(CompatError::InvalidRequest(
                    "invalid, duplicate, missing, or past-due pending removal".into(),
                ));
            }
        }
        if matches!(partial, 0 | 1) && snapshot.park.ego != 0 && !ids.contains(&snapshot.park.ego) {
            return Err(CompatError::InvalidRequest(
                "snapshot ego does not reference a snapshot ball".into(),
            ));
        }
        if partial == 2 && snapshot.park.current_time != self.park.current_time {
            return Err(CompatError::InvalidRequest(
                "partial mode 2 requires a snapshot from the current simulation tick".into(),
            ));
        }
        let committed_count = if partial == 2 {
            self.balls
                .keys()
                .copied()
                .chain(ids.iter().copied())
                .collect::<HashSet<_>>()
                .len()
        } else {
            ids.len()
        };
        if committed_count > self.park.limits.max_snapshot_balls {
            return Err(CompatError::InvalidRequest(
                "restored live ball capacity exceeded".into(),
            ));
        }
        if partial == 1 {
            snapshot_descriptor_bytes = 0;
            for ball in &snapshot.balls {
                let Ok(existing) = self.metadata(ball.id) else {
                    // Legacy rollback semantics consume but do not attach
                    // child descriptors for newly created entities.
                    continue;
                };
                let user_sensors = existing.sensors.iter().filter(|sensor| {
                    !sensor
                        .get("cloak_sensor")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                });
                let incoming_cloak = ball.sensors.iter().filter(|sensor| {
                    sensor
                        .get("cloak_sensor")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                });
                let child_count = existing
                    .minis
                    .len()
                    .saturating_add(user_sensors.clone().count())
                    .saturating_add(incoming_cloak.clone().count());
                if child_count > self.park.limits.max_child_shapes_per_ball {
                    return Err(CompatError::InvalidRequest(
                        "rollback-preserved child-shape limit exceeded".into(),
                    ));
                }
                for mini in &existing.minis {
                    mini_collider(mini)?;
                }
                for sensor in existing.sensors.iter().filter(|sensor| {
                    !sensor
                        .get("cloak_sensor")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                }) {
                    validate_sensor_descriptor(sensor)?;
                }
                for descriptor in existing
                    .minis
                    .iter()
                    .chain(user_sensors)
                    .chain(incoming_cloak)
                {
                    snapshot_descriptor_bytes = snapshot_descriptor_bytes
                        .checked_add(json_value_len(descriptor)?)
                        .ok_or_else(|| {
                            CompatError::InvalidRequest("child descriptor size overflow".into())
                        })?;
                }
            }
        } else if partial == 2 {
            for existing_id in self.balls.keys().copied().filter(|id| !ids.contains(id)) {
                let metadata = self.metadata(existing_id)?;
                for descriptor in metadata.minis.iter().chain(&metadata.sensors) {
                    snapshot_descriptor_bytes = snapshot_descriptor_bytes
                        .checked_add(json_value_len(descriptor)?)
                        .ok_or_else(|| {
                            CompatError::InvalidRequest("child descriptor size overflow".into())
                        })?;
                }
            }
        }
        if snapshot_descriptor_bytes > MAX_CHILD_DESCRIPTOR_BYTES {
            return Err(CompatError::InvalidRequest(
                "restored live child descriptor byte limit exceeded".into(),
            ));
        }

        // Partial restores update existing entities in place. Reject an
        // externally corrupted ECS world before deleting or changing anything
        // so an ordinary component error cannot turn restore into a partial
        // commit.
        if partial != 0 {
            self.validate_authoritative_world(false)?;
        }

        let preserved_generations = self
            .ball_generations
            .iter()
            .filter(|(id, _)| ids.contains(id))
            .map(|(id, generation)| {
                (
                    *id,
                    (
                        *generation,
                        self.retained_balls[generation].references,
                        self.retained_balls[generation].client,
                        self.retained_balls[generation].identity_fields,
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        self.next_ball_generation
            .checked_add(snapshot.balls.len() as u64)
            .ok_or_else(|| {
                CompatError::InvalidRequest("restored object generations exhausted".into())
            })?;
        // Everything above is detached parsing/validation. No live state is
        // mutated before the complete envelope passes those checks.
        if partial == 0 {
            self.clear_all()?;
            self.park.is_master = snapshot.park.is_master;
            self.park.local_counter = snapshot.park.local_counter;
            self.park.local_high_counter = snapshot.park.local_high_counter;
            self.park.use_iterative_collision = snapshot.park.use_iterative_collision;
            self.park.collision_substeps = snapshot.park.collision_substeps;
            self.park.use_dynamical_orientation = snapshot.park.use_dynamical_orientation;
            self.park.disable_dynamical_orientation_for_missiles =
                snapshot.park.disable_dynamical_orientation_for_missiles;
            self.park.use_new_orbit = snapshot.park.use_new_orbit;
            self.park.formations = snapshot.park.formations.clone();
            self.app
                .world_mut()
                .resource_mut::<Time<Fixed>>()
                .set_timestep(snapshot_tick_duration);
            self.app.world_mut().resource_mut::<SubstepCount>().0 =
                if snapshot.park.use_iterative_collision {
                    snapshot.park.collision_substeps
                } else {
                    1
                };
            self.app
                .world_mut()
                .resource_mut::<DestinySpaceFriction>()
                .0 = snapshot.park.friction;
        } else if partial == 1 {
            // Dedicated rollback mode restores the checkpoint's authoritative
            // entity set while retaining child descriptors only for entities
            // that existed at both times.
            let future_ids = self
                .balls
                .keys()
                .filter(|ball_id| !ids.contains(ball_id))
                .copied()
                .collect::<Vec<_>>();
            for ball_id in future_ids {
                self.remove_ball(ball_id)?;
            }
        }
        for ball in snapshot.balls {
            let partial_one_sensors = if partial == 1 && self.balls.contains_key(&ball.id) {
                let mut sensors = self
                    .metadata(ball.id)?
                    .sensors
                    .iter()
                    .filter(|sensor| {
                        !sensor
                            .get("cloak_sensor")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                sensors.extend(
                    ball.sensors
                        .iter()
                        .filter(|sensor| {
                            sensor
                                .get("cloak_sensor")
                                .and_then(Value::as_bool)
                                .unwrap_or(false)
                        })
                        .cloned(),
                );
                for sensor in &mut sensors {
                    if let Some(members) = sensor.get_mut("members").and_then(Value::as_array_mut) {
                        members.retain(|member| {
                            member
                                .as_i64()
                                .is_some_and(|member| member != ball.id && ids.contains(&member))
                        });
                    }
                }
                Some(sensors)
            } else {
                None
            };
            if partial != 1 && self.balls.contains_key(&ball.id) {
                // Partial mode 2 replaces one entity in place. References to
                // that logical ID in other sensors/events remain valid and
                // must not be scrubbed as if the ball had been deleted.
                self.remove_ball_internal(ball.id, partial != 2)?;
            }
            let position = vec3_from_slice(&ball.position, "position")?;
            let velocity = vec3_from_slice(&ball.velocity, "velocity")?;
            let mut args = vec![
                json!(ball.id),
                json!(ball.mass),
                json!(ball.radius),
                json!(ball.max_velocity),
                json!(ball.is_free),
                json!(ball.is_global),
                json!(ball.is_massive),
                json!(ball.is_interactive),
                json!(ball.is_space_junk),
                json!(position.x),
                json!(position.y),
                json!(position.z),
                json!(velocity.x),
                json!(velocity.y),
                json!(velocity.z),
                json!(ball.agility),
                json!(ball.speed_fraction),
            ];
            if self.park.use_dynamical_orientation {
                args.extend([
                    json!(ball.max_angular_velocity),
                    json!(ball.angular_agility),
                ]);
            }
            self.add_ball_with_identity(&args, Some(ball.id))?;
            if let Some((generation, references, client, identity_fields)) =
                preserved_generations.get(&ball.id)
            {
                let temporary = self
                    .ball_generations
                    .insert(ball.id, *generation)
                    .expect("restored object");
                if temporary != *generation {
                    self.retained_balls.remove(&temporary);
                }
                self.retained_balls.insert(
                    *generation,
                    RetainedBall {
                        id: ball.id,
                        attached: true,
                        moribund: false,
                        references: *references,
                        detached: None,
                        client: *client,
                        identity_fields: *identity_fields,
                    },
                );
            }
            self.metadata_mut(ball.id)?.mini_ids = ball.mini_ids.clone();
            self.set_max_angular_velocity(ball.id, ball.max_angular_velocity)?;
            self.set_rotation(ball.id, quat_from_slice(&ball.rotation)?)?;
            self.set_angular_velocity(
                ball.id,
                vec3_from_slice(&ball.angular_velocity, "angular_velocity")?,
            )?;
            if partial != 1 {
                self.restore_child_shapes(ball.id, &ball.minis, &ball.sensors)?;
            }
            let entity = self.entity(ball.id)?;
            {
                let mut metadata = self
                    .app
                    .world_mut()
                    .get_mut::<DestinyBallMetadata>(entity)
                    .ok_or_else(|| CompatError::Engine("missing DestinyBallMetadata".into()))?;
                metadata.is_cloaked = ball.is_cloaked;
                metadata.new_bubble_id = ball.new_bubble_id;
                metadata.old_bubble_id = ball.old_bubble_id;
                metadata.effect_stamp = ball.effect_stamp;
                metadata.formation_id = ball.formation_id;
                metadata.formation_slots = ball.formation_slots;
                metadata.massive_before_cloak = ball.massive_before_cloak;
                metadata.angular_agility = ball.angular_agility;
                metadata.agility = ball.agility;
                metadata.owner_id = ball.owner_id;
                metadata.mini_ids = ball.mini_ids.clone();
                if let Some(sensors) = partial_one_sensors {
                    metadata.sensors = sensors;
                }
            }
            self.set_massive(ball.id, ball.is_cloaked == 0 && ball.is_massive)?;
            if let Some(control) = ball.motion {
                let mode = match control.mode {
                    0 => OriginalMode::Goto,
                    1 => OriginalMode::Follow,
                    2 => OriginalMode::Stop,
                    3 => OriginalMode::Warp,
                    4 => OriginalMode::Orbit,
                    5 => OriginalMode::Missile,
                    6 => OriginalMode::Mushroom,
                    7 => OriginalMode::Boid,
                    8 => OriginalMode::Troll,
                    9 => OriginalMode::Miniball,
                    10 => OriginalMode::Field,
                    11 => OriginalMode::Rigid,
                    12 => OriginalMode::Formation,
                    _ => {
                        return Err(CompatError::InvalidRequest(
                            "unsupported restored motion mode".into(),
                        ));
                    }
                };
                self.motion.insert(
                    ball.id,
                    MotionControl {
                        mode,
                        destination: OriginalVec3 {
                            x: control.destination[0],
                            y: control.destination[1],
                            z: control.destination[2],
                        },
                        min_range: control.min_range,
                        warp_factor: control.warp_factor,
                        warp_distance: control.warp_distance,
                        follow_id: control.follow_id,
                        follow_range: control.follow_range,
                    },
                );
            } else {
                self.motion.remove(&ball.id);
            }
            if let Some(state) = ball.notifications {
                self.range_notifications.insert(
                    ball.id,
                    RangeNotifications {
                        range: state.range,
                        within: state.within,
                        target: state.target,
                        tracking_range: state.tracking_range,
                        tracking_within: state.tracking_within,
                    },
                );
            } else {
                self.range_notifications.remove(&ball.id);
            }
        }
        if matches!(partial, 0 | 1) {
            self.park.pending_removals.clear();
            let entities = self.balls.values().copied().collect::<Vec<_>>();
            for entity in entities {
                self.app
                    .world_mut()
                    .entity_mut(entity)
                    .remove::<DestinyPendingRemoval>();
            }
            for (ball_id, due_tick) in pending {
                self.park.pending_removals.insert(ball_id, due_tick);
                self.transition_retained_ball(ball_id, self.ball_generations[&ball_id], 5)?;
                let entity = self.entity(ball_id)?;
                self.app
                    .world_mut()
                    .entity_mut(entity)
                    .insert(DestinyPendingRemoval {
                        due_tick,
                        reason: "delayed".into(),
                    });
            }
            self.park.current_time = snapshot.park.current_time;
            self.park.time = snapshot.park.time;
            self.park.ego = snapshot.park.ego;
            self.app
                .world_mut()
                .resource_mut::<ProximityEventOutbox>()
                .0
                .clear();
            *self.app.world_mut().resource_mut::<CarbonNetworkOutbox>() =
                CarbonNetworkOutbox::default();
            let mut physics_time = self.app.world_mut().resource_mut::<Time<Physics>>();
            if snapshot.park.running {
                physics_time.unpause();
            } else {
                physics_time.pause();
            }
        } else {
            for ball_id in &ids {
                self.park.pending_removals.remove(ball_id);
                if let Some(entity) = self.balls.get(ball_id).copied() {
                    self.app
                        .world_mut()
                        .entity_mut(entity)
                        .remove::<DestinyPendingRemoval>();
                }
            }
            for (ball_id, due_tick) in pending {
                self.park.pending_removals.insert(ball_id, due_tick);
                self.transition_retained_ball(ball_id, self.ball_generations[&ball_id], 5)?;
                let entity = self.entity(ball_id)?;
                self.app
                    .world_mut()
                    .entity_mut(entity)
                    .insert(DestinyPendingRemoval {
                        due_tick,
                        reason: "delayed".into(),
                    });
            }
        }
        self.rebuild_follower_sets();
        self.sync_visual_transforms();
        Ok(())
    }

    fn restore_child_shapes(
        &mut self,
        ball_id: i64,
        minis: &[Value],
        sensors: &[Value],
    ) -> Result<(), CompatError> {
        for mini in minis {
            let object = mini.as_object().ok_or_else(|| {
                CompatError::InvalidRequest("snapshot mini collider must be an object".into())
            })?;
            let kind = object.get("kind").and_then(Value::as_str).ok_or_else(|| {
                CompatError::InvalidRequest("snapshot mini collider is missing kind".into())
            })?;
            let (title, args) = match kind {
                "sphere" => {
                    let position = object
                        .get("position")
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            CompatError::InvalidRequest(
                                "snapshot mini sphere is missing position".into(),
                            )
                        })?;
                    if position.len() != 3 {
                        return Err(CompatError::InvalidRequest(
                            "snapshot mini sphere position must have three elements".into(),
                        ));
                    }
                    let radius = object.get("radius").cloned().ok_or_else(|| {
                        CompatError::InvalidRequest("snapshot mini sphere is missing radius".into())
                    })?;
                    (
                        "destiny.Ball.AddMiniBall",
                        vec![
                            position[0].clone(),
                            position[1].clone(),
                            position[2].clone(),
                            radius,
                        ],
                    )
                }
                "capsule" => {
                    let a = object.get("a").and_then(Value::as_array).ok_or_else(|| {
                        CompatError::InvalidRequest(
                            "snapshot mini capsule is missing endpoint a".into(),
                        )
                    })?;
                    let b = object.get("b").and_then(Value::as_array).ok_or_else(|| {
                        CompatError::InvalidRequest(
                            "snapshot mini capsule is missing endpoint b".into(),
                        )
                    })?;
                    if a.len() != 3 || b.len() != 3 {
                        return Err(CompatError::InvalidRequest(
                            "snapshot mini capsule endpoints must have three elements".into(),
                        ));
                    }
                    let radius = object.get("radius").cloned().ok_or_else(|| {
                        CompatError::InvalidRequest(
                            "snapshot mini capsule is missing radius".into(),
                        )
                    })?;
                    (
                        "destiny.Ball.AddMiniCapsule",
                        vec![
                            a[0].clone(),
                            a[1].clone(),
                            a[2].clone(),
                            b[0].clone(),
                            b[1].clone(),
                            b[2].clone(),
                            radius,
                        ],
                    )
                }
                "box" => {
                    let basis = object
                        .get("basis")
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            CompatError::InvalidRequest("snapshot mini box is missing basis".into())
                        })?;
                    if basis.len() != 12 {
                        return Err(CompatError::InvalidRequest(
                            "snapshot mini box basis must have twelve elements".into(),
                        ));
                    }
                    ("destiny.Ball.AddMiniBox", basis.clone())
                }
                other => {
                    return Err(CompatError::InvalidRequest(format!(
                        "unsupported snapshot mini collider kind {other:?}",
                    )));
                }
            };
            self.dispatch_ball(title, "call", ball_id, &args)?;
        }

        self.metadata_mut(ball_id)?.sensors = sensors.to_vec();
        Ok(())
    }

    fn ensure_child_descriptor_budget(
        &self,
        ball_id: i64,
        descriptor: &Value,
        replace_user_sensor: bool,
        replace_cloak_sensor: bool,
    ) -> Result<(), CompatError> {
        self.entity(ball_id)?;
        let mut bytes = 0usize;
        for candidate_id in self.balls.keys().copied() {
            let metadata = self.metadata(candidate_id)?;
            for value in metadata
                .minis
                .iter()
                .chain(metadata.sensors.iter().filter(|sensor| {
                    if candidate_id != ball_id {
                        return true;
                    }
                    let is_cloak = sensor
                        .get("cloak_sensor")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    !((replace_user_sensor && !is_cloak) || (replace_cloak_sensor && is_cloak))
                }))
            {
                bytes = bytes.checked_add(json_value_len(value)?).ok_or_else(|| {
                    CompatError::InvalidRequest("child descriptor size overflow".into())
                })?;
            }
        }
        bytes = bytes
            .checked_add(json_value_len(descriptor)?)
            .ok_or_else(|| CompatError::InvalidRequest("child descriptor size overflow".into()))?;
        if bytes > MAX_CHILD_DESCRIPTOR_BYTES {
            return Err(CompatError::InvalidRequest(
                "live child descriptor byte limit exceeded".into(),
            ));
        }
        Ok(())
    }

    fn dispatch_ball(
        &mut self,
        title: &str,
        operation: &str,
        ball_id: i64,
        args: &[Value],
    ) -> Result<Value, CompatError> {
        let name = title.rsplit('.').next().unwrap_or(title);
        if operation == "get" {
            require_arity(args, &[0], title)?;
            return self
                .get_ball_property(ball_id, name)
                .map_err(|error| match error {
                    CompatError::UnsupportedTitle(_) => {
                        CompatError::UnsupportedTitle(title.to_owned())
                    }
                    other => other,
                });
        }
        if operation == "set" {
            require_arity(args, &[1], title)?;
            return self.set_ball_property(ball_id, name, args.first()).map_err(
                |error| match error {
                    CompatError::UnsupportedTitle(_) => {
                        CompatError::UnsupportedTitle(title.to_owned())
                    }
                    other => other,
                },
            );
        }
        if operation != "call" {
            return Err(CompatError::InvalidRequest(format!(
                "{title} does not support operation {operation:?}"
            )));
        }
        match name {
            "ReserveFormationSlot" => {
                require_arity(args, &[0], title)?;
                let count = self.formation_slot_count(ball_id)?;
                let mut metadata = self.metadata_mut(ball_id)?;
                let (reserved, slot) = reserve_formation_slot(metadata.formation_slots, count);
                metadata.formation_slots = reserved;
                Ok(json!(slot))
            }
            "FreeFormationSlot" => {
                require_arity(args, &[1], title)?;
                let slot = value_i64(args.first(), "slot")?;
                let count = self.formation_slot_count(ball_id)?;
                let mut metadata = self.metadata_mut(ball_id)?;
                metadata.formation_slots =
                    free_formation_slot(metadata.formation_slots, count, slot);
                Ok(Value::Null)
            }
            "GetRotatedVector" => {
                require_arity(args, &[1], title)?;
                let vector = value_vec3(args.first(), "vector")?;
                let entity = self.entity(ball_id)?;
                let rotation = self
                    .app
                    .world()
                    .get::<Rotation>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Rotation".into()))?
                    .0;
                let result = rotation * vector;
                if !result.is_finite() {
                    return Err(CompatError::InvalidRequest(
                        "rotated vector exceeds the finite output range".into(),
                    ));
                }
                Ok(json!([result.x, result.y, result.z]))
            }
            "AddMiniBall" => {
                if args.len() != 4 {
                    return Err(CompatError::InvalidRequest(
                        "AddMiniBall expects 4 arguments".into(),
                    ));
                }
                let x = value_f64(args.first(), "x")?;
                let y = value_f64(args.get(1), "y")?;
                let z = value_f64(args.get(2), "z")?;
                let radius = value_f64(args.get(3), "radius")?;
                self.entity(ball_id)?;
                if self.metadata(ball_id)?.minis.len() + self.metadata(ball_id)?.sensors.len()
                    >= self.park.limits.max_child_shapes_per_ball
                {
                    return Err(CompatError::InvalidRequest(
                        "child-shape limit exceeded".into(),
                    ));
                }
                let descriptor = json!({"kind":"sphere","position":[x,y,z],"radius":radius});
                mini_collider(&descriptor)?;
                self.ensure_child_descriptor_budget(ball_id, &descriptor, false, false)?;
                self.metadata_mut(ball_id)?.minis.push(descriptor);
                self.register_fixed_mini_objects(ball_id)?;
                if let Err(error) = self.rebuild_root_collider(ball_id) {
                    self.metadata_mut(ball_id)?.minis.pop();
                    return Err(error);
                }
                Ok(Value::Null)
            }
            "AddMiniCapsule" => {
                if args.len() != 7 {
                    return Err(CompatError::InvalidRequest(
                        "AddMiniCapsule expects 7 arguments".into(),
                    ));
                }
                let a = DVec3::new(
                    value_f64(args.first(), "ax")?,
                    value_f64(args.get(1), "ay")?,
                    value_f64(args.get(2), "az")?,
                );
                let b = DVec3::new(
                    value_f64(args.get(3), "bx")?,
                    value_f64(args.get(4), "by")?,
                    value_f64(args.get(5), "bz")?,
                );
                let radius = f64::from(value_f64(args.get(6), "radius")? as f32);
                if radius <= 0.0 {
                    return Err(CompatError::ValueError("Radius must be positive".into()));
                }
                self.entity(ball_id)?;
                if self.metadata(ball_id)?.minis.len() + self.metadata(ball_id)?.sensors.len()
                    >= self.park.limits.max_child_shapes_per_ball
                {
                    return Err(CompatError::InvalidRequest(
                        "child-shape limit exceeded".into(),
                    ));
                }
                let descriptor =
                    json!({"kind":"capsule","a":[a.x,a.y,a.z],"b":[b.x,b.y,b.z],"radius":radius});
                mini_collider(&descriptor)?;
                self.ensure_child_descriptor_budget(ball_id, &descriptor, false, false)?;
                self.metadata_mut(ball_id)?.minis.push(descriptor);
                self.register_fixed_mini_objects(ball_id)?;
                if let Err(error) = self.rebuild_root_collider(ball_id) {
                    self.metadata_mut(ball_id)?.minis.pop();
                    return Err(error);
                }
                Ok(Value::Null)
            }
            "AddMiniBox" => {
                if args.len() != 12 {
                    return Err(CompatError::InvalidRequest(
                        "AddMiniBox expects 12 arguments".into(),
                    ));
                }
                let values = args
                    .iter()
                    .enumerate()
                    .map(|(index, value)| value_f64(Some(value), &format!("component_{index}")))
                    .collect::<Result<Vec<_>, _>>()?;
                let corner = DVec3::new(values[0], values[1], values[2]);
                let axis_x = DVec3::new(values[3], values[4], values[5]);
                let axis_y = DVec3::new(values[6], values[7], values[8]);
                let axis_z = DVec3::new(values[9], values[10], values[11]);
                let lengths = DVec3::new(
                    stable_vec3_length(axis_x),
                    stable_vec3_length(axis_y),
                    stable_vec3_length(axis_z),
                );
                if !lengths.is_finite() || lengths.min_element() <= 0.0 {
                    return Err(CompatError::InvalidRequest(
                        "MiniBox axes must be finite and non-zero".into(),
                    ));
                }
                let unit_x = axis_x / lengths.x;
                let unit_y = axis_y / lengths.y;
                let unit_z = axis_z / lengths.z;
                if unit_x.dot(unit_y).abs() > 1e-6
                    || unit_x.dot(unit_z).abs() > 1e-6
                    || unit_y.dot(unit_z).abs() > 1e-6
                {
                    return Err(CompatError::InvalidRequest(
                        "MiniBox axes must be mutually orthogonal".into(),
                    ));
                }
                let basis = DMat3::from_cols(unit_x, unit_y, unit_z);
                if !basis.is_finite() || basis.determinant() <= 0.0 {
                    return Err(CompatError::InvalidRequest(
                        "MiniBox basis must be finite and right-handed".into(),
                    ));
                }
                let rotation = DQuat::from_mat3(&basis).normalize();
                let center = corner + (axis_x + axis_y + axis_z) * 0.5;
                if !rotation.is_finite() || !center.is_finite() {
                    return Err(CompatError::InvalidRequest(
                        "MiniBox transform overflowed".into(),
                    ));
                }
                self.entity(ball_id)?;
                if self.metadata(ball_id)?.minis.len() + self.metadata(ball_id)?.sensors.len()
                    >= self.park.limits.max_child_shapes_per_ball
                {
                    return Err(CompatError::InvalidRequest(
                        "child-shape limit exceeded".into(),
                    ));
                }
                let descriptor = json!({"kind":"box","basis":values});
                // Validate the exact stored representation before changing
                // metadata, keeping a failed add transactional.
                mini_collider(&descriptor)?;
                self.ensure_child_descriptor_budget(ball_id, &descriptor, false, false)?;
                self.metadata_mut(ball_id)?.minis.push(descriptor);
                self.register_fixed_mini_objects(ball_id)?;
                if let Err(error) = self.rebuild_root_collider(ball_id) {
                    self.metadata_mut(ball_id)?.minis.pop();
                    return Err(error);
                }
                Ok(Value::Null)
            }
            "AddProximitySensor" => self.add_proximity_sensor(ball_id, args),
            "ApplyImpulsiveForceAtPosition" => {
                if args.len() != 2 {
                    return Err(CompatError::InvalidRequest(
                        "ApplyImpulsiveForceAtPosition expects force and position".into(),
                    ));
                }
                let force = value_vec3(args.first(), "force")?;
                let lever = value_vec3(args.get(1), "position")?;
                let entity = self.entity(ball_id)?;
                let world = self.app.world();
                let mass = world
                    .get::<DestinyMass>(entity)
                    .ok_or_else(|| CompatError::Engine("missing DestinyMass".into()))?
                    .0;
                let radius = world
                    .get::<DestinyBallMetadata>(entity)
                    .ok_or_else(|| CompatError::Engine("missing DestinyBallMetadata".into()))?
                    .radius;
                let max_speed = world
                    .get::<MaxAngularSpeed>(entity)
                    .ok_or_else(|| CompatError::Engine("missing MaxAngularSpeed".into()))?
                    .0;
                let current = world
                    .get::<DestinyPresentationAngularVelocity>(entity)
                    .ok_or_else(|| {
                        CompatError::Engine("missing DestinyPresentationAngularVelocity".into())
                    })?
                    .0;
                let inertia = (0.4 * mass * radius * radius).max(f64::MIN_POSITIVE);
                let torque = lever.cross(force);
                if !torque.is_finite() || inertia.is_nan() {
                    return Err(CompatError::InvalidRequest(
                        "impulse-derived torque or inertia overflowed".into(),
                    ));
                }
                let delta = if inertia.is_infinite() {
                    DVec3::ZERO
                } else {
                    0.05 * torque / inertia
                };
                let mut angular = current + delta;
                if !angular.is_finite() {
                    return Err(CompatError::InvalidRequest(
                        "impulse-derived angular velocity overflowed".into(),
                    ));
                }
                let speed = stable_vec3_length(angular);
                if speed > max_speed && max_speed > 0.0 {
                    angular = stable_vec3_normalize(angular)
                        .map_or(DVec3::ZERO, |direction| direction * max_speed);
                }
                self.app
                    .world_mut()
                    .get_mut::<DestinyPresentationAngularVelocity>(entity)
                    .ok_or_else(|| {
                        CompatError::Engine("missing DestinyPresentationAngularVelocity".into())
                    })?
                    .0 = angular;
                Ok(Value::Null)
            }
            _ => Err(CompatError::UnsupportedTitle(title.to_owned())),
        }
    }

    fn get_ball_property(&self, ball_id: i64, name: &str) -> Result<Value, CompatError> {
        let entity = self.entity(ball_id)?;
        let world = self.app.world();
        let metadata = world
            .get::<DestinyBallMetadata>(entity)
            .ok_or_else(|| CompatError::Engine("missing DestinyBallMetadata".into()))?;
        match name {
            "id" => Ok(json!(ball_id)),
            "harmonic" | "corporationID" | "allianceID" => {
                let retained = &self.retained_balls[&self.ball_generations[&ball_id]];
                Ok(json!(
                    retained.identity_fields[match name {
                        "harmonic" => 0,
                        "corporationID" => 1,
                        _ => 2,
                    }]
                ))
            }
            "isMoribund" => Ok(json!(
                self.retained_balls[&self.ball_generations[&ball_id]].moribund
            )),
            "ownerId" => Ok(json!(metadata.owner_id)),
            "followId" => Ok(json!(self.motion.get(&ball_id).map_or(0, |control| {
                if control.mode == OriginalMode::Warp {
                    control.min_range.to_bits() as i64
                } else {
                    control.follow_id.unwrap_or(0)
                }
            }))),
            "followRange" => json_number(
                self.motion
                    .get(&ball_id)
                    .map_or(10.0, |control| f64::from(control.follow_range)),
            ),
            "gotoX" | "gotoY" | "gotoZ" => {
                let point = self
                    .motion
                    .get(&ball_id)
                    .map_or(OriginalVec3::ZERO, |control| control.destination);
                json_number(match name {
                    "gotoX" => point.x,
                    "gotoY" => point.y,
                    _ => point.z,
                })
            }
            "miniBalls" | "miniCapsules" | "miniBoxes" => {
                let kind = match name {
                    "miniBalls" => "sphere",
                    "miniCapsules" => "capsule",
                    _ => "box",
                };
                Ok(Value::Array(
                    metadata
                        .minis
                        .iter()
                        .enumerate()
                        .filter(|(_, mini)| mini.get("kind").and_then(Value::as_str) == Some(kind))
                        .map(|(index, mini)| {
                            let mut row = mini.clone();
                            row["id"] = json!(metadata.mini_ids.get(index).copied().unwrap_or(0));
                            row
                        })
                        .collect(),
                ))
            }
            "mode" => Ok(json!(mode_number(
                self.motion
                    .get(&ball_id)
                    .map_or(OriginalMode::Stop, |control| control.mode)
            ))),
            "ballpark" => Ok(json!("park:0")),
            "centerDist" | "surfaceDist" => {
                if self.park.ego <= 0
                    || self.park.ego == ball_id
                    || !self.balls.contains_key(&self.park.ego)
                {
                    return json_number(0.0);
                }
                let center =
                    stable_vec3_length(self.position(ball_id)? - self.position(self.park.ego)?);
                if name == "centerDist" {
                    json_number(center)
                } else {
                    let ego_radius = self.metadata(self.park.ego)?.radius;
                    json_number((center - metadata.radius - ego_radius).max(0.0))
                }
            }
            "mass" => json_number(
                world
                    .get::<DestinyMass>(entity)
                    .ok_or_else(|| CompatError::Engine("missing DestinyMass".into()))?
                    .0,
            ),
            "radius" => json_number(metadata.radius),
            "maxVelocity" => json_number(
                world
                    .get::<MaxLinearSpeed>(entity)
                    .ok_or_else(|| CompatError::Engine("missing MaxLinearSpeed".into()))?
                    .0,
            ),
            "maxAngularVelocity" => json_number(
                world
                    .get::<MaxAngularSpeed>(entity)
                    .ok_or_else(|| CompatError::Engine("missing MaxAngularSpeed".into()))?
                    .0,
            ),
            "x" | "y" | "z" => {
                let value = world
                    .get::<Position>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Position".into()))?
                    .0;
                json_number(match name {
                    "x" => value.x,
                    "y" => value.y,
                    _ => value.z,
                })
            }
            "vx" | "vy" | "vz" => {
                let value = world
                    .get::<LinearVelocity>(entity)
                    .ok_or_else(|| CompatError::Engine("missing LinearVelocity".into()))?
                    .0;
                json_number(match name {
                    "vx" => value.x,
                    "vy" => value.y,
                    _ => value.z,
                })
            }
            "wx" | "wy" | "wz" => {
                let value = world
                    .get::<AngularVelocity>(entity)
                    .ok_or_else(|| CompatError::Engine("missing AngularVelocity".into()))?
                    .0;
                json_number(match name {
                    "wx" => value.x,
                    "wy" => value.y,
                    _ => value.z,
                })
            }
            "rx" | "ry" | "rz" | "rw" => {
                let value = world
                    .get::<Rotation>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Rotation".into()))?
                    .0;
                json_number(match name {
                    "rx" => value.x,
                    "ry" => value.y,
                    "rz" => value.z,
                    _ => value.w,
                })
            }
            "roll" | "pitch" | "yaw" => {
                let rotation = world
                    .get::<Rotation>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Rotation".into()))?
                    .0;
                let (roll, pitch, yaw) = rotation.to_euler(EulerRot::XYZ);
                json_number(match name {
                    "roll" => roll,
                    "pitch" => pitch,
                    _ => yaw,
                })
            }
            "isFree" => Ok(json!(metadata.is_free)),
            "isGlobal" => Ok(json!(metadata.is_global)),
            "isMassive" => Ok(json!(metadata.is_massive)),
            "isInteractive" => Ok(json!(metadata.is_interactive)),
            "isCloaked" => Ok(json!(metadata.is_cloaked)),
            "newBubbleId" => Ok(json!(metadata.new_bubble_id)),
            "oldBubbleId" => Ok(json!(metadata.old_bubble_id)),
            "effectStamp" => Ok(json!(metadata.effect_stamp)),
            "formationID" => Ok(json!(metadata.formation_id)),
            "Agility" => json_number(metadata.agility),
            "speedFraction" => json_number(metadata.speed_fraction),
            _ => Err(CompatError::UnsupportedTitle(format!(
                "destiny.Ball.{name}"
            ))),
        }
    }

    fn set_ball_property(
        &mut self,
        ball_id: i64,
        name: &str,
        value: Option<&Value>,
    ) -> Result<Value, CompatError> {
        match name {
            "mass" => self.set_mass(ball_id, value_f64(value, "mass")?.max(0.0))?,
            "radius" => self.set_radius(ball_id, value_f64(value, "radius")?.max(0.0))?,
            "maxVelocity" => {
                self.set_max_velocity(ball_id, value_f64(value, "maxVelocity")?.max(0.0))?
            }
            "maxAngularVelocity" => self.set_max_angular_velocity(
                ball_id,
                value_f64(value, "maxAngularVelocity")?.max(0.0),
            )?,
            "x" | "y" | "z" => {
                let entity = self.entity(ball_id)?;
                let mut position = self
                    .app
                    .world()
                    .get::<Position>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Position".into()))?
                    .0;
                let component = value_f64(value, name)?;
                match name {
                    "x" => position.x = component,
                    "y" => position.y = component,
                    _ => position.z = component,
                }
                self.set_position(ball_id, position)?;
            }
            "vx" | "vy" | "vz" => {
                let entity = self.entity(ball_id)?;
                let mut velocity = self
                    .app
                    .world()
                    .get::<LinearVelocity>(entity)
                    .ok_or_else(|| CompatError::Engine("missing LinearVelocity".into()))?
                    .0;
                let component = value_f64(value, name)?;
                match name {
                    "vx" => velocity.x = component,
                    "vy" => velocity.y = component,
                    _ => velocity.z = component,
                }
                self.set_velocity(ball_id, velocity)?;
            }
            "wx" | "wy" | "wz" => {
                let entity = self.entity(ball_id)?;
                let mut velocity = self
                    .app
                    .world()
                    .get::<AngularVelocity>(entity)
                    .ok_or_else(|| CompatError::Engine("missing AngularVelocity".into()))?
                    .0;
                let component = value_f64(value, name)?;
                match name {
                    "wx" => velocity.x = component,
                    "wy" => velocity.y = component,
                    _ => velocity.z = component,
                }
                self.set_angular_velocity(ball_id, velocity)?;
            }
            "rx" | "ry" | "rz" | "rw" => {
                let entity = self.entity(ball_id)?;
                let current = self
                    .app
                    .world()
                    .get::<Rotation>(entity)
                    .ok_or_else(|| CompatError::Engine("missing Rotation".into()))?
                    .0;
                let mut components = [current.x, current.y, current.z, current.w];
                components[match name {
                    "rx" => 0,
                    "ry" => 1,
                    "rz" => 2,
                    _ => 3,
                }] = value_f64(value, name)?;
                self.set_rotation(
                    ball_id,
                    normalized_quat(DQuat::from_xyzw(
                        components[0],
                        components[1],
                        components[2],
                        components[3],
                    ))?,
                )?;
            }
            "roll" | "pitch" | "yaw" | "newBubbleId" | "oldBubbleId" | "effectStamp" => {
                return Err(CompatError::InvalidRequest(format!(
                    "destiny.Ball.{name} is read-only"
                )));
            }
            "isFree" => self.set_free(ball_id, value_bool(value, "isFree")?)?,
            "isGlobal" => self.metadata_mut(ball_id)?.is_global = value_bool(value, "isGlobal")?,
            "isMassive" => self.set_massive(ball_id, value_bool(value, "isMassive")?)?,
            "isInteractive" => {
                self.set_interactive(ball_id, value_bool(value, "isInteractive")?)?
            }
            "isCloaked" => {
                let mode = value_i64(value, "isCloaked")?;
                if !(0..=3).contains(&mode) {
                    return Err(CompatError::InvalidRequest(
                        "isCloaked must be in 0..=3".into(),
                    ));
                }
                if mode == 0 {
                    self.uncloak_ball(ball_id)?;
                } else {
                    self.cloak_ball(ball_id, mode as i32, None)?;
                }
            }
            "Agility" => {
                let agility = value_f64(value, "Agility")?;
                let agility = if agility <= 0.0 { 1.0 } else { agility };
                if agility > MAX_AGILITY {
                    return Err(CompatError::InvalidRequest(
                        "Agility exceeds the solver-safe limit".into(),
                    ));
                }
                self.metadata_mut(ball_id)?.agility = agility;
            }
            "speedFraction" => {
                self.metadata_mut(ball_id)?.speed_fraction =
                    value_f64(value, "speedFraction")?.clamp(0.0, 1.0)
            }
            _ => {
                return Err(CompatError::UnsupportedTitle(format!(
                    "destiny.Ball.{name}"
                )));
            }
        }
        Ok(Value::Null)
    }

    fn set_interactive(&mut self, id: i64, enabled: bool) -> Result<(), CompatError> {
        let bubble = self.metadata(id)?.new_bubble_id;
        self.metadata_mut(id)?.is_interactive = enabled;
        if let Some(tracking) = &mut self.original_bubbles {
            tracking.interactive(id, bubble, enabled);
        }
        Ok(())
    }

    fn metadata_mut(&mut self, ball_id: i64) -> Result<Mut<'_, DestinyBallMetadata>, CompatError> {
        let entity = self.entity(ball_id)?;
        self.app
            .world_mut()
            .get_mut::<DestinyBallMetadata>(entity)
            .ok_or_else(|| CompatError::Engine("missing DestinyBallMetadata".into()))
    }

    fn formation_slot_count(&self, ball_id: i64) -> Result<Option<usize>, CompatError> {
        let id = self.metadata(ball_id)?.formation_id;
        Ok(if id == NO_FORMATION {
            None
        } else {
            self.park.formations.get(usize::from(id)).map(Vec::len)
        })
    }

    fn set_position(&mut self, ball_id: i64, position: DVec3) -> Result<(), CompatError> {
        if !solver_safe_vec3(position, MAX_COORDINATE) {
            return Err(CompatError::InvalidRequest(
                "position exceeds the solver-safe coordinate limit".into(),
            ));
        }
        let entity = self.entity(ball_id)?;
        let world = self.app.world_mut();
        if world.get::<Position>(entity).is_none() || world.get::<Transform>(entity).is_none() {
            return Err(CompatError::Engine(
                "ball is missing Position or Transform".into(),
            ));
        }
        world
            .get_mut::<Position>(entity)
            .ok_or_else(|| CompatError::Engine("missing Position".into()))?
            .0 = position;
        world
            .get_mut::<Transform>(entity)
            .ok_or_else(|| CompatError::Engine("missing Transform".into()))?
            .translation = visual_vec3(position);
        Ok(())
    }

    fn set_velocity(&mut self, ball_id: i64, velocity: DVec3) -> Result<(), CompatError> {
        if !solver_safe_vec3(velocity, MAX_VELOCITY) {
            return Err(CompatError::InvalidRequest(
                "velocity exceeds the solver-safe limit".into(),
            ));
        }
        let entity = self.entity(ball_id)?;
        let planned_rotation = if self.park.use_dynamical_orientation {
            None
        } else {
            stable_vec3_normalize(velocity)
                .map(|direction| normalized_quat(DQuat::from_rotation_arc(DVec3::X, direction)))
                .transpose()?
        };
        let world = self.app.world_mut();
        if world.get::<LinearVelocity>(entity).is_none()
            || (planned_rotation.is_some()
                && (world.get::<Rotation>(entity).is_none()
                    || world.get::<Transform>(entity).is_none()
                    || world.get::<AngularVelocity>(entity).is_none()))
        {
            return Err(CompatError::Engine(
                "ball is missing velocity/orientation components".into(),
            ));
        }
        if let Some(rotation) = planned_rotation {
            let mut transform = *world
                .get::<Transform>(entity)
                .ok_or_else(|| CompatError::Engine("missing Transform".into()))?;
            transform.rotation = rotation.as_quat();
            world.entity_mut(entity).insert((
                LinearVelocity(velocity),
                Rotation(rotation),
                AngularVelocity(DVec3::ZERO),
                transform,
            ));
        } else {
            world
                .get_mut::<LinearVelocity>(entity)
                .ok_or_else(|| CompatError::Engine("missing LinearVelocity".into()))?
                .0 = velocity;
        }
        Ok(())
    }

    fn set_angular_velocity(&mut self, ball_id: i64, velocity: DVec3) -> Result<(), CompatError> {
        if !solver_safe_vec3(velocity, MAX_VELOCITY) {
            return Err(CompatError::InvalidRequest(
                "angular velocity exceeds the solver-safe limit".into(),
            ));
        }
        let entity = self.entity(ball_id)?;
        self.app
            .world_mut()
            .get_mut::<AngularVelocity>(entity)
            .ok_or_else(|| CompatError::Engine("missing AngularVelocity".into()))?
            .0 = velocity;
        Ok(())
    }

    fn set_rotation(&mut self, ball_id: i64, rotation: DQuat) -> Result<(), CompatError> {
        let rotation = normalized_quat(rotation)?;
        let entity = self.entity(ball_id)?;
        let world = self.app.world_mut();
        if world.get::<Rotation>(entity).is_none()
            || world.get::<Transform>(entity).is_none()
            || world.get::<AngularVelocity>(entity).is_none()
        {
            return Err(CompatError::Engine(
                "ball is missing Rotation, Transform, or AngularVelocity".into(),
            ));
        }
        world
            .get_mut::<Rotation>(entity)
            .ok_or_else(|| CompatError::Engine("missing Rotation".into()))?
            .0 = rotation;
        world
            .get_mut::<Transform>(entity)
            .ok_or_else(|| CompatError::Engine("missing Transform".into()))?
            .rotation = rotation.as_quat();
        world
            .get_mut::<AngularVelocity>(entity)
            .ok_or_else(|| CompatError::Engine("missing AngularVelocity".into()))?
            .0 = DVec3::ZERO;
        Ok(())
    }

    fn set_mass(&mut self, ball_id: i64, mass: f64) -> Result<(), CompatError> {
        if !mass.is_finite() || !(0.0..=MAX_MASS).contains(&mass) {
            return Err(CompatError::InvalidRequest(
                "mass must be finite and within the solver-safe range".into(),
            ));
        }
        let entity = self.entity(ball_id)?;
        let world = self.app.world_mut();
        if world.get::<DestinyMass>(entity).is_none() || world.get::<Mass>(entity).is_none() {
            return Err(CompatError::Engine(
                "ball is missing DestinyMass or Mass".into(),
            ));
        }
        world
            .get_mut::<DestinyMass>(entity)
            .ok_or_else(|| CompatError::Engine("missing DestinyMass".into()))?
            .0 = mass;
        world
            .get_mut::<Mass>(entity)
            .ok_or_else(|| CompatError::Engine("missing Mass".into()))?
            .0 = avian_mass(mass);
        Ok(())
    }

    fn set_radius(&mut self, ball_id: i64, radius: f64) -> Result<(), CompatError> {
        if !radius.is_finite() || !(0.0..=MAX_RADIUS).contains(&radius) {
            return Err(CompatError::InvalidRequest(
                "radius must be finite and within the solver-safe range".into(),
            ));
        }
        let previous = self.metadata(ball_id)?.radius;
        self.metadata_mut(ball_id)?.radius = radius;
        if let Err(error) = self.rebuild_root_collider(ball_id) {
            self.metadata_mut(ball_id)?.radius = previous;
            return Err(error);
        }
        Ok(())
    }

    fn allocate_mini_identity(&mut self) -> Result<i64, CompatError> {
        let (id, local, high) = crate::collision_identity::allocate_collision_id(
            self.park.is_master,
            -1,
            self.park.local_counter,
            self.park.local_high_counter,
        )
        .ok_or_else(|| {
            CompatError::InvalidRequest("mini collision identity counter exhausted".into())
        })?;
        self.park.local_counter = local;
        self.park.local_high_counter = high;
        Ok(id)
    }

    fn register_fixed_mini_objects(&mut self, parent_id: i64) -> Result<(), CompatError> {
        let metadata = self.metadata(parent_id)?.clone();
        if metadata.is_free {
            return Ok(());
        }
        let position = self.position(parent_id)?;
        let mass = self
            .app
            .world()
            .get::<DestinyMass>(self.entity(parent_id)?)
            .ok_or_else(|| CompatError::Engine("mini parent is missing mass".into()))?
            .0;
        for (index, mini) in metadata.minis.iter().enumerate() {
            let mut id = self
                .metadata(parent_id)?
                .mini_ids
                .get(index)
                .copied()
                .unwrap_or(0);
            if id == 0 {
                id = self.allocate_mini_identity()?;
            }
            if mini.get("kind").and_then(Value::as_str) == Some("sphere") {
                let local = value_vec3(mini.get("position"), "mini ball position")?;
                let absolute = position + local;
                let radius = value_f64(mini.get("radius"), "mini ball radius")?;
                if !self.balls.contains_key(&id) {
                    let args = vec![
                        json!(id),
                        json!(mass),
                        json!(radius),
                        json!(0.0),
                        json!(false),
                        json!(false),
                        json!(true),
                        json!(false),
                        json!(false),
                        json!(absolute.x),
                        json!(absolute.y),
                        json!(absolute.z),
                        json!(0.0),
                        json!(0.0),
                        json!(0.0),
                        json!(0.0),
                        json!(0.0),
                    ];
                    // AddMiniball in source bypasses the public AddBall evolve
                    // guard. This private identity path is also used by a
                    // prevalidated restore; public callers never supply it.
                    self.add_ball_with_identity(&args, Some(id))?;
                    let control = MotionControl {
                        mode: OriginalMode::Miniball,
                        ..MotionControl::default()
                    };
                    self.motion.insert(id, control);
                }
                self.set_position(id, absolute)?;
                let mut child = self.metadata_mut(id)?;
                child.owner_id = parent_id;
                child.agility = 0.0;
            }
            let mut parent = self.metadata_mut(parent_id)?;
            if parent.mini_ids.len() < parent.minis.len() {
                let size = parent.minis.len();
                parent.mini_ids.resize(size, 0);
            }
            parent.mini_ids[index] = id;
        }
        Ok(())
    }

    /// Rebuilds the one authoritative Avian collider for a ball. Destiny mini
    /// shapes are represented as an f64 compound collider instead of Bevy
    /// transform children, whose local transforms are limited to f32.
    fn rebuild_root_collider(&mut self, ball_id: i64) -> Result<(), CompatError> {
        let entity = self.entity(ball_id)?;
        let metadata = self.metadata(ball_id)?.clone();
        let collider = root_collider_for_metadata(&metadata)?;
        self.app.world_mut().entity_mut(entity).insert(collider);
        Ok(())
    }

    fn set_max_velocity(&mut self, ball_id: i64, speed: f64) -> Result<(), CompatError> {
        if !speed.is_finite() || !(0.0..=MAX_VELOCITY).contains(&speed) {
            return Err(CompatError::InvalidRequest(
                "max speed must be finite and within the solver-safe range".into(),
            ));
        }
        let entity = self.entity(ball_id)?;
        self.app
            .world_mut()
            .get_mut::<MaxLinearSpeed>(entity)
            .ok_or_else(|| CompatError::Engine("missing MaxLinearSpeed".into()))?
            .0 = speed;
        Ok(())
    }

    fn set_max_angular_velocity(&mut self, ball_id: i64, speed: f64) -> Result<(), CompatError> {
        if !speed.is_finite() || !(0.0..=MAX_VELOCITY).contains(&speed) {
            return Err(CompatError::InvalidRequest(
                "max angular speed must be finite and within the solver-safe range".into(),
            ));
        }
        let entity = self.entity(ball_id)?;
        self.app
            .world_mut()
            .get_mut::<MaxAngularSpeed>(entity)
            .ok_or_else(|| CompatError::Engine("missing MaxAngularSpeed".into()))?
            .0 = speed;
        Ok(())
    }

    fn set_free(&mut self, ball_id: i64, is_free: bool) -> Result<(), CompatError> {
        let was_free = self.metadata(ball_id)?.is_free;
        let entity = self.entity(ball_id)?;
        let staged_collider = if was_free == is_free {
            None
        } else {
            let mut staged = self.metadata(ball_id)?.clone();
            staged.is_free = is_free;
            Some(root_collider_for_metadata(&staged)?)
        };
        if !is_free {
            let world = self.app.world();
            if world.get::<LinearVelocity>(entity).is_none()
                || world.get::<AngularVelocity>(entity).is_none()
            {
                return Err(CompatError::Engine(
                    "ball is missing LinearVelocity or AngularVelocity".into(),
                ));
            }
        }
        self.app.world_mut().entity_mut(entity).insert(if is_free {
            RigidBody::Dynamic
        } else {
            RigidBody::Static
        });
        if let Some(collider) = staged_collider {
            self.app.world_mut().entity_mut(entity).insert(collider);
        }
        self.metadata_mut(ball_id)?.is_free = is_free;
        if was_free != is_free {
            if is_free {
                let children = self.metadata(ball_id)?.mini_ids.clone();
                for child in children {
                    if self.balls.contains_key(&child)
                        && self.metadata(child)?.owner_id == ball_id
                        && self
                            .motion
                            .get(&child)
                            .is_some_and(|control| control.mode == OriginalMode::Miniball)
                    {
                        self.remove_ball_internal(child, true)?;
                    }
                }
            } else {
                self.metadata_mut(ball_id)?.mini_ids.clear();
                self.register_fixed_mini_objects(ball_id)?;
                self.rebuild_root_collider(ball_id)?;
            }
        }
        if !is_free {
            self.set_velocity(ball_id, DVec3::ZERO)?;
            self.set_angular_velocity(ball_id, DVec3::ZERO)?;
        }
        Ok(())
    }

    fn set_massive(&mut self, ball_id: i64, is_massive: bool) -> Result<(), CompatError> {
        let is_cloaked = self.metadata(ball_id)?.is_cloaked;
        if is_massive && is_cloaked != 0 {
            return Err(CompatError::InvalidRequest(
                "a cloaked ball cannot be made massive".into(),
            ));
        }
        let entity = self.entity(ball_id)?;
        if is_massive {
            self.app
                .world_mut()
                .entity_mut(entity)
                .remove::<ColliderDisabled>();
        } else {
            self.app
                .world_mut()
                .entity_mut(entity)
                .insert(ColliderDisabled);
        }
        self.metadata_mut(ball_id)?.is_massive = is_massive;
        Ok(())
    }

    fn cloak_ball(
        &mut self,
        ball_id: i64,
        mode: i32,
        uncloak_range: Option<f64>,
    ) -> Result<(), CompatError> {
        if !(1..=3).contains(&mode) {
            return Err(CompatError::InvalidRequest(
                "cloak mode must be 1, 2, or 3".into(),
            ));
        }
        self.entity(ball_id)?;
        let sensor_range = if self.park.is_master && mode == 1 {
            let range = uncloak_range.unwrap_or(2000.0);
            if !range.is_finite() || range <= 0.0 {
                return Err(CompatError::InvalidRequest(
                    "uncloak range must be positive".into(),
                ));
            }
            if !(self.metadata(ball_id)?.radius + range).is_finite() {
                return Err(CompatError::InvalidRequest(
                    "uncloak range plus ball radius overflowed".into(),
                ));
            }
            let non_cloak_sensors = self
                .metadata(ball_id)?
                .sensors
                .iter()
                .filter(|sensor| {
                    !sensor
                        .get("cloak_sensor")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .count();
            if self.metadata(ball_id)?.minis.len() + non_cloak_sensors + 1
                > self.park.limits.max_child_shapes_per_ball
            {
                return Err(CompatError::InvalidRequest("sensor limit exceeded".into()));
            }
            Some(range)
        } else {
            None
        };
        let cloak_descriptor = sensor_range.map(|range| {
            json!({
                "range": range,
                "period": 2.0,
                "shuffle": 0,
                "only_interactives": false,
                "elapsed": 0.0,
                "members": [],
                "cloak_sensor": true,
            })
        });
        if let Some(descriptor) = &cloak_descriptor {
            validate_sensor_descriptor(descriptor)?;
            self.ensure_child_descriptor_budget(ball_id, descriptor, false, true)?;
        }
        let was_cloaked = self.metadata(ball_id)?.is_cloaked != 0;
        if !was_cloaked {
            let was_massive = self.metadata(ball_id)?.is_massive;
            self.metadata_mut(ball_id)?.massive_before_cloak = Some(was_massive);
        }
        {
            let mut metadata = self.metadata_mut(ball_id)?;
            metadata.sensors.retain(|sensor| {
                !sensor
                    .get("cloak_sensor")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            });
            metadata.is_cloaked = mode;
        }
        self.set_massive(ball_id, false)?;
        if let Some(descriptor) = cloak_descriptor {
            self.metadata_mut(ball_id)?.sensors.push(descriptor);
        }
        Ok(())
    }

    fn uncloak_ball(&mut self, ball_id: i64) -> Result<(), CompatError> {
        self.entity(ball_id)?;
        {
            let mut metadata = self.metadata_mut(ball_id)?;
            metadata.is_cloaked = 0;
            metadata.sensors.retain(|sensor| {
                !sensor
                    .get("cloak_sensor")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            });
            metadata.massive_before_cloak = None;
        }

        // Original Destiny's Ballpark::UncloakBall always makes a ball
        // massive when it is not in proper warp, even when the ball was
        // already uncloaked. This compatibility subset does not implement
        // Warp, so every representable state is a non-warp state.
        self.set_massive(ball_id, true)?;
        self.remove_from_sensor_members(ball_id)?;
        Ok(())
    }

    fn check_visibility(&self, source_id: i64, destination_id: i64) -> Result<i64, CompatError> {
        if self.park.pending_removals.contains_key(&source_id)
            || self.park.pending_removals.contains_key(&destination_id)
        {
            return Ok(-2);
        }
        let (source, destination) =
            match (self.balls.get(&source_id), self.balls.get(&destination_id)) {
                (Some(_), Some(_)) => (self.position(source_id)?, self.position(destination_id)?),
                _ => return Ok(-2),
            };
        let source_bubble = self.metadata(source_id)?.new_bubble_id;
        let destination_metadata = self.metadata(destination_id)?;
        if destination_metadata.is_cloaked != 0
            || (!destination_metadata.is_global
                && destination_metadata.new_bubble_id != source_bubble)
        {
            return Ok(-2);
        }
        let segment = destination - source;
        if !segment.is_finite() {
            return Ok(-2);
        }
        let segment_length = stable_vec3_length(segment);
        if !segment_length.is_finite() {
            return Ok(-2);
        }
        if segment_length == 0.0 {
            return Ok(0);
        }
        let direction = if segment_length == 0.0 {
            DVec3::ZERO
        } else {
            segment / segment_length
        };
        let mut blockers = Vec::new();
        for id in self.balls.keys().copied() {
            if id == source_id || id == destination_id {
                continue;
            }
            if self.park.pending_removals.contains_key(&id) {
                continue;
            }
            let metadata = self.metadata(id)?;
            if !metadata.is_massive
                || metadata.is_cloaked != 0
                || (!metadata.is_global && metadata.new_bubble_id != source_bubble)
            {
                continue;
            }
            let offset = self.position(id)? - source;
            if !offset.is_finite() {
                continue;
            }
            let distance_along = offset.dot(direction);
            let parameter = distance_along / segment_length;
            if !(0.0..1.0).contains(&parameter) {
                continue;
            }
            let closest = source + direction * distance_along;
            if stable_vec3_length(self.position(id)? - closest) <= metadata.radius {
                blockers.push((parameter, id));
            }
        }
        blockers.sort_by(|left, right| left.0.total_cmp(&right.0).then(left.1.cmp(&right.1)));
        Ok(blockers.first().map_or(0, |(_, id)| *id))
    }

    fn add_proximity_sensor(&mut self, ball_id: i64, args: &[Value]) -> Result<Value, CompatError> {
        if !(1..=4).contains(&args.len()) {
            return Err(CompatError::InvalidRequest(
                "AddProximitySensor expects 1 to 4 arguments".into(),
            ));
        }
        let range = value_f64(args.first(), "range")?;
        let owner_radius = self.metadata(ball_id)?.radius;
        let sensor_reach = owner_radius + range;
        if !sensor_reach.is_finite() {
            return Err(CompatError::InvalidRequest(
                "sensor range plus ball radius overflowed".into(),
            ));
        }
        if sensor_reach < 0.0 {
            return Ok(json!(-1));
        }
        let period = args
            .get(1)
            .map_or(Ok(2.0), |value| value_f64(Some(value), "period"))?;
        let shuffle = args
            .get(2)
            .map_or(Ok(0), |value| value_i64(Some(value), "shuffle"))?;
        let only_interactives = args.get(3).map_or(Ok(false), |value| {
            value_bool(Some(value), "onlyInteractives")
        })?;
        let existing_sensor_count = self.metadata(ball_id)?.sensors.len();
        let has_user_sensor = self.metadata(ball_id)?.sensors.iter().any(|sensor| {
            !sensor
                .get("cloak_sensor")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        });
        if self.metadata(ball_id)?.minis.len()
            + existing_sensor_count
            + usize::from(!has_user_sensor)
            > self.park.limits.max_child_shapes_per_ball
        {
            return Err(CompatError::InvalidRequest("sensor limit exceeded".into()));
        }
        let elapsed = if shuffle == 0 {
            // The original replacement clears members but retains elapsed
            // unless shuffle explicitly chooses a new initial phase.
            self.metadata(ball_id)?
                .sensors
                .iter()
                .find(|sensor| {
                    !sensor
                        .get("cloak_sensor")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .and_then(|sensor| sensor.get("elapsed"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        } else {
            deterministic_sensor_phase(ball_id, period)
        };
        let descriptor = json!({
            "range": range,
            "period": period,
            "shuffle": shuffle,
            "only_interactives": only_interactives,
            "elapsed": elapsed,
            "members": [],
            "cloak_sensor": false,
        });
        validate_sensor_descriptor(&descriptor)?;
        self.ensure_child_descriptor_budget(ball_id, &descriptor, has_user_sensor, false)?;
        let mut metadata = self.metadata_mut(ball_id)?;
        if let Some(user_sensor) = metadata.sensors.iter_mut().find(|sensor| {
            !sensor
                .get("cloak_sensor")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        }) {
            *user_sensor = descriptor;
        } else {
            metadata.sensors.insert(0, descriptor);
        }
        Ok(json!(0))
    }

    fn remove_from_sensor_members(&mut self, ball_id: i64) -> Result<(), CompatError> {
        let owner_ids = self.balls.keys().copied().collect::<Vec<_>>();
        let mut staged = Vec::with_capacity(owner_ids.len());
        for owner_id in owner_ids {
            let mut metadata = self.metadata(owner_id)?.clone();
            for sensor in &mut metadata.sensors {
                let Some(object) = sensor.as_object_mut() else {
                    continue;
                };
                let Some(members) = object.get_mut("members").and_then(Value::as_array_mut) else {
                    continue;
                };
                members.retain(|member| member.as_i64() != Some(ball_id));
            }
            staged.push((self.entity(owner_id)?, metadata));
        }
        let world = self.app.world_mut();
        for (entity, metadata) in staged {
            world.entity_mut(entity).insert(metadata);
        }
        Ok(())
    }

    fn dispatch_park(
        &mut self,
        title: &str,
        operation: &str,
        args: &[Value],
    ) -> Result<Value, CompatError> {
        let name = title.rsplit('.').next().unwrap_or(title);
        if operation == "get" {
            require_arity(args, &[0], title)?;
            return match name {
                "isRunning" => Ok(json!(self.is_running())),
                "tickInterval" => json_number(
                    self.app
                        .world()
                        .resource::<Time<Fixed>>()
                        .timestep()
                        .as_secs_f64()
                        * 1000.0,
                ),
                "friction" => json_number(self.app.world().resource::<DestinySpaceFriction>().0),
                "currentTime" => Ok(json!(self.park.current_time)),
                "time" => Ok(json!(self.park.time)),
                "isMaster" => Ok(json!(self.park.is_master)),
                "ego" => Ok(json!(self.park.ego)),
                _ => Err(CompatError::UnsupportedTitle(title.to_owned())),
            };
        }
        if operation == "set" {
            require_arity(args, &[1], title)?;
            match name {
                "tickInterval" => {
                    let milliseconds = value_f64(args.first(), "tickInterval")?;
                    let duration = tick_duration(milliseconds)?;
                    self.app
                        .world_mut()
                        .resource_mut::<Time<Fixed>>()
                        .set_timestep(duration);
                }
                "friction" => {
                    let coefficient = value_f64(args.first(), "friction")?;
                    if coefficient < 0.0 {
                        return Err(CompatError::InvalidRequest(
                            "friction must be non-negative".into(),
                        ));
                    }
                    self.app
                        .world_mut()
                        .resource_mut::<DestinySpaceFriction>()
                        .0 = coefficient;
                }
                "time" => self.park.time = value_i64(args.first(), name)?,
                "ego" => self.park.ego = value_i64(args.first(), "ego")?,
                "isMaster" => self.park.is_master = value_bool(args.first(), "isMaster")?,
                "isRunning" | "currentTime" => {
                    return Err(CompatError::InvalidRequest(format!(
                        "destiny.Ballpark.{name} is read-only"
                    )));
                }
                _ => return Err(CompatError::UnsupportedTitle(title.to_owned())),
            }
            return Ok(Value::Null);
        }
        if operation != "call" {
            return Err(CompatError::InvalidRequest(format!(
                "{title} does not support operation {operation:?}"
            )));
        }
        if name != "AddBall"
            && let Some(allowed) = park_call_arities(name)
        {
            require_arity(args, allowed, title)?;
        }

        match name {
            "AddBall" => Ok(json!(self.add_ball(args)?)),
            "LoadFormations" => {
                let rows = args.first().and_then(Value::as_array).ok_or_else(|| {
                    CompatError::InvalidRequest("formations must be an array".into())
                })?;
                let formations = rows
                    .iter()
                    .map(|row| {
                        let row = row.as_array().filter(|row| row.len() == 2).ok_or_else(|| {
                            CompatError::InvalidRequest(
                                "each formation must contain a name and offsets".into(),
                            )
                        })?;
                        let offsets = row[1].as_array().ok_or_else(|| {
                            CompatError::InvalidRequest("formation offsets must be an array".into())
                        })?;
                        offsets
                            .iter()
                            .map(|offset| {
                                let offset = value_vec3(Some(offset), "formation offset")?;
                                Ok([offset.x, offset.y, offset.z])
                            })
                            .collect::<Result<Vec<_>, CompatError>>()
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                validate_formations(&formations)?;
                self.park.formations = formations;
                Ok(Value::Null)
            }
            "SetBallFormation" => {
                let ball_id = value_i64(args.first(), "ball_id")?;
                let requested = value_i64(args.get(1), "formation_id")?;
                if self.balls.contains_key(&ball_id) {
                    let count = self.park.formations.len();
                    let old = self.metadata(ball_id)?.formation_id;
                    let (formation, _) = assign_formation(
                        old,
                        self.metadata(ball_id)?.formation_slots,
                        requested,
                        count,
                    );
                    if formation == old {
                        return Ok(Value::Null);
                    }
                    let mut remaining = VecDeque::new();
                    if old != NO_FORMATION {
                        let old_length = self
                            .park
                            .formations
                            .get(usize::from(old))
                            .map(Vec::len)
                            .unwrap_or(0);
                        let new_length = self
                            .park
                            .formations
                            .get(usize::from(formation))
                            .map(Vec::len)
                            .unwrap_or(0);
                        for follower in self.followers(ball_id) {
                            if self.balls.contains_key(&follower)
                                && (formation == NO_FORMATION
                                    || crate::formation::oversized_follower(
                                        old_length,
                                        new_length,
                                        mode_number(
                                            self.motion
                                                .get(&follower)
                                                .map_or(OriginalMode::Stop, |control| control.mode),
                                        ),
                                        self.metadata(follower)?.effect_stamp,
                                    )
                                    .ok_or_else(|| {
                                        CompatError::InvalidRequest(
                                            "formation count exceeds C int representation".into(),
                                        )
                                    })?)
                            {
                                remaining.push_back((follower, self.ball_generations[&follower]));
                            }
                        }
                    }
                    self.mode_callbacks
                        .push_back(CallCallback::FormationChange {
                            id: ball_id,
                            generation: self.ball_generations[&ball_id],
                            formation,
                            remaining,
                        });
                }
                Ok(Value::Null)
            }
            "ClearAll" => {
                if self.evolution_phase != 0 {
                    return Err(CompatError::InvalidRequest(
                        "Can not clear ballpark during Evolve().".into(),
                    ));
                }
                self.clear_all()?;
                Ok(Value::Null)
            }
            "RemoveBall" => {
                let ball_id = value_i64(args.first(), "ball_id")?;
                let delay = args
                    .get(1)
                    .map_or(Ok(0), |value| value_i64(Some(value), "delay"))?;
                self.schedule_remove_ball(ball_id, delay)?;
                Ok(Value::Null)
            }
            "InitializeBubbles" => {
                require_arity(args, &[0, 1], title)?;
                let source = args
                    .first()
                    .map_or(Ok(0), |value| value_i64(Some(value), "sourceId"))?;
                if source != 0 {
                    self.update_source_ball_bubble(source)?;
                } else {
                    self.initialize_source_bubbles()?;
                }
                Ok(Value::Null)
            }
            "CopyBubbles" => {
                require_arity(args, &[0], title)?;
                if let Some(tracking) = &mut self.original_bubbles {
                    tracking.copy();
                }
                Ok(Value::Null)
            }
            "GetBubbleAtCoordinates" => {
                require_arity(args, &[3], title)?;
                let point = OriginalVec3 {
                    x: value_f64(args.first(), "x")?,
                    y: value_f64(args.get(1), "y")?,
                    z: value_f64(args.get(2), "z")?,
                };
                match &self.original_bubbles {
                    Some(tracking) => {
                        Ok(json!(tracking.bubble_at(point).map_err(|error| {
                            CompatError::InvalidRequest(error.into())
                        })?))
                    }
                    None if self.balls.is_empty() => Ok(Value::Null),
                    None => Err(CompatError::InvalidRequest(
                        "source uninitialized partition query remains unimplemented".into(),
                    )),
                }
            }
            "AdditionsAndDeletions" => {
                require_arity(args, &[1, 2], title)?;
                let users = args[0].as_array().ok_or_else(|| {
                    CompatError::InvalidRequest("userShips must be a list".into())
                })?;
                let mut aliases = [0usize, 1, 2, 3];
                if let Some(value) = args.get(1) {
                    let values = value
                        .as_array()
                        .filter(|values| values.len() == 4)
                        .ok_or_else(|| {
                            CompatError::InvalidRequest("four dictionary aliases required".into())
                        })?;
                    for (index, value) in values.iter().enumerate() {
                        aliases[index] = value
                            .as_u64()
                            .filter(|value| *value <= index as u64)
                            .ok_or_else(|| {
                                CompatError::InvalidRequest("invalid dictionary alias index".into())
                            })? as usize;
                    }
                }
                self.source_bubble_updates(users, aliases)
            }
            "AddMushroom" => {
                require_arity(args, &[3], title)?;
                let owner = value_i64(args.first(), "ownerId")?;
                let range = value_f64(args.get(1), "range")? as f32;
                let duration = value_f64(args.get(2), "time")?;
                if range <= 0.0 || duration <= 0.0 || !self.balls.contains_key(&owner) {
                    return Ok(Value::Null);
                }
                if !range.is_finite() || f64::from(range) > MAX_RADIUS {
                    return Err(CompatError::InvalidRequest(
                        "mushroom range exceeds solver envelope".into(),
                    ));
                }
                let position = self.position(owner)?;
                let initial = crate::mushroom::initial_ball(OriginalVec3 {
                    x: position.x,
                    y: position.y,
                    z: position.z,
                });
                let mut parameters = vec![
                    json!(initial.id),
                    json!(initial.mass),
                    json!(initial.radius),
                    json!(initial.max_velocity),
                    json!(initial.is_free),
                    json!(initial.is_global),
                    json!(initial.is_massive),
                    json!(initial.is_interactive),
                    json!(initial.is_space_junk),
                    json!(position.x),
                    json!(position.y),
                    json!(position.z),
                    json!(0.0),
                    json!(0.0),
                    json!(0.0),
                    json!(1.0),
                    json!(1.0),
                ];
                if self.park.use_dynamical_orientation {
                    parameters.extend([json!(0.0), json!(1.0)]);
                }
                let handle = self.add_ball(&parameters)?;
                let id = parse_ball_target(Some(&handle))?;
                self.motion.insert(
                    id,
                    MotionControl {
                        mode: OriginalMode::Stop,
                        destination: OriginalVec3 {
                            x: duration,
                            y: 0.0,
                            z: 0.0,
                        },
                        follow_range: range,
                        ..MotionControl::default()
                    },
                );
                let started = self.park.current_time;
                let mut metadata = self.metadata_mut(id)?;
                metadata.owner_id = owner;
                metadata.effect_stamp = started;
                self.queue_mode_change(id, OriginalMode::Mushroom);
                Ok(Value::Null)
            }
            "Pause" => {
                self.app.world_mut().resource_mut::<Time<Physics>>().pause();
                Ok(Value::Null)
            }
            "Start" => {
                self.app
                    .world_mut()
                    .resource_mut::<Time<Physics>>()
                    .unpause();
                Ok(Value::Null)
            }
            "Evolve" => {
                self.begin_callback_tick(true)?;
                Ok(Value::Null)
            }
            "WarpTo" => {
                require_arity(args, &[4, 5, 6], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                let point = OriginalVec3 {
                    x: value_f64(args.get(1), "x")?,
                    y: value_f64(args.get(2), "y")?,
                    z: value_f64(args.get(3), "z")?,
                };
                let min_range = args
                    .get(4)
                    .map(|v| value_f64(Some(v), "minRange"))
                    .transpose()?
                    .unwrap_or(20_000.0);
                let min_range = if min_range < 0.0 { 0.0 } else { min_range };
                let factor = args
                    .get(5)
                    .map(|v| value_i64(Some(v), "warpFactor"))
                    .transpose()?
                    .unwrap_or(20)
                    .max(1);
                let factor = i32::try_from(factor)
                    .map_err(|_| CompatError::InvalidRequest("warpFactor exceeds i32".into()))?;
                if !self.balls.contains_key(&id) {
                    return Ok(Value::Null);
                }
                self.begin_warp(id, point, min_range, factor)?;
                Ok(Value::Null)
            }
            "EntityWarpIn" => {
                require_arity(args, &[5], title)?;
                let id = value_i64(args.first(), "srcId")?;
                let point = OriginalVec3 {
                    x: value_f64(args.get(1), "x")?,
                    y: value_f64(args.get(2), "y")?,
                    z: value_f64(args.get(3), "z")?,
                };
                let factor = i32::try_from(value_i64(args.get(4), "warpFactor")?)
                    .map_err(|_| CompatError::InvalidRequest("warpFactor exceeds C int".into()))?
                    .max(1);
                if self.balls.contains_key(&id) {
                    self.begin_warp(id, point, 0.0, factor)?;
                    self.mode_callbacks.push_back(CallCallback::EntityArrival {
                        id,
                        generation: self.ball_generations[&id],
                        point,
                    });
                }
                Ok(Value::Null)
            }
            "GotoPoint" | "GotoDirection" => {
                require_arity(args, &[4], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                let point = OriginalVec3 {
                    x: value_f64(args.get(1), "x")?,
                    y: value_f64(args.get(2), "y")?,
                    z: value_f64(args.get(3), "z")?,
                };
                if !self.balls.contains_key(&id) {
                    return Ok(Value::Null);
                }
                if name == "GotoDirection" {
                    self.begin_goto_direction(id, point)?;
                } else {
                    self.begin_goto_point(id, point)?;
                }
                Ok(Value::Null)
            }
            "SetNotificationRange" => {
                require_arity(args, &[2], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                let range = value_f64(args.get(1), "notificationRange")?;
                if range >= 0.0 && self.balls.contains_key(&id) {
                    let state = self.range_notifications.entry(id).or_default();
                    state.range = range;
                    state.within = false;
                }
                Ok(Value::Null)
            }
            "SetTargetTracking" => {
                require_arity(args, &[3], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                let target = value_i64(args.get(1), "target_id")?;
                let range = value_f64(args.get(2), "trackingRange")?;
                if self.balls.contains_key(&id) {
                    if range <= 0.0 || target == 0 {
                        let state = self.range_notifications.entry(id).or_default();
                        state.target = None;
                        state.tracking_range = 0.0;
                    } else if self.balls.contains_key(&target) {
                        let state = self.range_notifications.entry(id).or_default();
                        state.target = Some(target);
                        state.tracking_range = range;
                        state.tracking_within = false;
                    }
                }
                Ok(Value::Null)
            }
            "FollowBall" | "Orbit" => {
                require_arity(args, &[2, 3], title)?;
                let id = value_i64(args.first(), "ball_id")?;
                let target = value_i64(args.get(1), "target_id")?;
                let range = args
                    .get(2)
                    .map(|v| value_source_float(Some(v), "range"))
                    .transpose()?
                    .unwrap_or(1.0) as f32;
                // The original method receives float after the Python cast;
                // nonfinite float range is ignored by FollowBall/Orbit.
                if !range.is_finite()
                    || id == target
                    || !self.balls.contains_key(&id)
                    || !self.balls.contains_key(&target)
                    || self
                        .ball_generations
                        .get(&target)
                        .is_some_and(|generation| self.retained_balls[generation].moribund)
                {
                    return Ok(Value::Null);
                }
                let owner = self.metadata(id)?;
                let other = self.metadata(target)?;
                if owner.new_bubble_id != other.new_bubble_id || other.is_cloaked != 0 {
                    return Ok(Value::Null);
                }
                self.begin_follow(
                    id,
                    target,
                    if name == "Orbit" {
                        FollowKind::Orbit(range)
                    } else {
                        FollowKind::Follow(range)
                    },
                )?;
                Ok(Value::Null)
            }
            "MissileFollow" => {
                require_arity(args, &[3], title)?;
                let id = value_i64(args.first(), "srcId")?;
                let target = value_i64(args.get(1), "dstId")?;
                let owner = value_i64(args.get(2), "ownerId")?;
                let moribund = self
                    .ball_generations
                    .get(&target)
                    .is_some_and(|generation| self.retained_balls[generation].moribund);
                if crate::missile::admitted(
                    id,
                    target,
                    owner,
                    self.balls.contains_key(&id),
                    self.balls.contains_key(&target),
                    moribund,
                ) {
                    self.begin_follow(id, target, FollowKind::Missile(owner))?;
                }
                Ok(Value::Null)
            }
            "LaunchMissile" => {
                require_arity(args, &[5], title)?;
                let id = value_i64(args.first(), "srcID")?;
                let target = value_i64(args.get(1), "dstID")?;
                let owner = value_i64(args.get(2), "ownerID")?;
                let aimed = value_u8(args.get(3), "aimedLaunch")?;
                let massive = value_u8(args.get(4), "massive")?;
                if target < 0 {
                    return Err(CompatError::InvalidRequest(
                        "Can not launch missile on a negative ballID".into(),
                    ));
                }
                let owner_check = owner.checked_abs().ok_or_else(|| {
                    CompatError::InvalidRequest("owner absolute ID overflows i64".into())
                })?;
                if !self.balls.contains_key(&owner_check) {
                    return Ok(Value::Null);
                }
                let launcher = self.entity(owner_check)?;
                let position = self.position(owner_check)?;
                let velocity = self
                    .app
                    .world()
                    .get::<LinearVelocity>(launcher)
                    .ok_or_else(|| CompatError::Engine("missing launcher velocity".into()))?
                    .0;
                let maximum = self
                    .app
                    .world()
                    .get::<MaxLinearSpeed>(launcher)
                    .ok_or_else(|| CompatError::Engine("missing launcher max speed".into()))?
                    .0 as f32;
                let rotation = self
                    .app
                    .world()
                    .get::<Rotation>(launcher)
                    .ok_or_else(|| CompatError::Engine("missing launcher rotation".into()))?
                    .0;
                let stationary = rotation * -DVec3::Z;
                let has_target = target != owner && self.balls.contains_key(&target);
                let target_cloaked = has_target && self.metadata(target)?.is_cloaked != 0;
                let offset = if has_target {
                    let delta = self.position(target)? - position;
                    Some(OriginalVec3 {
                        x: delta.x,
                        y: delta.y,
                        z: delta.z,
                    })
                } else {
                    None
                };
                let (initial_velocity, max_velocity) = crate::missile::launch_velocity(
                    OriginalVec3 {
                        x: velocity.x,
                        y: velocity.y,
                        z: velocity.z,
                    },
                    maximum,
                    OriginalVec3 {
                        x: stationary.x,
                        y: stationary.y,
                        z: stationary.z,
                    },
                    offset,
                    aimed,
                    target_cloaked,
                );
                if self.balls.contains_key(&id) {
                    if massive != 0 {
                        self.set_massive(id, true)?;
                    }
                    self.set_position(id, position)?;
                    self.set_velocity(
                        id,
                        DVec3::new(initial_velocity.x, initial_velocity.y, initial_velocity.z),
                    )?;
                    if !target_cloaked {
                        if !has_target {
                            if max_velocity > 0.0 {
                                self.begin_goto_direction(id, initial_velocity)?;
                            }
                        } else {
                            let moribund =
                                self.retained_balls[&self.ball_generations[&target]].moribund;
                            if crate::missile::admitted(id, target, owner, true, true, moribund) {
                                self.begin_follow(id, target, FollowKind::Missile(owner))?;
                            }
                        }
                    }
                }
                Ok(Value::Null)
            }
            "FormationFollow" => {
                require_arity(args, &[3], title)?;
                let id = value_i64(args.first(), "srcId")?;
                let target = value_i64(args.get(1), "dstId")?;
                let slot = i32::try_from(value_i64(args.get(2), "slot")?)
                    .map_err(|_| CompatError::InvalidRequest("slot exceeds C int".into()))?
                    as i8;
                if id != target
                    && self.balls.contains_key(&id)
                    && self.balls.contains_key(&target)
                    && self
                        .formation_slot_count(target)?
                        .is_some_and(|count| i64::from(slot) < count as i64)
                {
                    self.begin_follow(id, target, FollowKind::Formation(slot))?;
                }
                Ok(Value::Null)
            }
            "SetBallHarmonic" => {
                require_arity(args, &[5], title)?;
                let id = value_i64(args.first(), "srcId")?;
                let harmonic = value_i64(args.get(1), "harmonic")?;
                let corporation = value_c_int(args.get(2), "corporationID")?;
                let alliance = value_c_int(args.get(3), "allianceID")?;
                let field = value_c_int(args.get(4), "field")? != 0;
                if self.balls.contains_key(&id) {
                    let generation = self.ball_generations[&id];
                    self.retained_balls
                        .get_mut(&generation)
                        .expect("live identity")
                        .identity_fields = crate::harmonic::fields(harmonic, corporation, alliance);
                    let action = crate::harmonic::action(
                        mode_number(
                            self.motion
                                .get(&id)
                                .map_or(OriginalMode::Stop, |control| control.mode),
                        ),
                        field,
                    );
                    if action == 10 {
                        self.stop_command(id)?;
                        self.mode_callbacks.push_back(CallCallback::SetMode {
                            id,
                            generation,
                            mode: OriginalMode::Field,
                            troll_delay: None,
                        });
                    } else if action == 2 {
                        self.stop_command(id)?;
                    }
                }
                Ok(Value::Null)
            }
            "SetBoid" => {
                require_arity(args, &[2], title)?;
                let id = value_i64(args.first(), "srcId")?;
                let enabled = value_c_int(args.get(1), "turnOn")? != 0;
                if self.balls.contains_key(&id) {
                    if enabled {
                        self.queue_mode_change(id, OriginalMode::Boid);
                    } else {
                        self.stop_command(id)?;
                    }
                }
                Ok(Value::Null)
            }
            "SetBallRigid" | "SetBallTroll" => {
                require_arity(
                    args,
                    if name == "SetBallTroll" { &[2] } else { &[1] },
                    title,
                )?;
                let id = value_i64(args.first(), "srcId")?;
                let troll_delay = if name == "SetBallTroll" {
                    Some(value_c_int(args.get(1), "delay")?)
                } else {
                    None
                };
                if self.balls.contains_key(&id) {
                    self.stop_command(id)?;
                    self.mode_callbacks.push_back(CallCallback::SetMode {
                        id,
                        generation: self.ball_generations[&id],
                        mode: if troll_delay.is_some() {
                            OriginalMode::Troll
                        } else {
                            OriginalMode::Rigid
                        },
                        troll_delay,
                    });
                }
                Ok(Value::Null)
            }
            "AdjustTimes" => {
                self.park.time =
                    adjust_time(self.park.time, value_i64(args.first(), "time_delta")?)
                        .ok_or_else(|| {
                            CompatError::InvalidRequest("time adjustment overflow".into())
                        })?;
                Ok(Value::Null)
            }
            "SetBallPosition" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.set_position(
                    id,
                    DVec3::new(
                        value_f64(args.get(1), "x")?,
                        value_f64(args.get(2), "y")?,
                        value_f64(args.get(3), "z")?,
                    ),
                )?;
                Ok(Value::Null)
            }
            "SetBallVelocity" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.set_velocity(
                    id,
                    DVec3::new(
                        value_f64(args.get(1), "vx")?,
                        value_f64(args.get(2), "vy")?,
                        value_f64(args.get(3), "vz")?,
                    ),
                )?;
                Ok(Value::Null)
            }
            "SetBallAngularVelocity" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.set_angular_velocity(
                    id,
                    DVec3::new(
                        value_f64(args.get(1), "wx")?,
                        value_f64(args.get(2), "wy")?,
                        value_f64(args.get(3), "wz")?,
                    ),
                )?;
                Ok(Value::Null)
            }
            "SetBallRotation" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.set_rotation(
                    id,
                    normalized_quat(DQuat::from_xyzw(
                        value_f64(args.get(1), "rx")?,
                        value_f64(args.get(2), "ry")?,
                        value_f64(args.get(3), "rz")?,
                        value_f64(args.get(4), "rw")?,
                    ))?,
                )?;
                Ok(Value::Null)
            }
            "SetBallMass" => {
                let id = value_i64(args.first(), "ball_id")?;
                let mass = value_f64(args.get(1), "mass")?;
                if positive_setter_accepts(mass) {
                    self.set_mass(id, mass)?;
                }
                Ok(Value::Null)
            }
            "SetBallRadius" => {
                let id = value_i64(args.first(), "ball_id")?;
                let radius = value_f64(args.get(1), "radius")?;
                if non_negative_setter_accepts(radius) {
                    self.set_radius(id, radius)?;
                }
                Ok(Value::Null)
            }
            "SetMaxSpeed" => {
                let id = value_i64(args.first(), "ball_id")?;
                let speed = value_f64(args.get(1), "speed")?;
                if non_negative_setter_accepts(speed) {
                    self.set_max_velocity(id, speed)?;
                }
                Ok(Value::Null)
            }
            "SetMaxAngularSpeed" => {
                let id = value_i64(args.first(), "ball_id")?;
                let speed = value_f64(args.get(1), "speed")?;
                if non_negative_setter_accepts(speed) {
                    self.set_max_angular_velocity(id, speed)?;
                }
                Ok(Value::Null)
            }
            "SetBallFree" => {
                self.set_free(
                    value_i64(args.first(), "ball_id")?,
                    value_bool(args.get(1), "is_free")?,
                )?;
                Ok(Value::Null)
            }
            "SetBallGlobal" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.metadata_mut(id)?.is_global = value_bool(args.get(1), "is_global")?;
                Ok(Value::Null)
            }
            "SetBallMassive" => {
                self.set_massive(
                    value_i64(args.first(), "ball_id")?,
                    value_bool(args.get(1), "is_massive")?,
                )?;
                Ok(Value::Null)
            }
            "SetBallInteractive" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.set_interactive(id, value_bool(args.get(1), "is_interactive")?)?;
                Ok(Value::Null)
            }
            "SetSpeedFraction" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.metadata_mut(id)?.speed_fraction =
                    clamp_speed_fraction(value_f64(args.get(1), "speed_fraction")?);
                Ok(Value::Null)
            }
            "SetBallAgility" => {
                let id = value_i64(args.first(), "ball_id")?;
                let agility = value_f64(args.get(1), "agility")?;
                if positive_setter_accepts(agility) {
                    if agility > MAX_AGILITY {
                        return Err(CompatError::InvalidRequest(
                            "agility exceeds the solver-safe limit".into(),
                        ));
                    }
                    self.metadata_mut(id)?.agility = agility;
                }
                Ok(Value::Null)
            }
            "Stop" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.stop_command(id)?;
                Ok(Value::Null)
            }
            "GetFollowers" => {
                require_arity(args, &[1], title)?;
                let target = value_i64(args.first(), "targetId")?;
                self.entity(target)?;
                Ok(json!(
                    self.followers(target)
                        .into_iter()
                        .filter(|id| self.balls.contains_key(id))
                        .collect::<Vec<_>>()
                ))
            }
            "GetAccuracy" => {
                require_arity(args, &[7], title)?;
                let src = value_i64(args.first(), "srcId")?;
                let dst = value_i64(args.get(1), "dstId")?;
                let mut parameters = [0.0; 5];
                for (index, value) in parameters.iter_mut().enumerate() {
                    *value = value_f64(args.get(index + 2), "accuracy parameter")?;
                }
                let project = |id| -> Result<Option<crate::accuracy::Ball>, CompatError> {
                    if !self.balls.contains_key(&id) {
                        return Ok(None);
                    }
                    let position = self.position(id)?;
                    let velocity = self
                        .app
                        .world()
                        .get::<LinearVelocity>(self.entity(id)?)
                        .ok_or_else(|| CompatError::Engine("missing accuracy velocity".into()))?
                        .0;
                    Ok(Some(crate::accuracy::Ball {
                        position: OriginalVec3 {
                            x: position.x,
                            y: position.y,
                            z: position.z,
                        },
                        velocity: OriginalVec3 {
                            x: velocity.x,
                            y: velocity.y,
                            z: velocity.z,
                        },
                        radius: self.metadata(id)?.radius as f32,
                    }))
                };
                Ok(json!(crate::accuracy::values(
                    src == dst,
                    project(src)?,
                    project(dst)?,
                    parameters
                )))
            }
            "GetRemoteFollowers" => {
                require_arity(args, &[0], title)?;
                let mut result = Vec::new();
                for (&id, control) in &self.motion {
                    if !self.balls.contains_key(&id) {
                        continue;
                    }
                    let target = control
                        .follow_id
                        .and_then(|target| self.balls.get(&target).map(|_| target));
                    let same = match target {
                        Some(target) => {
                            self.metadata(id)?.new_bubble_id == self.metadata(target)?.new_bubble_id
                        }
                        None => false,
                    };
                    if crate::ball_lifecycle::remote_follower(
                        self.metadata(id)?.is_free,
                        mode_number(control.mode),
                        target.is_some(),
                        same,
                    ) {
                        result.push(id);
                    }
                }
                result.sort_unstable();
                Ok(json!(result))
            }
            "GetCenterDist" => {
                let first_id = value_i64(args.first(), "first_id")?;
                let second_id = value_i64(args.get(1), "second_id")?;
                if !self.balls.contains_key(&first_id) || !self.balls.contains_key(&second_id) {
                    Ok(Value::Null)
                } else {
                    json_number(stable_vec3_length(
                        self.position(first_id)? - self.position(second_id)?,
                    ))
                }
            }
            "GetSurfaceDist" => self.surface_distance(args),
            "GetBallIdsInRange" => self.get_ball_ids_in_range(args, false),
            "GetBallIdsAndDistInRange" => self.get_ball_ids_in_range(args, true),
            "GetBallIdsInCapsule" => self.get_ball_ids_in_capsule(args),
            "GetBallIdsInCone" => self.get_ball_ids_in_cone(args),
            "GetBallIdsInRangeOfTriangle" => self.get_ball_ids_in_triangle(args),
            "ScanCone" => self.scan_cone(args),
            "AddProximitySensor" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.add_proximity_sensor(id, &args[1..])
            }
            "RemoveProximitySensor" => {
                self.remove_proximity_sensors(value_i64(args.first(), "ball_id")?)?;
                Ok(Value::Null)
            }
            "CloakBall" => {
                let id = value_i64(args.first(), "ball_id")?;
                let mode = value_i64(args.get(1), "cloak_mode")?;
                if !(1..=3).contains(&mode) {
                    return Err(CompatError::InvalidRequest(
                        "cloak mode must be 1, 2, or 3".into(),
                    ));
                }
                let range = args
                    .get(2)
                    .map(|value| value_f64(Some(value), "uncloak_range"))
                    .transpose()?;
                self.cloak_ball(id, mode as i32, range)?;
                Ok(Value::Null)
            }
            "UncloakBall" => {
                let id = value_i64(args.first(), "ball_id")?;
                self.uncloak_ball(id)?;
                Ok(Value::Null)
            }
            "CheckVisibility" => {
                let source = value_i64(args.first(), "source_id")?;
                let destination = value_i64(args.get(1), "destination_id")?;
                Ok(json!(self.check_visibility(source, destination)?))
            }
            "GetCurrentEgoPos" => {
                if self.park.ego <= 0
                    || !self.balls.contains_key(&self.park.ego)
                    || self.park.pending_removals.contains_key(&self.park.ego)
                {
                    return Ok(json!([0.0, 0.0, 0.0]));
                }
                let position = self.position(self.park.ego)?;
                Ok(json!([position.x, position.y, position.z]))
            }
            "WriteFullStateToStream" => {
                let source_id = match args.first() {
                    None | Some(Value::Null) => None,
                    Some(value) if value.as_i64() == Some(-1) => None,
                    Some(value) => Some(value_i64(Some(value), "source_id")?),
                };
                Ok(json!(self.serialize_snapshot(None, source_id)?))
            }
            "WriteBallsToStream" => {
                let ids = args
                    .first()
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        CompatError::InvalidRequest("WriteBallsToStream expects an ID array".into())
                    })?
                    .iter()
                    .map(|value| value_i64(Some(value), "ball_id"))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(json!(self.serialize_snapshot(Some(&ids), None)?))
            }
            "ReadFullStateFromStream" => {
                let encoded = args.first().and_then(Value::as_str).ok_or_else(|| {
                    CompatError::InvalidRequest(
                        "ReadFullStateFromStream expects base64 data".into(),
                    )
                })?;
                self.deserialize_snapshot(
                    encoded,
                    args.get(1)
                        .map(|value| value_i64(Some(value), "partial"))
                        .transpose()?
                        .unwrap_or(0),
                )?;
                Ok(Value::Null)
            }
            "GetBoxCenter" => get_box_center_value(args),
            _ => Err(CompatError::UnsupportedTitle(title.to_owned())),
        }
    }

    fn metadata(&self, ball_id: i64) -> Result<&DestinyBallMetadata, CompatError> {
        let entity = self.entity(ball_id)?;
        self.app
            .world()
            .get::<DestinyBallMetadata>(entity)
            .ok_or_else(|| CompatError::Engine("missing DestinyBallMetadata".into()))
    }

    fn position(&self, ball_id: i64) -> Result<DVec3, CompatError> {
        let entity = self.entity(ball_id)?;
        self.app
            .world()
            .get::<Position>(entity)
            .map(|position| position.0)
            .ok_or_else(|| CompatError::Engine("missing Position".into()))
    }

    fn surface_distance(&self, args: &[Value]) -> Result<Value, CompatError> {
        let first_id = value_i64(args.first(), "first_id")?;
        let second_id = value_i64(args.get(1), "second_id")?;
        if !self.balls.contains_key(&first_id) || !self.balls.contains_key(&second_id) {
            return Ok(Value::Null);
        }
        let center = stable_vec3_length(self.position(first_id)? - self.position(second_id)?);
        let distance = surface_distance_from_center(
            center,
            self.metadata(first_id)?.radius,
            self.metadata(second_id)?.radius,
        );
        json_number(distance)
    }

    fn get_ball_ids_in_range(
        &self,
        args: &[Value],
        include_distance: bool,
    ) -> Result<Value, CompatError> {
        let (center, range, excluded, include_cloaked) = if args.len() == 2 || args.len() == 3 {
            let id = value_i64(args.first(), "source_id")?;
            (
                self.position(id)?,
                value_f64(args.get(1), "range")?,
                Some(id),
                args.get(2)
                    .map_or(Ok(false), |value| value_bool(Some(value), "includeCloaked"))?,
            )
        } else if args.len() == 4 || args.len() == 5 {
            (
                DVec3::new(
                    value_f64(args.first(), "x")?,
                    value_f64(args.get(1), "y")?,
                    value_f64(args.get(2), "z")?,
                ),
                value_f64(args.get(3), "range")?,
                None,
                args.get(4)
                    .map_or(Ok(false), |value| value_bool(Some(value), "includeCloaked"))?,
            )
        } else {
            return Err(CompatError::InvalidRequest(
                "invalid GetBallIdsInRange arguments".into(),
            ));
        };
        if range < 0.0 {
            return Err(CompatError::InvalidRequest(
                "range must be non-negative".into(),
            ));
        }
        if excluded.is_some_and(|id| self.park.pending_removals.contains_key(&id)) {
            return Ok(json!([]));
        }
        if let Some(source_id) = excluded
            && self.metadata(source_id)?.new_bubble_id < 0
        {
            return Ok(json!([]));
        }
        let mut ids: Vec<_> = self.balls.keys().copied().collect();
        ids.sort_unstable();
        let mut result = Vec::new();
        for id in ids {
            if Some(id) == excluded {
                continue;
            }
            if self.park.pending_removals.contains_key(&id) {
                continue;
            }
            let metadata = self.metadata(id)?;
            if metadata.is_cloaked != 0 && !include_cloaked {
                continue;
            }
            if let Some(source_id) = excluded {
                let source = self.metadata(source_id)?;
                if !metadata.is_global && metadata.new_bubble_id != source.new_bubble_id {
                    continue;
                }
            }
            let reach = range + metadata.radius;
            if !reach.is_finite() {
                return Err(CompatError::InvalidRequest(
                    "range plus ball radius overflowed".into(),
                ));
            }
            let distance = stable_vec3_length(self.position(id)? - center);
            if distance <= reach {
                result.push(if include_distance {
                    Value::Array(vec![json_number(distance * distance)?, json!(id)])
                } else {
                    json!(id)
                });
            }
        }
        Ok(Value::Array(result))
    }

    fn get_ball_ids_in_capsule(&self, args: &[Value]) -> Result<Value, CompatError> {
        let source_id = value_i64(args.first(), "source_id")?;
        if self.park.pending_removals.contains_key(&source_id) {
            return Ok(json!([]));
        }
        if self.metadata(source_id)?.new_bubble_id < 0 {
            return Ok(json!([]));
        }
        let start = self.position(source_id)?;
        let segment = DVec3::new(
            value_f64(args.get(1), "x")?,
            value_f64(args.get(2), "y")?,
            value_f64(args.get(3), "z")?,
        );
        let radius = value_f64(args.get(4), "radius")?;
        if radius < 0.0 {
            return Err(CompatError::InvalidRequest(
                "capsule radius must be non-negative".into(),
            ));
        }
        let segment_length = stable_vec3_length(segment);
        if !segment_length.is_finite() {
            return Err(CompatError::InvalidRequest(
                "capsule segment length overflowed".into(),
            ));
        }
        let direction = if segment_length == 0.0 {
            DVec3::ZERO
        } else {
            segment / segment_length
        };
        let mut result = Vec::new();
        let mut ids: Vec<_> = self.balls.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            if self.park.pending_removals.contains_key(&id) {
                continue;
            }
            let metadata = self.metadata(id)?;
            let source_metadata = self.metadata(source_id)?;
            if id == source_id
                || metadata.is_cloaked != 0
                || (!metadata.is_global && metadata.new_bubble_id != source_metadata.new_bubble_id)
            {
                continue;
            }
            let point = self.position(id)?;
            let distance_along = if segment_length == 0.0 {
                0.0
            } else {
                (point - start).dot(direction).clamp(0.0, segment_length)
            };
            let closest = start + direction * distance_along;
            let reach = radius + self.metadata(id)?.radius;
            if !reach.is_finite() {
                return Err(CompatError::InvalidRequest(
                    "capsule radius plus ball radius overflowed".into(),
                ));
            }
            if stable_vec3_length(point - closest) <= reach {
                result.push(json!(id));
            }
        }
        Ok(Value::Array(result))
    }

    fn get_ball_ids_in_cone(&self, args: &[Value]) -> Result<Value, CompatError> {
        let source_id = value_i64(args.first(), "source_id")?;
        if self.park.pending_removals.contains_key(&source_id) {
            return Ok(json!([]));
        }
        if self.metadata(source_id)?.new_bubble_id < 0 {
            return Ok(json!([]));
        }
        let source = self.position(source_id)?;
        let vector = DVec3::new(
            value_f64(args.get(1), "x")?,
            value_f64(args.get(2), "y")?,
            value_f64(args.get(3), "z")?,
        );
        let angle = value_f64(args.get(4), "angle")?;
        if !(0.0..=core::f64::consts::PI).contains(&angle) {
            return Err(CompatError::InvalidRequest(
                "cone angle must be in 0..=pi".into(),
            ));
        }
        let height = stable_vec3_length(vector);
        if !height.is_finite() {
            return Err(CompatError::InvalidRequest(
                "cone vector length overflowed".into(),
            ));
        }
        if height == 0.0 {
            return Ok(json!([]));
        }
        let direction = vector / height;
        let mut result = Vec::new();
        let mut ids: Vec<_> = self.balls.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            if self.park.pending_removals.contains_key(&id) {
                continue;
            }
            let metadata = self.metadata(id)?;
            let source_metadata = self.metadata(source_id)?;
            if id == source_id
                || metadata.is_cloaked != 0
                || (!metadata.is_global && metadata.new_bubble_id != source_metadata.new_bubble_id)
            {
                continue;
            }
            let offset = self.position(id)? - source;
            let radius = self.metadata(id)?.radius;
            let reach = height + radius;
            if !reach.is_finite() {
                return Err(CompatError::InvalidRequest(
                    "cone height plus ball radius overflowed".into(),
                ));
            }
            if sphere_intersects_destiny_cone(offset, radius, direction, height, angle) {
                result.push(json!(id));
            }
        }
        Ok(Value::Array(result))
    }

    fn get_ball_ids_in_triangle(&self, args: &[Value]) -> Result<Value, CompatError> {
        if args.len() != 8 {
            return Err(CompatError::InvalidRequest(
                "GetBallIdsInRangeOfTriangle expects 8 arguments".into(),
            ));
        }
        let source_id = value_i64(args.first(), "source_id")?;
        if self.park.pending_removals.contains_key(&source_id) {
            return Ok(json!([]));
        }
        if self.metadata(source_id)?.new_bubble_id < 0 {
            return Ok(json!([]));
        }
        let a = self.position(source_id)?;
        let b = a + DVec3::new(
            value_f64(args.get(1), "ux")?,
            value_f64(args.get(2), "uy")?,
            value_f64(args.get(3), "uz")?,
        );
        let c = a + DVec3::new(
            value_f64(args.get(4), "vx")?,
            value_f64(args.get(5), "vy")?,
            value_f64(args.get(6), "vz")?,
        );
        if !b.is_finite() || !c.is_finite() {
            return Err(CompatError::InvalidRequest(
                "triangle vertices overflowed".into(),
            ));
        }
        let edge_u = b - a;
        let edge_v = c - a;
        let scale = edge_u.abs().max_element().max(edge_v.abs().max_element());
        let normal = if scale == 0.0 {
            DVec3::ZERO
        } else {
            (edge_u / scale).cross(edge_v / scale)
        };
        if !normal.is_finite() || stable_vec3_length(normal) <= f64::EPSILON * 16.0 {
            return Err(CompatError::InvalidRequest(
                "triangle vertices must define a non-degenerate triangle".into(),
            ));
        }
        let range = value_f64(args.get(7), "range")?;
        if range < 0.0 {
            return Err(CompatError::InvalidRequest(
                "range must be non-negative".into(),
            ));
        }
        let mut result = Vec::new();
        let mut ids: Vec<_> = self.balls.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            if self.park.pending_removals.contains_key(&id) {
                continue;
            }
            let metadata = self.metadata(id)?;
            let source_metadata = self.metadata(source_id)?;
            if id == source_id
                || metadata.is_cloaked != 0
                || (!metadata.is_global && metadata.new_bubble_id != source_metadata.new_bubble_id)
            {
                continue;
            }
            let closest = closest_point_on_triangle(self.position(id)?, a, b, c);
            let reach = range + self.metadata(id)?.radius;
            if !reach.is_finite() {
                return Err(CompatError::InvalidRequest(
                    "triangle range plus ball radius overflowed".into(),
                ));
            }
            if stable_vec3_length(self.position(id)? - closest) <= reach {
                result.push(json!(id));
            }
        }
        Ok(Value::Array(result))
    }

    fn scan_cone(&self, args: &[Value]) -> Result<Value, CompatError> {
        if args.len() != 6 {
            return Err(CompatError::InvalidRequest(
                "ScanCone expects 6 arguments".into(),
            ));
        }
        let source_id = value_i64(args.first(), "source_id")?;
        let angle = value_f64(args.get(1), "angle")?;
        if angle < 0.0 {
            return Err(CompatError::InvalidRequest(
                "angle must be non-negative".into(),
            ));
        }
        let range = value_f64(args.get(2), "range")?;
        if range <= 0.0
            || !self.balls.contains_key(&source_id)
            || self.park.pending_removals.contains_key(&source_id)
        {
            return Ok(Value::Null);
        }
        let mut direction = DVec3::new(
            value_f64(args.get(3), "dx")?,
            value_f64(args.get(4), "dy")?,
            value_f64(args.get(5), "dz")?,
        );
        let direction_length = stable_vec3_length(direction);
        if !direction_length.is_finite() || direction_length <= f64::EPSILON {
            return Ok(Value::Null);
        }
        direction /= direction_length;
        let origin = self.position(source_id)?;
        let source_bubble = self.metadata(source_id)?.new_bubble_id;
        let half_angle = angle * 0.5;
        let sphere = half_angle > core::f64::consts::PI;
        let cosine = half_angle.cos();
        let mut ids = self.balls.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        let result = ids
            .into_iter()
            .filter(|id| {
                let Ok(metadata) = self.metadata(*id) else {
                    return false;
                };
                if *id == source_id
                    || *id < 0
                    || self.park.pending_removals.contains_key(id)
                    || metadata.is_cloaked != 0
                    || (!metadata.is_global && metadata.new_bubble_id != source_bubble)
                {
                    return false;
                }
                let Ok(position) = self.position(*id) else {
                    return false;
                };
                let offset = position - origin;
                let distance = stable_vec3_length(offset);
                let projection = direction.dot(offset);
                distance <= range
                    && (sphere || (projection >= 0.0 && projection >= cosine * distance))
            })
            .map(|id| json!(id))
            .collect();
        Ok(Value::Array(result))
    }

    fn remove_proximity_sensors(&mut self, ball_id: i64) -> Result<(), CompatError> {
        self.entity(ball_id)?;
        self.metadata_mut(ball_id)?.sensors.retain(|sensor| {
            sensor
                .get("cloak_sensor")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        });
        Ok(())
    }
}

fn avian_mass(value: f64) -> f32 {
    if value == 0.0 {
        0.0
    } else {
        // Preserve the distinction between a supported positive Destiny mass
        // and Avian's zero-mass/infinite-solver-mass sentinel. A plain cast can
        // underflow small positive f64 values to zero and invert collision
        // behavior just as surely as an unchecked overflow can.
        value.clamp(f32::MIN_POSITIVE as f64, f32::MAX as f64) as f32
    }
}

fn default_local_counter() -> i64 {
    -1
}
fn default_local_high_counter() -> i64 {
    -(1_i64 << 30)
}

fn visual_vec3(value: DVec3) -> Vec3 {
    let project = |component: f64| component.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32;
    Vec3::new(project(value.x), project(value.y), project(value.z))
}

fn solver_safe_vec3(value: DVec3, limit: f64) -> bool {
    value.is_finite() && value.abs().max_element() <= limit
}

fn json_value_len(value: &Value) -> Result<usize, CompatError> {
    json_encoded_len(value, MAX_CHILD_DESCRIPTOR_BYTES)
        .map_err(|error| CompatError::InvalidRequest(format!("invalid descriptor: {error}")))
}

struct JsonByteCounter {
    len: usize,
    limit: usize,
}

impl Write for JsonByteCounter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let new_len = self
            .len
            .checked_add(buffer.len())
            .ok_or_else(|| io::Error::other("JSON byte length overflow"))?;
        if new_len > self.limit {
            return Err(io::Error::other("JSON byte limit exceeded"));
        }
        self.len = new_len;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn json_encoded_len<T: Serialize>(value: &T, limit: usize) -> Result<usize, String> {
    let mut counter = JsonByteCounter { len: 0, limit };
    serde_json::to_writer(&mut counter, value).map_err(|error| error.to_string())?;
    Ok(counter.len)
}

struct BoundedJsonBytes {
    bytes: Vec<u8>,
    limit: usize,
}

impl Write for BoundedJsonBytes {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let new_len = self
            .bytes
            .len()
            .checked_add(buffer.len())
            .ok_or_else(|| io::Error::other("JSON byte length overflow"))?;
        if new_len > self.limit {
            return Err(io::Error::other("JSON byte limit exceeded"));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn bounded_json_bytes<T: Serialize>(value: &T, limit: usize) -> Result<Vec<u8>, String> {
    let mut writer = BoundedJsonBytes {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut writer, value).map_err(|error| error.to_string())?;
    Ok(writer.bytes)
}

fn stable_vec3_length(vector: DVec3) -> f64 {
    if !vector.is_finite() {
        return f64::INFINITY;
    }
    let scale = vector.abs().max_element();
    if scale == 0.0 {
        0.0
    } else {
        scale * (vector / scale).length()
    }
}

fn stable_vec3_normalize(vector: DVec3) -> Option<DVec3> {
    if !vector.is_finite() {
        return None;
    }
    let scale = vector.abs().max_element();
    if scale == 0.0 {
        return None;
    }
    let scaled = vector / scale;
    let length = scaled.length();
    (length > f64::EPSILON && length.is_finite()).then_some(scaled / length)
}

fn sphere_intersects_destiny_cone(
    offset: DVec3,
    radius: f64,
    direction: DVec3,
    height: f64,
    angle: f64,
) -> bool {
    if !offset.is_finite() {
        return false;
    }
    if angle == 0.0 {
        let distance_along = direction.dot(offset).clamp(0.0, height);
        return stable_vec3_length(offset - direction * distance_along) <= radius;
    }
    let distance = stable_vec3_length(offset);
    let maximum = height + radius;
    if !distance.is_finite() || distance > maximum {
        return false;
    }
    let sine = angle.sin();
    let cosine = angle.cos();
    let axial = direction.dot(offset);
    let radial = stable_vec3_length(offset - direction * axial);
    // Algebraically equivalent to the shifted-apex test, without ever
    // constructing radius / sin(angle). The quotient overflows for valid
    // subnormal angles and previously rejected on-axis spheres.
    let expanded_axial = axial.mul_add(sine, radius);
    if !axial.is_finite()
        || !radial.is_finite()
        || !expanded_axial.is_finite()
        || expanded_axial <= 0.0
        || expanded_axial <= cosine.abs() * radial
    {
        return false;
    }
    let reverse_projection = -axial;
    !(reverse_projection > 0.0 && reverse_projection >= distance * sine.abs() && distance > radius)
}

fn tick_duration(milliseconds: f64) -> Result<Duration, CompatError> {
    if !milliseconds.is_finite() || milliseconds <= 0.0 {
        return Err(CompatError::InvalidRequest(
            "tick interval must be finite and positive".into(),
        ));
    }
    let duration = Duration::try_from_secs_f64(milliseconds / 1000.0).map_err(|_| {
        CompatError::InvalidRequest("tick interval is outside Duration's supported range".into())
    })?;
    if duration.is_zero() {
        return Err(CompatError::InvalidRequest(
            "tick interval is below Duration's one-nanosecond resolution".into(),
        ));
    }
    Ok(duration)
}

fn deterministic_sensor_phase(ball_id: i64, period: f64) -> f64 {
    // SplitMix64 gives a stable, platform-independent pseudo-random phase
    // without introducing hidden RNG state into snapshots or replays.
    let mut value = (ball_id as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    let unit = (value >> 11) as f64 * (1.0 / ((1_u64 << 53) as f64));
    unit * period
}

fn root_collider_for_metadata(metadata: &DestinyBallMetadata) -> Result<Collider, CompatError> {
    if metadata.is_free || metadata.minis.is_empty() {
        return Ok(Collider::sphere(metadata.radius));
    }
    let mut shapes: Vec<(Position, Rotation, Collider)> = vec![(
        Position(DVec3::ZERO),
        Rotation(DQuat::IDENTITY),
        Collider::sphere(metadata.radius),
    )];
    for (index, descriptor) in metadata.minis.iter().enumerate() {
        if descriptor.get("kind").and_then(Value::as_str) == Some("sphere")
            && metadata.mini_ids.get(index).copied().unwrap_or(0) != 0
        {
            continue; // This source mini sphere has its own registered fixed body.
        }
        shapes.push(mini_collider(descriptor)?);
    }
    if shapes.len() == 1 {
        Ok(Collider::sphere(metadata.radius))
    } else {
        Ok(Collider::compound(shapes))
    }
}

fn mini_collider(value: &Value) -> Result<(Position, Rotation, Collider), CompatError> {
    let object = value.as_object().ok_or_else(|| {
        CompatError::InvalidRequest("mini collider descriptor must be an object".into())
    })?;
    match object.get("kind").and_then(Value::as_str) {
        Some("sphere") => {
            require_exact_object_keys(object, &["kind", "position", "radius"], "mini sphere")?;
            let position = value_vec3(object.get("position"), "mini sphere position")?;
            let radius = value_f64(object.get("radius"), "mini sphere radius")?;
            if radius <= 0.0 || radius > MAX_RADIUS || !solver_safe_vec3(position, MAX_COORDINATE) {
                return Err(CompatError::InvalidRequest(
                    "mini sphere exceeds the solver-safe position/radius envelope".into(),
                ));
            }
            Ok((
                Position(position),
                Rotation(DQuat::IDENTITY),
                Collider::sphere(radius),
            ))
        }
        Some("capsule") => {
            require_exact_object_keys(object, &["kind", "a", "b", "radius"], "mini capsule")?;
            let a = value_vec3(object.get("a"), "mini capsule endpoint a")?;
            let b = value_vec3(object.get("b"), "mini capsule endpoint b")?;
            let radius = value_f64(object.get("radius"), "mini capsule radius")?;
            let segment_length = stable_vec3_length(b - a);
            if radius <= 0.0
                || radius > MAX_RADIUS
                || !solver_safe_vec3(a, MAX_COORDINATE)
                || !solver_safe_vec3(b, MAX_COORDINATE)
                || !segment_length.is_finite()
            {
                return Err(CompatError::InvalidRequest(
                    "mini capsule needs a positive radius and finite endpoints".into(),
                ));
            }
            if segment_length == 0.0 {
                // Original Capsule::CheckCollision explicitly treats a zero
                // length axis as a sphere. Avoid a degenerate Avian capsule.
                Ok((
                    Position(a),
                    Rotation(DQuat::IDENTITY),
                    Collider::sphere(radius),
                ))
            } else {
                Ok((
                    Position(DVec3::ZERO),
                    Rotation(DQuat::IDENTITY),
                    Collider::capsule_endpoints(radius, a, b),
                ))
            }
        }
        Some("box") => {
            require_exact_object_keys(object, &["kind", "basis"], "mini box")?;
            let values = object
                .get("basis")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    CompatError::InvalidRequest(
                        "mini box basis must be a twelve-element array".into(),
                    )
                })?;
            if values.len() != 12 {
                return Err(CompatError::InvalidRequest(
                    "mini box basis must have twelve elements".into(),
                ));
            }
            let values = values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    value_f64(Some(value), &format!("mini box component {index}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let corner = DVec3::new(values[0], values[1], values[2]);
            let axis_x = DVec3::new(values[3], values[4], values[5]);
            let axis_y = DVec3::new(values[6], values[7], values[8]);
            let axis_z = DVec3::new(values[9], values[10], values[11]);
            let lengths = DVec3::new(
                stable_vec3_length(axis_x),
                stable_vec3_length(axis_y),
                stable_vec3_length(axis_z),
            );
            if !lengths.is_finite() || lengths.min_element() <= 0.0 {
                return Err(CompatError::InvalidRequest(
                    "mini box axes must be finite and non-zero".into(),
                ));
            }
            if !solver_safe_vec3(corner, MAX_COORDINATE)
                || !solver_safe_vec3(axis_x, MAX_COORDINATE)
                || !solver_safe_vec3(axis_y, MAX_COORDINATE)
                || !solver_safe_vec3(axis_z, MAX_COORDINATE)
                || lengths.max_element() > MAX_RADIUS
            {
                return Err(CompatError::InvalidRequest(
                    "mini box exceeds the solver-safe geometry envelope".into(),
                ));
            }
            let (unit_x, unit_y, unit_z) =
                (axis_x / lengths.x, axis_y / lengths.y, axis_z / lengths.z);
            if unit_x.dot(unit_y).abs() > 1e-6
                || unit_x.dot(unit_z).abs() > 1e-6
                || unit_y.dot(unit_z).abs() > 1e-6
            {
                return Err(CompatError::InvalidRequest(
                    "mini box axes must be mutually orthogonal".into(),
                ));
            }
            let basis = DMat3::from_cols(unit_x, unit_y, unit_z);
            if !basis.is_finite() || basis.determinant() <= 0.0 {
                return Err(CompatError::InvalidRequest(
                    "mini box basis must be finite and right-handed".into(),
                ));
            }
            let center = corner + (axis_x + axis_y + axis_z) * 0.5;
            if !center.is_finite() {
                return Err(CompatError::InvalidRequest(
                    "mini box center overflowed".into(),
                ));
            }
            Ok((
                Position(center),
                Rotation(DQuat::from_mat3(&basis).normalize()),
                Collider::cuboid(lengths.x, lengths.y, lengths.z),
            ))
        }
        Some(kind) => Err(CompatError::InvalidRequest(format!(
            "unsupported mini collider kind {kind:?}",
        ))),
        None => Err(CompatError::InvalidRequest(
            "mini collider is missing kind".into(),
        )),
    }
}

fn validate_sensor_descriptor(value: &Value) -> Result<(), CompatError> {
    let object = value
        .as_object()
        .ok_or_else(|| CompatError::InvalidRequest("sensor descriptor must be an object".into()))?;
    const SENSOR_KEYS: &[&str] = &[
        "range",
        "period",
        "shuffle",
        "only_interactives",
        "elapsed",
        "members",
        "cloak_sensor",
    ];
    if let Some(active) = object.get("active") {
        if !active.is_boolean() {
            return Err(CompatError::InvalidRequest(
                "sensor active must be boolean".into(),
            ));
        }
        let mut keys = SENSOR_KEYS.to_vec();
        keys.push("active");
        require_exact_object_keys(object, &keys, "sensor descriptor")?;
    } else {
        require_exact_object_keys(object, SENSOR_KEYS, "sensor descriptor")?;
    }
    let range = value_f64(object.get("range"), "sensor range")?;
    if range.abs() > MAX_COORDINATE {
        return Err(CompatError::InvalidRequest(
            "sensor range exceeds the solver-safe limit".into(),
        ));
    }
    let period = value_f64(object.get("period"), "sensor period")?;
    value_i64(object.get("shuffle"), "sensor shuffle")?;
    if !object
        .get("only_interactives")
        .is_some_and(Value::is_boolean)
    {
        return Err(CompatError::InvalidRequest(
            "sensor only_interactives must be boolean".into(),
        ));
    }
    if !object.get("cloak_sensor").is_some_and(Value::is_boolean) {
        return Err(CompatError::InvalidRequest(
            "sensor cloak_sensor must be boolean".into(),
        ));
    }
    let elapsed = value_f64(object.get("elapsed"), "sensor elapsed")?;
    if elapsed < 0.0 || (period > 0.0 && elapsed >= period) {
        return Err(CompatError::InvalidRequest(
            "sensor elapsed must be in [0, period)".into(),
        ));
    }
    let members = object
        .get("members")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CompatError::InvalidRequest("sensor members must be an integer array".into())
        })?;
    let mut unique = HashSet::new();
    for member in members {
        let member = value_i64(Some(member), "sensor member")?;
        if !unique.insert(member) {
            return Err(CompatError::InvalidRequest(
                "sensor members must be unique".into(),
            ));
        }
    }
    Ok(())
}

fn require_exact_object_keys(
    object: &serde_json::Map<String, Value>,
    expected: &[&str],
    name: &str,
) -> Result<(), CompatError> {
    if object.len() == expected.len() && object.keys().all(|key| expected.contains(&key.as_str())) {
        return Ok(());
    }
    Err(CompatError::InvalidRequest(format!(
        "{name} fields do not match the snapshot schema"
    )))
}

fn validate_formations(formations: &[Vec<[f64; 3]>]) -> Result<(), CompatError> {
    if formations.len() > MAX_FORMATION_DEFINITIONS {
        return Err(CompatError::InvalidRequest(
            "formation definition limit exceeded".into(),
        ));
    }
    let mut offsets = 0usize;
    for formation in formations {
        offsets = offsets
            .checked_add(formation.len())
            .ok_or_else(|| CompatError::InvalidRequest("formation offset count overflow".into()))?;
        if offsets > MAX_FORMATION_OFFSETS
            || formation
                .iter()
                .flatten()
                .any(|value| !value.is_finite() || value.abs() > MAX_COORDINATE)
        {
            return Err(CompatError::InvalidRequest(
                "invalid or oversized formation offsets".into(),
            ));
        }
    }
    Ok(())
}

fn validate_ball_snapshot(ball: &BallSnapshot, max_children: usize) -> Result<(), CompatError> {
    if !ball.mini_ids.is_empty() && ball.mini_ids.len() != ball.minis.len() {
        return Err(CompatError::InvalidRequest(
            "mini identity list does not match descriptors".into(),
        ));
    }
    if ball
        .motion
        .as_ref()
        .is_some_and(|control| control.mode == 9)
        && (ball.is_free || ball.owner_id == ball.id || !ball.minis.is_empty())
    {
        return Err(CompatError::InvalidRequest(
            "invalid mini ball ownership or lifecycle".into(),
        ));
    }
    if let Some(control) = &ball.motion
        && (!matches!(control.mode, 0..=12)
            || control.warp_factor <= 0
            || !control.min_range.is_finite()
            || control.min_range < 0.0
            || !control.warp_distance.is_finite()
            || !control.follow_range.is_finite()
            || control
                .destination
                .iter()
                .any(|value| !value.is_finite() || value.abs() > MAX_COORDINATE)
            || (matches!(control.mode, 1 | 4 | 5 | 12) && control.follow_id.is_none()))
    {
        return Err(CompatError::InvalidRequest(
            "invalid snapshot motion control".into(),
        ));
    }
    if let Some(state) = &ball.notifications
        && (!state.range.is_finite()
            || state.range < 0.0
            || !state.tracking_range.is_finite()
            || state.tracking_range < 0.0)
    {
        return Err(CompatError::InvalidRequest(
            "invalid snapshot notification state".into(),
        ));
    }
    if ball.formation_id > 127 && ball.formation_id != NO_FORMATION {
        return Err(CompatError::InvalidRequest(
            "invalid snapshot formation ID".into(),
        ));
    }
    let scalars = [
        ball.mass,
        ball.radius,
        ball.max_velocity,
        ball.agility,
        ball.speed_fraction,
        ball.max_angular_velocity,
        ball.angular_agility,
    ];
    if scalars.iter().any(|value| !value.is_finite()) {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} contains a non-finite scalar",
            ball.id
        )));
    }
    if ball.mass < 0.0
        || ball.radius < 0.0
        || ball.max_velocity < 0.0
        || ball.max_angular_velocity < 0.0
        || (ball.agility <= 0.0
            && !(ball.agility == 0.0
                && !ball.is_free
                && ball
                    .motion
                    .as_ref()
                    .is_some_and(|control| control.mode == 9)))
        || ball.angular_agility < 0.0
        || !(0.0..=1.0).contains(&ball.speed_fraction)
        || ball.mass > MAX_MASS
        || ball.radius > MAX_RADIUS
        || ball.max_velocity > MAX_VELOCITY
        || ball.max_angular_velocity > MAX_VELOCITY
        || ball.agility > MAX_AGILITY
        || ball.angular_agility > MAX_AGILITY
    {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} contains an out-of-range scalar",
            ball.id
        )));
    }
    let position = vec3_from_slice(&ball.position, "position")?;
    let velocity = vec3_from_slice(&ball.velocity, "velocity")?;
    let angular_velocity = vec3_from_slice(&ball.angular_velocity, "angular_velocity")?;
    if !solver_safe_vec3(position, MAX_COORDINATE)
        || !solver_safe_vec3(velocity, MAX_VELOCITY)
        || !solver_safe_vec3(angular_velocity, MAX_VELOCITY)
    {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} exceeds the solver-safe vector envelope",
            ball.id
        )));
    }
    quat_from_slice(&ball.rotation)?;
    if !(0..=3).contains(&ball.is_cloaked) {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} has an invalid cloak mode",
            ball.id
        )));
    }
    if ball.is_cloaked != 0 && ball.is_massive {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} is both cloaked and massive",
            ball.id
        )));
    }
    if ball.is_cloaked != 0 && ball.massive_before_cloak.is_none() {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} is cloaked without its pre-cloak mass state",
            ball.id
        )));
    }
    if ball.is_cloaked == 0 && ball.massive_before_cloak.is_some() {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} retains cloak-only state while uncloaked",
            ball.id
        )));
    }
    if ball.minis.len().saturating_add(ball.sensors.len()) > max_children {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} exceeds the child-shape limit",
            ball.id
        )));
    }
    let descriptor_bytes =
        ball.minis
            .iter()
            .chain(&ball.sensors)
            .try_fold(0usize, |total, value| {
                total.checked_add(json_value_len(value)?).ok_or_else(|| {
                    CompatError::InvalidRequest("child descriptor size overflow".into())
                })
            })?;
    if descriptor_bytes > MAX_CHILD_DESCRIPTOR_BYTES {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} exceeds the child descriptor byte limit",
            ball.id
        )));
    }
    let cloak_sensors = ball
        .sensors
        .iter()
        .filter(|sensor| {
            sensor
                .get("cloak_sensor")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    if ball.sensors.len().saturating_sub(cloak_sensors) > 1 || cloak_sensors > 1 {
        return Err(CompatError::InvalidRequest(format!(
            "ball {} has duplicate user or cloak sensors",
            ball.id
        )));
    }
    for mini in &ball.minis {
        mini_collider(mini)?;
    }
    for sensor in &ball.sensors {
        validate_sensor_descriptor(sensor)?;
        let sensor_range = sensor.get("range").and_then(Value::as_f64).ok_or_else(|| {
            CompatError::InvalidRequest(format!(
                "ball {} has a sensor without a numeric range",
                ball.id
            ))
        })?;
        let sensor_reach = ball.radius + sensor_range;
        if !sensor_reach.is_finite() {
            return Err(CompatError::InvalidRequest(format!(
                "ball {} has a proximity sensor whose range overflows with its radius",
                ball.id
            )));
        }
        if sensor_reach < 0.0 {
            return Err(CompatError::InvalidRequest(format!(
                "ball {} has a proximity sensor whose range is inside its center",
                ball.id
            )));
        }
    }
    Ok(())
}

fn is_park_target(target: &str) -> bool {
    let parts = target.split(':').collect::<Vec<_>>();
    parts.len() == 2 && parts[0] == "park" && parts[1].parse::<u64>().is_ok()
}

fn encoded_sequence(value: &Value) -> Option<&Vec<Value>> {
    if let Some(values) = value.as_array() {
        return Some(values);
    }
    let object = value.as_object()?;
    (object.len() == 1)
        .then(|| object.get("$destiny_bevy_tuple_v1"))
        .flatten()?
        .as_array()
}

fn validate_queued_network_rows(value: &Value, mode: &str) -> Result<(), CompatError> {
    decode_canonical(value.clone()).map_err(|error| {
        CompatError::InvalidRequest(format!("invalid canonical Carbon value: {error}"))
    })?;
    let mut expanded_rows = 0usize;
    validate_queued_network_rows_inner(value, mode, &mut expanded_rows)
}

fn validate_queued_network_rows_inner(
    value: &Value,
    mode: &str,
    expanded_rows: &mut usize,
) -> Result<(), CompatError> {
    if mode == "batch" {
        let object = value.as_object().ok_or_else(|| {
            CompatError::InvalidRequest("Carbon batch updates must be an object".into())
        })?;
        validate_queued_network_rows_inner(
            object.get("singlecasts").unwrap_or(&Value::Null),
            "singlecast",
            expanded_rows,
        )?;
        return validate_queued_network_rows_inner(
            object.get("narrowcasts").unwrap_or(&Value::Null),
            "narrowcast",
            expanded_rows,
        );
    }
    let rows = value
        .as_array()
        .ok_or_else(|| CompatError::InvalidRequest("Carbon updates must be an array".into()))?;
    if rows.len() > MAX_NETWORK_UPDATE_ROWS {
        return Err(CompatError::InvalidRequest(
            "Carbon update row limit exceeded".into(),
        ));
    }
    for encoded_row in rows {
        let row = encoded_sequence(encoded_row).ok_or_else(|| {
            CompatError::InvalidRequest("Carbon update row must be an array or tuple".into())
        })?;
        if row.len() < 3
            || row
                .get(1)
                .and_then(Value::as_str)
                .is_none_or(|action| action.is_empty())
            || row
                .get(2)
                .is_none_or(|state| encoded_sequence(state).is_none())
        {
            return Err(CompatError::InvalidRequest(
                "Carbon update row has an invalid action or state".into(),
            ));
        }
        if mode == "singlecast" {
            if row.first().and_then(Value::as_i64).is_none() {
                return Err(CompatError::InvalidRequest(
                    "Carbon singlecast recipient must be int64".into(),
                ));
            }
            *expanded_rows = (*expanded_rows).checked_add(1).ok_or_else(|| {
                CompatError::InvalidRequest("expanded Carbon row count overflowed".into())
            })?;
        } else {
            let recipients = row.first().and_then(encoded_sequence).ok_or_else(|| {
                CompatError::InvalidRequest(
                    "Carbon narrowcast recipients must be an array or tuple".into(),
                )
            })?;
            if recipients.len() > MAX_NETWORK_RECIPIENTS_PER_ROW
                || recipients
                    .iter()
                    .any(|recipient| recipient.as_i64().is_none())
            {
                return Err(CompatError::InvalidRequest(
                    "Carbon narrowcast recipient list is invalid or oversized".into(),
                ));
            }
            *expanded_rows = (*expanded_rows)
                .checked_add(recipients.len())
                .ok_or_else(|| {
                    CompatError::InvalidRequest("expanded Carbon row count overflowed".into())
                })?;
        }
        if *expanded_rows > MAX_NETWORK_EXPANDED_ROWS {
            return Err(CompatError::InvalidRequest(
                "expanded Carbon update count exceeds the compatibility limit".into(),
            ));
        }
    }
    Ok(())
}

fn validate_network_request(
    request: &CompatRequest,
    expected_mode: &str,
) -> Result<(), CompatError> {
    if request.operation != "call"
        || !request.target.as_deref().is_some_and(is_park_target)
        || request.args.len() != 1
    {
        return Err(CompatError::InvalidRequest(format!(
            "Carbon {expected_mode} requires operation=call, one envelope, and a park target",
        )));
    }
    Ok(())
}

fn park_call_arities(name: &str) -> Option<&'static [usize]> {
    if name == "LoadFormations" {
        return Some(&[1]);
    }
    if name == "SetBallFormation" {
        return Some(&[2]);
    }
    match name {
        "ClearAll" | "Pause" | "Start" | "Evolve" | "GetCurrentEgoPos" => Some(&[0]),
        "AdjustTimes" | "Stop" | "RemoveProximitySensor" | "UncloakBall" | "WriteBallsToStream" => {
            Some(&[1])
        }
        "RemoveBall" => Some(&[1, 2]),
        "SetBallPosition" | "SetBallVelocity" | "SetBallAngularVelocity" => Some(&[4]),
        "SetBallRotation" => Some(&[5]),
        "SetBallMass" | "SetBallRadius" | "SetMaxSpeed" | "SetMaxAngularSpeed" | "SetBallFree"
        | "SetBallGlobal" | "SetBallMassive" | "SetBallInteractive" | "SetSpeedFraction"
        | "SetBallAgility" | "GetCenterDist" | "GetSurfaceDist" | "CheckVisibility" => Some(&[2]),
        "GetBallIdsInRange" | "GetBallIdsAndDistInRange" => Some(&[2, 3, 4, 5]),
        "GetBallIdsInCapsule" | "GetBallIdsInCone" => Some(&[5]),
        "GetBallIdsInRangeOfTriangle" => Some(&[8]),
        "ScanCone" => Some(&[6]),
        "AddProximitySensor" => Some(&[2, 3, 4, 5]),
        "CloakBall" => Some(&[2, 3]),
        "WriteFullStateToStream" => Some(&[0, 1]),
        "ReadFullStateFromStream" => Some(&[1, 2]),
        "GetBoxCenter" => Some(&[4]),
        _ => None,
    }
}

fn require_arity(args: &[Value], allowed: &[usize], title: &str) -> Result<(), CompatError> {
    if allowed.contains(&args.len()) {
        return Ok(());
    }
    Err(CompatError::InvalidRequest(format!(
        "{title} expects argument count in {allowed:?}, got {}",
        args.len(),
    )))
}

fn parse_ball_target(target: Option<&str>) -> Result<i64, CompatError> {
    let target = target
        .ok_or_else(|| CompatError::InvalidRequest("ball operation requires target".into()))?;
    let parts = target.split(':').collect::<Vec<_>>();
    let id = match parts.as_slice() {
        ["ball", id] => *id,
        ["ball", park, id] if park.parse::<u64>().is_ok() => *id,
        ["ball", park, generation, id]
            if park.parse::<u64>().is_ok() && generation.parse::<u64>().is_ok() =>
        {
            *id
        }
        _ => {
            return Err(CompatError::InvalidRequest(format!(
                "invalid ball target {target:?}"
            )));
        }
    };
    if id.is_empty() {
        return Err(CompatError::InvalidRequest(format!(
            "invalid ball target {target:?}"
        )));
    }
    id.parse()
        .map_err(|_| CompatError::InvalidRequest(format!("invalid ball target {target:?}")))
}

fn parse_ball_generation(target: Option<&str>) -> Result<Option<u64>, CompatError> {
    match target.map(|value| value.split(':').collect::<Vec<_>>()) {
        Some(parts) if parts.len() == 4 => parts[2]
            .parse::<u64>()
            .map(Some)
            .map_err(|_| CompatError::InvalidRequest("invalid object generation".into())),
        _ => Ok(None),
    }
}

fn retained_ball_property(ball: &RetainedBall, name: &str) -> Result<Value, CompatError> {
    let snapshot = ball
        .detached
        .as_ref()
        .ok_or_else(|| CompatError::Engine("missing retained object state".into()))?;
    match name {
        "id" => Ok(json!(ball.id)),
        "isMoribund" => Ok(json!(ball.moribund)),
        "harmonic" | "corporationID" | "allianceID" => Ok(json!(
            ball.identity_fields[match name {
                "harmonic" => 0,
                "corporationID" => 1,
                _ => 2,
            }]
        )),
        "roll" | "pitch" | "yaw" => {
            let rotation = quat_from_slice(&snapshot.rotation)?;
            let (yaw, pitch, roll) = rotation.to_euler(EulerRot::YXZ);
            json_number(match name {
                "roll" => roll,
                "pitch" => pitch,
                _ => yaw,
            })
        }
        "followId" => Ok(json!(
            snapshot
                .motion
                .as_ref()
                .and_then(|motion| motion.follow_id)
                .unwrap_or(0)
        )),
        "centerDist" => json_number(-1.0),
        "surfaceDist" => json_number(-1.0 - snapshot.radius),
        "ballpark" => Ok(Value::Null),
        "mode" => Ok(json!(
            snapshot
                .motion
                .as_ref()
                .map_or(mode_number(OriginalMode::Stop), |motion| motion.mode)
        )),
        "x" | "y" | "z" => json_number(
            snapshot.position[match name {
                "x" => 0,
                "y" => 1,
                _ => 2,
            }],
        ),
        "vx" | "vy" | "vz" => json_number(
            snapshot.velocity[match name {
                "vx" => 0,
                "vy" => 1,
                _ => 2,
            }],
        ),
        "mass" => json_number(snapshot.mass),
        "ownerId" => Ok(json!(snapshot.owner_id)),
        "followRange" => json_number(
            snapshot
                .motion
                .as_ref()
                .map_or(10.0, |motion| f64::from(motion.follow_range)),
        ),
        "gotoX" | "gotoY" | "gotoZ" => {
            json_number(snapshot.motion.as_ref().map_or(0.0, |motion| {
                motion.destination[match name {
                    "gotoX" => 0,
                    "gotoY" => 1,
                    _ => 2,
                }]
            }))
        }
        "radius" => json_number(snapshot.radius),
        "maxVelocity" => json_number(snapshot.max_velocity),
        "Agility" => json_number(snapshot.agility),
        "speedFraction" => json_number(snapshot.speed_fraction),
        "isFree" => Ok(json!(snapshot.is_free)),
        "isGlobal" => Ok(json!(snapshot.is_global)),
        "isMassive" => Ok(json!(snapshot.is_massive)),
        "isInteractive" => Ok(json!(snapshot.is_interactive)),
        "isCloaked" => Ok(json!(snapshot.is_cloaked)),
        "newBubbleId" => Ok(json!(snapshot.new_bubble_id)),
        "oldBubbleId" => Ok(json!(snapshot.old_bubble_id)),
        "effectStamp" => Ok(json!(snapshot.effect_stamp)),
        "formationID" => Ok(json!(snapshot.formation_id)),
        "miniBalls" | "miniCapsules" | "miniBoxes" => {
            let kind = match name {
                "miniBalls" => "sphere",
                "miniCapsules" => "capsule",
                _ => "box",
            };
            Ok(Value::Array(
                snapshot
                    .minis
                    .iter()
                    .enumerate()
                    .filter(|(_, mini)| mini.get("kind").and_then(Value::as_str) == Some(kind))
                    .map(|(index, mini)| {
                        let mut row = mini.clone();
                        row["id"] = json!(snapshot.mini_ids.get(index).copied().unwrap_or(0));
                        row
                    })
                    .collect(),
            ))
        }
        _ => Err(CompatError::UnsupportedTitle(format!(
            "destiny.Ball.{name}"
        ))),
    }
}

fn retained_ball_call(
    ball: &RetainedBall,
    name: &str,
    args: &[Value],
) -> Result<Value, CompatError> {
    match name {
        "ReserveFormationSlot" => {
            require_arity(args, &[0], name)?;
            Ok(json!(-1))
        }
        "FreeFormationSlot" => {
            require_arity(args, &[1], name)?;
            value_i64(args.first(), "slot")?;
            Ok(Value::Null)
        }
        "GetRotatedVector" => {
            require_arity(args, &[1], name)?;
            let vector = value_vec3(args.first(), "vector")?;
            let snapshot = ball
                .detached
                .as_ref()
                .ok_or_else(|| CompatError::Engine("missing retained state".into()))?;
            let result = quat_from_slice(&snapshot.rotation)? * vector;
            Ok(json!([result.x, result.y, result.z]))
        }
        _ => Err(CompatError::UnsupportedTitle(format!(
            "destiny.Ball.{name}"
        ))),
    }
}

fn value_source_float(value: Option<&Value>, name: &str) -> Result<f64, CompatError> {
    if let Some(object) = value.and_then(Value::as_object)
        && object.len() == 1
        && object.contains_key("ieee754_bits")
    {
        return object["ieee754_bits"]
            .as_u64()
            .map(f64::from_bits)
            .ok_or_else(|| CompatError::InvalidRequest("IEEE754 operand requires64bits".into()));
    }
    value_f64(value, name)
}

fn value_f64(value: Option<&Value>, name: &str) -> Result<f64, CompatError> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| CompatError::InvalidRequest(format!("{name} must be a finite number")))
}

fn value_u8(value: Option<&Value>, name: &str) -> Result<u8, CompatError> {
    let number = match value {
        Some(Value::Bool(value)) => i64::from(*value),
        _ => value_i64(value, name)?,
    };
    u8::try_from(number)
        .map_err(|_| CompatError::InvalidRequest(format!("{name} exceeds source unsigned byte")))
}

fn value_c_int(value: Option<&Value>, name: &str) -> Result<i32, CompatError> {
    let number = match value {
        Some(Value::Bool(value)) => i64::from(*value),
        _ => value_i64(value, name)?,
    };
    i32::try_from(number).map_err(|_| CompatError::InvalidRequest(format!("{name} exceeds C int")))
}

fn value_i64(value: Option<&Value>, name: &str) -> Result<i64, CompatError> {
    value
        .and_then(Value::as_i64)
        .ok_or_else(|| CompatError::InvalidRequest(format!("{name} must be an integer")))
}

fn value_bool(value: Option<&Value>, name: &str) -> Result<bool, CompatError> {
    match value {
        Some(Value::Bool(value)) => Ok(*value),
        Some(Value::Number(value)) if value.as_i64().is_some() => Ok(value.as_i64() != Some(0)),
        _ => Err(CompatError::InvalidRequest(format!(
            "{name} must be a boolean"
        ))),
    }
}

fn json_number(value: f64) -> Result<Value, CompatError> {
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| CompatError::Engine("attempted to return a non-finite number".into()))
}

fn value_vec3(value: Option<&Value>, name: &str) -> Result<DVec3, CompatError> {
    let values = value.and_then(Value::as_array).ok_or_else(|| {
        CompatError::InvalidRequest(format!("{name} must be a three-element array"))
    })?;
    if values.len() != 3 {
        return Err(CompatError::InvalidRequest(format!(
            "{name} must have three elements"
        )));
    }
    Ok(DVec3::new(
        value_f64(values.first(), name)?,
        value_f64(values.get(1), name)?,
        value_f64(values.get(2), name)?,
    ))
}

fn vec3_from_slice(values: &[f64], name: &str) -> Result<DVec3, CompatError> {
    if values.len() != 3 || values.iter().any(|value| !value.is_finite()) {
        return Err(CompatError::InvalidRequest(format!(
            "{name} must contain three finite numbers"
        )));
    }
    Ok(DVec3::new(values[0], values[1], values[2]))
}

fn quat_from_slice(values: &[f64]) -> Result<DQuat, CompatError> {
    if values.len() != 4 || values.iter().any(|value| !value.is_finite()) {
        return Err(CompatError::InvalidRequest(
            "rotation must contain four finite numbers".into(),
        ));
    }
    normalized_quat(DQuat::from_xyzw(values[0], values[1], values[2], values[3]))
}

fn normalized_quat(rotation: DQuat) -> Result<DQuat, CompatError> {
    if !rotation.is_finite() {
        return Err(CompatError::InvalidRequest(
            "rotation quaternion must be finite and non-zero".into(),
        ));
    }
    let scale = rotation
        .x
        .abs()
        .max(rotation.y.abs())
        .max(rotation.z.abs())
        .max(rotation.w.abs());
    if scale == 0.0 {
        return Err(CompatError::InvalidRequest(
            "rotation quaternion must be finite and non-zero".into(),
        ));
    }
    let scaled = DQuat::from_xyzw(
        rotation.x / scale,
        rotation.y / scale,
        rotation.z / scale,
        rotation.w / scale,
    );
    if scaled.length_squared() <= f64::EPSILON {
        return Err(CompatError::InvalidRequest(
            "rotation quaternion must be finite and non-zero".into(),
        ));
    }
    let stable_length = scale * scaled.length();
    if stable_length.is_finite() && (stable_length - 1.0).abs() <= f64::EPSILON * 8.0 {
        // Avoid cumulative one-ulp drift when an already-normalized snapshot
        // quaternion is restored and captured repeatedly.
        return Ok(rotation);
    }
    Ok(scaled.normalize())
}

fn get_box_center_value(args: &[Value]) -> Result<Value, CompatError> {
    if args.len() != 4 {
        return Err(CompatError::InvalidRequest(
            "GetBoxCenter expects level, x, y, z".into(),
        ));
    }
    let level = value_i64(args.first(), "level")?;
    if !(0..8).contains(&level) {
        return Err(CompatError::InvalidRequest("Illegal level".into()));
    }
    let big_box = ((1_i64 << 16) as f64) * 480.0 * 0.25;
    let width = big_box / ((1_i64 << (2 * level)) as f64);
    let grid = 1_i64 << (2 * level - 16 + 39);
    let center = |value: f64| -> Result<f64, CompatError> {
        let quotient = (value + 0.5 * width * grid as f64) / width;
        // `as i64` saturates out-of-range f64 values in Rust. That differs
        // from both the supported Destiny domain and the Python adapter, so
        // reject before conversion. The exclusive upper bound avoids the
        // rounded f64 representation of i64::MAX (2^63).
        if !quotient.is_finite() || quotient < i64::MIN as f64 || quotient >= -(i64::MIN as f64) {
            return Err(CompatError::InvalidRequest(
                "box coordinate maps outside the signed 64-bit grid".into(),
            ));
        }
        let index = quotient.trunc() as i64;
        Ok(index as f64 * width + width * 0.5 - width * grid as f64 * 0.5)
    };
    let x = center(value_f64(args.get(1), "x")?)?;
    let y = center(value_f64(args.get(2), "y")?)?;
    let z = center(value_f64(args.get(3), "z")?)?;
    Ok(json!([x, y, z]))
}

// Real-Time Collision Detection, Christer Ericson, closest point on triangle.
fn closest_point_on_triangle(point: DVec3, a: DVec3, b: DVec3, c: DVec3) -> DVec3 {
    let raw_ab = b - a;
    let raw_ac = c - a;
    let raw_ap = point - a;
    let scale = raw_ab
        .abs()
        .max_element()
        .max(raw_ac.abs().max_element())
        .max(raw_ap.abs().max_element());
    if scale == 0.0 {
        return a;
    }
    let ab = raw_ab / scale;
    let ac = raw_ac / scale;
    let ap = raw_ap / scale;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = (point - b) / scale;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + raw_ab * (d1 / (d1 - d3));
    }

    let cp = (point - c) / scale;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + raw_ac * (d2 / (d2 - d6));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }

    let denominator = 1.0 / (va + vb + vc);
    let v = vb * denominator;
    let w = vc * denominator;
    a + raw_ab * v + raw_ac * w
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(
        title: &str,
        operation: &str,
        target: Option<&str>,
        args: Vec<Value>,
    ) -> CompatRequest {
        CompatRequest {
            title: title.into(),
            operation: operation.into(),
            target: target.map(str::to_owned),
            args,
            kwargs: serde_json::Map::new(),
        }
    }

    fn runtime(is_master: bool) -> CompatRuntime {
        let mut runtime = CompatRuntime::new(CompatOptions {
            enable_network_components: false,
            ..Default::default()
        })
        .expect("runtime");
        runtime
            .dispatch(request(
                "destiny.Ballpark.__init__",
                "construct",
                None,
                vec![json!(is_master)],
            ))
            .expect("construct");
        runtime
    }

    fn add_ball(runtime: &mut CompatRuntime, id: i64, is_free: bool, is_massive: bool) {
        runtime
            .dispatch(request(
                "destiny.Ballpark.AddBall",
                "call",
                Some("park:0"),
                vec![
                    json!(id),
                    json!(10.0),
                    json!(2.0),
                    json!(100.0),
                    json!(is_free),
                    json!(false),
                    json!(is_massive),
                    json!(true),
                    json!(false),
                    json!(0.0),
                    json!(0.0),
                    json!(0.0),
                    json!(10.0),
                    json!(0.0),
                    json!(0.0),
                    json!(0.5),
                    json!(1.0),
                ],
            ))
            .expect("add ball");
    }

    // Original Destiny regression:
    // python/destiny/test/ballpark/test_getters_and_setters.py
    #[test]
    fn original_ignored_setter_values_leave_existing_ball_unchanged() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        let entity = runtime.entity(1).expect("entity");

        let before_mass = runtime.world().get::<DestinyMass>(entity).expect("mass").0;
        let before_radius = runtime.metadata(1).expect("metadata").radius;
        let before_speed = runtime
            .world()
            .get::<MaxLinearSpeed>(entity)
            .expect("max speed")
            .0;
        let before_angular_speed = runtime
            .world()
            .get::<MaxAngularSpeed>(entity)
            .expect("max angular speed")
            .0;
        let before_agility = runtime.metadata(1).expect("metadata").agility;

        for value in [-1.0, 0.0] {
            runtime
                .dispatch(request(
                    "destiny.Ballpark.SetBallMass",
                    "call",
                    Some("park:0"),
                    vec![json!(1), json!(value)],
                ))
                .expect("ignored mass");
            runtime
                .dispatch(request(
                    "destiny.Ballpark.SetBallAgility",
                    "call",
                    Some("park:0"),
                    vec![json!(1), json!(value)],
                ))
                .expect("ignored agility");
        }
        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallRadius",
                "call",
                Some("park:0"),
                vec![json!(1), json!(-1.0)],
            ))
            .expect("ignored radius");
        runtime
            .dispatch(request(
                "destiny.Ballpark.SetMaxSpeed",
                "call",
                Some("park:0"),
                vec![json!(1), json!(-1.0)],
            ))
            .expect("ignored max speed");
        runtime
            .dispatch(request(
                "destiny.Ballpark.SetMaxAngularSpeed",
                "call",
                Some("park:0"),
                vec![json!(1), json!(-1.0)],
            ))
            .expect("ignored max angular speed");

        assert_eq!(
            runtime.world().get::<DestinyMass>(entity).expect("mass").0,
            before_mass
        );
        assert_eq!(runtime.metadata(1).expect("metadata").radius, before_radius);
        assert_eq!(
            runtime
                .world()
                .get::<MaxLinearSpeed>(entity)
                .expect("max speed")
                .0,
            before_speed
        );
        assert_eq!(
            runtime
                .world()
                .get::<MaxAngularSpeed>(entity)
                .expect("max angular speed")
                .0,
            before_angular_speed
        );
        assert_eq!(
            runtime.metadata(1).expect("metadata").agility,
            before_agility
        );
    }

    // Original Destiny regressions:
    // python/destiny/test/ballpark/test_time.py
    #[test]
    fn original_time_start_and_pause_contracts_match() {
        let mut runtime = runtime(false);
        assert_eq!(runtime.park.time, 0);
        assert!(runtime.world().resource::<Time<Physics>>().is_paused());

        runtime
            .dispatch(request(
                "destiny.Ballpark.AdjustTimes",
                "call",
                Some("park:0"),
                vec![json!(2)],
            ))
            .expect("adjust by two");
        assert_eq!(runtime.park.time, 2);
        runtime
            .dispatch(request(
                "destiny.Ballpark.AdjustTimes",
                "call",
                Some("park:0"),
                vec![json!(3)],
            ))
            .expect("adjust by three");
        assert_eq!(runtime.park.time, 5);

        runtime
            .dispatch(request(
                "destiny.Ballpark.Start",
                "call",
                Some("park:0"),
                vec![],
            ))
            .expect("start");
        assert!(!runtime.world().resource::<Time<Physics>>().is_paused());

        runtime
            .dispatch(request(
                "destiny.Ballpark.Pause",
                "call",
                Some("park:0"),
                vec![],
            ))
            .expect("pause");
        assert!(runtime.world().resource::<Time<Physics>>().is_paused());
    }

    // Original Destiny regressions:
    // python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters
    #[test]
    #[expect(
        clippy::approx_constant,
        reason = "3.14 is the exact original Destiny setter fixture, not an approximation of pi"
    )]
    fn original_basic_setters_match_observable_state() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        let entity = runtime.entity(1).expect("entity");

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallMass",
                "call",
                Some("park:0"),
                vec![json!(1), json!(3.14)],
            ))
            .expect("mass");
        assert_eq!(
            runtime.world().get::<DestinyMass>(entity).expect("mass").0,
            3.14
        );

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallRadius",
                "call",
                Some("park:0"),
                vec![json!(1), json!(3.14)],
            ))
            .expect("radius");
        assert_eq!(runtime.metadata(1).expect("metadata").radius, 3.14);

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetMaxSpeed",
                "call",
                Some("park:0"),
                vec![json!(1), json!(3.14)],
            ))
            .expect("max speed");
        assert_eq!(
            runtime
                .world()
                .get::<MaxLinearSpeed>(entity)
                .expect("max speed")
                .0,
            3.14
        );

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallPosition",
                "call",
                Some("park:0"),
                vec![json!(1), json!(1.0), json!(2.0), json!(3.0)],
            ))
            .expect("position");
        assert_eq!(
            runtime.position(1).expect("position"),
            DVec3::new(1.0, 2.0, 3.0)
        );

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallVelocity",
                "call",
                Some("park:0"),
                vec![json!(1), json!(1.0), json!(2.0), json!(3.0)],
            ))
            .expect("velocity");
        assert_eq!(
            runtime
                .world()
                .get::<LinearVelocity>(entity)
                .expect("velocity")
                .0,
            DVec3::new(1.0, 2.0, 3.0)
        );

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetSpeedFraction",
                "call",
                Some("park:0"),
                vec![json!(1), json!(0.2)],
            ))
            .expect("speed fraction");
        assert_eq!(runtime.metadata(1).expect("metadata").speed_fraction, 0.2);

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallFree",
                "call",
                Some("park:0"),
                vec![json!(1), json!(false)],
            ))
            .expect("not free");
        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallFree",
                "call",
                Some("park:0"),
                vec![json!(1), json!(true)],
            ))
            .expect("free");
        assert!(runtime.metadata(1).expect("metadata").is_free);

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallMassive",
                "call",
                Some("park:0"),
                vec![json!(1), json!(false)],
            ))
            .expect("not massive");
        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallMassive",
                "call",
                Some("park:0"),
                vec![json!(1), json!(true)],
            ))
            .expect("massive");
        assert!(runtime.metadata(1).expect("metadata").is_massive);

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallGlobal",
                "call",
                Some("park:0"),
                vec![json!(1), json!(true)],
            ))
            .expect("global");
        assert!(runtime.metadata(1).expect("metadata").is_global);

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallAgility",
                "call",
                Some("park:0"),
                vec![json!(1), json!(3.14)],
            ))
            .expect("agility");
        assert_eq!(runtime.metadata(1).expect("metadata").agility, 3.14);

        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallInteractive",
                "call",
                Some("park:0"),
                vec![json!(1), json!(false)],
            ))
            .expect("not interactive");
        runtime
            .dispatch(request(
                "destiny.Ballpark.SetBallInteractive",
                "call",
                Some("park:0"),
                vec![json!(1), json!(true)],
            ))
            .expect("interactive");
        assert!(runtime.metadata(1).expect("metadata").is_interactive);
    }

    // Original Destiny regressions:
    // python/destiny/test/ballpark/test_getters_and_setters.py distance cases.
    #[test]
    fn original_center_and_surface_distance_cases_match() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        add_ball(&mut runtime, 2, true, true);

        for id in [1, 2] {
            runtime
                .dispatch(request(
                    "destiny.Ballpark.SetBallRadius",
                    "call",
                    Some("park:0"),
                    vec![json!(id), json!(0.0)],
                ))
                .expect("zero radius");
        }

        let center_same = runtime
            .dispatch(request(
                "destiny.Ballpark.GetCenterDist",
                "call",
                Some("park:0"),
                vec![json!(1), json!(2)],
            ))
            .expect("center same");
        let surface_same = runtime
            .dispatch(request(
                "destiny.Ballpark.GetSurfaceDist",
                "call",
                Some("park:0"),
                vec![json!(1), json!(2)],
            ))
            .expect("surface same");
        assert_eq!(center_same, json!(0.0));
        assert_eq!(surface_same, json!(0.0));

        runtime
            .set_position(2, DVec3::new(100.0, 0.0, 0.0))
            .expect("axis position");
        assert_eq!(
            runtime
                .dispatch(request(
                    "destiny.Ballpark.GetCenterDist",
                    "call",
                    Some("park:0"),
                    vec![json!(1), json!(2)],
                ))
                .expect("center axis"),
            json!(100.0)
        );
        assert_eq!(
            runtime
                .dispatch(request(
                    "destiny.Ballpark.GetSurfaceDist",
                    "call",
                    Some("park:0"),
                    vec![json!(1), json!(2)],
                ))
                .expect("surface axis"),
            json!(100.0)
        );

        runtime.set_radius(1, 10.0).expect("left radius");
        runtime.set_radius(2, 5.0).expect("right radius");
        assert_eq!(
            runtime
                .dispatch(request(
                    "destiny.Ballpark.GetSurfaceDist",
                    "call",
                    Some("park:0"),
                    vec![json!(1), json!(2)],
                ))
                .expect("surface radii"),
            json!(85.0)
        );

        runtime.set_radius(1, 0.0).expect("left radius zero");
        runtime.set_radius(2, 0.0).expect("right radius zero");
        runtime
            .set_position(2, DVec3::new(1.0, 2.0, 3.0))
            .expect("3d position");
        let observed = runtime
            .dispatch(request(
                "destiny.Ballpark.GetSurfaceDist",
                "call",
                Some("park:0"),
                vec![json!(1), json!(2)],
            ))
            .expect("3d surface")
            .as_f64()
            .expect("number");
        assert!((observed - 3.7416573867739413).abs() < 1.0e-12);
    }

    // Original Destiny regressions:
    // test_can_create_ballpark
    // test_add_ball_adds_ball_to_park
    // test_remove_ball_from_park
    #[test]
    fn original_ballpark_membership_lifecycle_matches() {
        let mut runtime = runtime(false);
        assert!(runtime.balls.is_empty());

        add_ball(&mut runtime, 1, true, true);
        assert_eq!(runtime.balls.len(), 1);
        assert!(runtime.balls.contains_key(&1));

        runtime
            .dispatch(request(
                "destiny.Ballpark.RemoveBall",
                "call",
                Some("park:0"),
                vec![json!(1)],
            ))
            .expect("remove");
        assert!(runtime.balls.is_empty());
        assert!(!runtime.balls.contains_key(&1));
    }

    #[test]
    fn retained_generation_leases_reclaim_only_after_final_detached_release() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, false, false);
        let generation = runtime.ball_generations[&1];
        runtime.transition_retained_ball(1, generation, 0).unwrap();
        runtime.transition_retained_ball(1, generation, 0).unwrap();
        runtime.remove_ball(1).unwrap();
        let retained = &runtime.retained_balls[&generation];
        assert!(!retained.attached && retained.moribund);
        assert_eq!(retained.references, 2);
        add_ball(&mut runtime, 1, false, false);
        let replacement = runtime.ball_generations[&1];
        assert_ne!(replacement, generation);
        runtime.transition_retained_ball(1, generation, 1).unwrap();
        assert_eq!(runtime.retained_balls[&generation].references, 1);
        runtime.transition_retained_ball(1, generation, 1).unwrap();
        assert!(!runtime.retained_balls.contains_key(&generation));
        assert!(runtime.retained_balls[&replacement].attached);
        assert!(runtime.transition_retained_ball(1, generation, 0).is_err());
        runtime.clear_all().unwrap();
        assert!(!runtime.retained_balls.contains_key(&replacement));
    }

    #[test]
    fn exact_space_damping_and_pause_contract() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        runtime
            .dispatch(request(
                "destiny.Ballpark.tickInterval",
                "set",
                Some("park:0"),
                vec![json!(100.0)],
            ))
            .expect("tick interval");
        runtime
            .dispatch(request(
                "destiny.Ballpark.friction",
                "set",
                Some("park:0"),
                vec![json!(5.0)],
            ))
            .expect("friction");

        assert_eq!(runtime.position(1).expect("position"), DVec3::ZERO);

        runtime
            .dispatch(request(
                "destiny.Ballpark.Evolve",
                "call",
                Some("park:0"),
                vec![],
            ))
            .expect("evolve");
        let decay = (-0.1_f64).exp();
        assert!((runtime.position(1).expect("position").x - 10.0 * (1.0 - decay)).abs() < 1e-10);
        let entity = runtime.entity(1).expect("entity");
        let velocity = runtime
            .world()
            .get::<LinearVelocity>(entity)
            .expect("velocity")
            .0;
        assert!((velocity.x - 10.0 * decay).abs() < 1e-10);

        let stepped_position = runtime.position(1).expect("position");
        runtime.update().expect("paused update");
        assert_eq!(runtime.position(1).expect("position"), stepped_position);
    }

    #[test]
    fn fixed_mini_balls_are_registered_bodies_with_source_lifecycle() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, false, true);
        runtime
            .dispatch(request(
                "destiny.Ball.AddMiniBall",
                "call",
                Some("ball:0:1"),
                vec![json!(3.0), json!(4.0), json!(5.0), json!(1.0)],
            ))
            .expect("mini ball");
        let entity = runtime.entity(1).expect("entity");
        let collider = runtime.world().get::<Collider>(entity).expect("collider");
        assert!(collider.shape().as_compound().is_none());
        let first_id = runtime.metadata(1).expect("parent").mini_ids[0];
        assert_eq!(first_id, -(1_i64 << 30) - 1);
        assert_eq!(runtime.balls.len(), 2);
        assert_eq!(
            runtime.position(first_id).expect("mini position"),
            runtime.position(1).expect("parent position") + DVec3::new(3.0, 4.0, 5.0)
        );
        assert_eq!(runtime.metadata(first_id).expect("mini").owner_id, 1);
        assert!(!runtime.metadata(first_id).expect("mini").is_free);
        assert!(runtime.metadata(first_id).expect("mini").is_massive);

        runtime.set_free(1, true).expect("dynamic");
        assert!(!runtime.balls.contains_key(&first_id));
        assert_eq!(runtime.balls.len(), 1);
        assert!(
            runtime
                .world()
                .get::<Collider>(entity)
                .expect("collider")
                .shape()
                .as_compound()
                .is_none()
        );
        runtime.set_free(1, false).expect("static");
        let second_id = runtime.metadata(1).expect("parent").mini_ids[0];
        assert_eq!(second_id, first_id - 1);
        assert!(runtime.balls.contains_key(&second_id));
        runtime.set_massive(1, false).expect("disable collider");
        assert!(
            runtime
                .world()
                .entity(entity)
                .contains::<ColliderDisabled>()
        );
        assert!(
            !runtime
                .world()
                .entity(runtime.entity(second_id).expect("mini entity"))
                .contains::<ColliderDisabled>()
        );
        runtime
            .remove_ball(1)
            .expect("remove parent and owned mini");
        assert!(runtime.balls.is_empty());
    }

    #[test]
    fn cloak_transition_is_validated_before_mutation() {
        let mut runtime = runtime(true);
        add_ball(&mut runtime, 1, false, true);
        assert!(runtime.cloak_ball(1, 1, Some(-1.0)).is_err());
        assert_eq!(runtime.metadata(1).expect("metadata").is_cloaked, 0);
        assert!(runtime.metadata(1).expect("metadata").is_massive);

        runtime.cloak_ball(1, 1, None).expect("cloak");
        assert_eq!(runtime.metadata(1).expect("metadata").is_cloaked, 1);
        assert!(!runtime.metadata(1).expect("metadata").is_massive);
        assert_eq!(runtime.metadata(1).expect("metadata").sensors.len(), 1);
        runtime.uncloak_ball(1).expect("uncloak");
        assert_eq!(runtime.metadata(1).expect("metadata").is_cloaked, 0);
        assert!(runtime.metadata(1).expect("metadata").is_massive);
        assert!(runtime.metadata(1).expect("metadata").sensors.is_empty());
    }

    #[test]
    fn invalid_snapshot_is_atomic_and_f64_position_is_lossless() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        let precise = 30_000_000_000_000.125_f64;
        runtime
            .set_position(1, DVec3::new(precise, 0.0, 0.0))
            .expect("position");
        assert_eq!(runtime.position(1).expect("position").x, precise);

        let encoded = runtime.serialize_snapshot(None, None).expect("snapshot");
        let bytes = BASE64.decode(encoded).expect("base64");
        let mut payload: Value = serde_json::from_slice(&bytes).expect("json");
        payload["balls"][0]["radius"] = json!("not-a-number");
        let invalid = BASE64.encode(serde_json::to_vec(&payload).expect("json"));
        assert!(runtime.deserialize_snapshot(&invalid, 0).is_err());
        assert_eq!(runtime.position(1).expect("position").x, precise);
    }

    #[test]
    fn capture_snapshot_returns_one_lock_consistent_stamp_and_state() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        let captured = runtime
            .dispatch(request(
                "dbc.compat.Ballpark.CaptureSnapshot",
                "call",
                Some("park:0"),
                vec![json!(-1)],
            ))
            .expect("capture");
        assert_eq!(captured["current_time"], json!(0));
        let encoded = captured["snapshot"].as_str().expect("snapshot base64");
        let bytes = BASE64.decode(encoded).expect("snapshot base64");
        let payload: Value = serde_json::from_slice(&bytes).expect("snapshot JSON");
        assert_eq!(payload["park"]["current_time"], captured["current_time"]);
        assert_eq!(payload["balls"][0]["id"], json!(1));
    }

    #[test]
    fn box_center_rejects_out_of_int64_grid_coordinates() {
        assert!(
            get_box_center_value(&[json!(7), json!(1.0e100), json!(0.0), json!(0.0),]).is_err()
        );
    }

    #[test]
    fn positive_f64_mass_never_projects_to_avians_zero_sentinel() {
        assert_eq!(avian_mass(0.0), 0.0);
        assert_eq!(avian_mass(f64::MIN_POSITIVE), f32::MIN_POSITIVE);
        assert_eq!(avian_mass(f64::MAX), f32::MAX);
    }

    #[test]
    fn network_envelopes_are_versioned_and_bounded() {
        let mut runtime = runtime(false);
        let valid = json!({
            "protocol": "destiny-carbon-update",
            "schema_version": 2,
            "mode": "singlecast",
            "batch_id": 1,
            "updates": [[7, "DoDestinyUpdate", []]],
        });
        runtime
            .queue_network("singlecast", valid.clone())
            .expect("queue");
        runtime
            .queue_network("singlecast", valid.clone())
            .expect("idempotent retry");
        {
            let outbox = runtime.world().resource::<CarbonNetworkOutbox>();
            assert_eq!(outbox.singlecasts.len(), 1);
            assert!(outbox.queued_bytes > 0);
        }
        let mut conflict = valid;
        conflict["updates"] = json!([[8, "DoDestinyUpdate", []]]);
        assert!(runtime.queue_network("singlecast", conflict).is_err());
        assert!(
            runtime
                .queue_network("singlecast", json!({"protocol": "wrong"}))
                .is_err()
        );
    }

    #[test]
    fn extreme_friction_underflow_is_finite_and_remains_paused() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        runtime
            .dispatch(request(
                "destiny.Ballpark.friction",
                "set",
                Some("park:0"),
                vec![json!(f64::MAX)],
            ))
            .expect("friction");
        let entity = runtime.entity(1).expect("entity");
        let before_damping = *runtime
            .world()
            .get::<LinearDamping>(entity)
            .expect("linear damping");

        runtime
            .dispatch(request(
                "destiny.Ballpark.Evolve",
                "call",
                Some("park:0"),
                vec![],
            ))
            .expect("extreme damping evolve");
        assert_eq!(runtime.park.current_time, 1);
        assert!(runtime.world().resource::<Time<Physics>>().is_paused());
        assert!(runtime.position(1).expect("position").is_finite());
        assert_eq!(
            runtime
                .world()
                .get::<LinearDamping>(entity)
                .expect("linear damping")
                .0,
            before_damping.0,
        );
    }

    #[test]
    fn v4_snapshot_modes_and_cloak_state_are_strict() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        add_ball(&mut runtime, 2, true, true);
        runtime.park.ego = 1;
        let filtered = runtime
            .serialize_snapshot(Some(&[2]), None)
            .expect("filtered snapshot");
        let filtered_bytes = BASE64.decode(filtered).expect("filtered base64");
        let filtered_payload: ParkSnapshot =
            strict_json_from_slice(&filtered_bytes).expect("filtered payload");
        assert_eq!(filtered_payload.park.ego, 0);

        let encoded = runtime.serialize_snapshot(None, None).expect("snapshot");
        assert!(
            runtime
                .dispatch(request(
                    "dbc.compat.Ballpark.Deserialize",
                    "call",
                    Some("park:0"),
                    vec![json!(encoded), json!("1")],
                ))
                .is_err()
        );

        let mut invalid = runtime.ball_snapshot(1).expect("ball snapshot");
        invalid.is_cloaked = 1;
        invalid.is_massive = false;
        invalid.massive_before_cloak = None;
        assert!(validate_ball_snapshot(&invalid, MAX_CONFIGURED_CHILD_SHAPES_PER_BALL).is_err());

        runtime
            .dispatch(request(
                "destiny.Ball.AddMiniBall",
                "call",
                Some("ball:1"),
                vec![json!(1.0), json!(0.0), json!(0.0), json!(0.25)],
            ))
            .expect("mini");
        let encoded = runtime
            .serialize_snapshot(None, None)
            .expect("snapshot with mini");
        let raw = String::from_utf8(BASE64.decode(encoded).expect("base64")).expect("UTF-8");
        let duplicate = raw.replacen(
            "\"kind\":\"sphere\"",
            "\"kind\":\"sphere\",\"kind\":\"sphere\"",
            1,
        );
        assert!(
            runtime
                .deserialize_snapshot(&BASE64.encode(duplicate.as_bytes()), 0)
                .is_err()
        );
    }

    #[test]
    fn snapshot_emission_validates_live_descriptors_and_sorts_members() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        add_ball(&mut runtime, 2, true, true);
        add_ball(&mut runtime, 3, true, true);
        runtime.metadata_mut(1).expect("metadata").sensors = vec![json!({
            "range": 100.0,
            "period": 2.0,
            "shuffle": 0,
            "only_interactives": false,
            "elapsed": 0.0,
            "members": [3, 2],
            "cloak_sensor": false,
        })];

        let encoded = runtime.serialize_snapshot(None, None).expect("snapshot");
        let bytes = BASE64.decode(encoded).expect("snapshot base64");
        let snapshot: ParkSnapshot = strict_json_from_slice(&bytes).expect("snapshot payload");
        let owner = snapshot
            .balls
            .iter()
            .find(|ball| ball.id == 1)
            .expect("owner ball");
        assert_eq!(owner.sensors[0]["members"], json!([2, 3]));

        runtime.metadata_mut(1).expect("metadata").sensors[0]["members"] = json!([99]);
        assert!(runtime.serialize_snapshot(None, None).is_err());
    }

    #[test]
    fn v4_zero_angle_cone_extreme_triangle_and_global_blocker_are_defined() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        add_ball(&mut runtime, 2, true, true);
        add_ball(&mut runtime, 3, true, true);
        runtime
            .set_position(2, DVec3::new(5.0e99, 5.0e99, 0.0))
            .expect("far position");
        runtime
            .set_position(3, DVec3::new(5.0, 0.0, 0.0))
            .expect("near position");
        runtime.advance_one_tick(true).expect("assign bubble");

        assert_eq!(
            runtime
                .get_ball_ids_in_cone(&[json!(1), json!(10.0), json!(0.0), json!(0.0), json!(0.0),])
                .expect("zero cone"),
            json!([3]),
        );
        assert_eq!(
            runtime
                .get_ball_ids_in_cone(&[
                    json!(1),
                    json!(10.0),
                    json!(0.0),
                    json!(0.0),
                    json!(f64::from_bits(1)),
                ])
                .expect("subnormal cone"),
            json!([3]),
        );
        let triangle = runtime
            .get_ball_ids_in_triangle(&[
                json!(1),
                json!(1.0e100),
                json!(0.0),
                json!(0.0),
                json!(0.0),
                json!(1.0e100),
                json!(0.0),
                json!(0.0),
            ])
            .expect("extreme triangle");
        assert!(
            triangle
                .as_array()
                .expect("triangle rows")
                .contains(&json!(2))
        );

        runtime.metadata_mut(3).expect("global metadata").is_global = true;
        runtime
            .metadata_mut(3)
            .expect("global metadata")
            .new_bubble_id = 99;
        runtime
            .set_position(2, DVec3::new(10.0, 0.0, 0.0))
            .expect("destination");
        assert_eq!(runtime.check_visibility(1, 2).expect("visibility"), 3);
    }

    #[test]
    fn speed_clamped_damped_motion_uses_the_clamped_initial_velocity() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        runtime.set_max_velocity(1, 1.0).expect("speed limit");
        runtime
            .dispatch(request(
                "destiny.Ballpark.tickInterval",
                "set",
                Some("park:0"),
                vec![json!(100.0)],
            ))
            .expect("tick interval");
        runtime
            .dispatch(request(
                "destiny.Ballpark.friction",
                "set",
                Some("park:0"),
                vec![json!(5.0)],
            ))
            .expect("friction");
        runtime.advance_one_tick(true).expect("evolve");
        let x = (5.0_f64 / 10.0 / 0.5) * 0.1;
        let expected = -(-x).exp_m1() / x * 0.1;
        assert!((runtime.position(1).expect("position").x - expected).abs() < 1.0e-10);
    }

    #[test]
    fn velocity_setter_preflights_orientation_components_before_mutation() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        let entity = runtime.entity(1).expect("entity");
        let before = runtime
            .world()
            .get::<LinearVelocity>(entity)
            .expect("linear velocity")
            .0;
        runtime.world_mut().entity_mut(entity).remove::<Transform>();

        assert!(runtime.set_velocity(1, DVec3::X).is_err());
        assert_eq!(
            runtime
                .world()
                .get::<LinearVelocity>(entity)
                .expect("linear velocity")
                .0,
            before,
        );
    }

    #[test]
    fn lifecycle_transactions_preflight_before_mutation() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        let entity = runtime.entity(1).expect("entity");
        let before_velocity = runtime
            .world()
            .get::<LinearVelocity>(entity)
            .expect("linear velocity")
            .0;
        runtime
            .world_mut()
            .entity_mut(entity)
            .remove::<AngularVelocity>();

        assert!(runtime.schedule_remove_ball(1, 5).is_err());
        assert_eq!(
            runtime
                .world()
                .get::<LinearVelocity>(entity)
                .expect("linear velocity")
                .0,
            before_velocity,
        );
        assert!(!runtime.park.pending_removals.contains_key(&1));

        runtime
            .world_mut()
            .entity_mut(entity)
            .insert(AngularVelocity(DVec3::ZERO));
        runtime
            .dispatch(request(
                "destiny.Ballpark.RemoveBall",
                "call",
                Some("park:0"),
                vec![json!(1), json!(5)],
            ))
            .expect("scheduled removal");
        let due = runtime.park.pending_removals[&1];
        runtime.world_mut().entity_mut(entity).remove::<Transform>();
        let replacement = vec![
            json!(1),
            json!(20.0),
            json!(2.0),
            json!(100.0),
            json!(true),
            json!(false),
            json!(true),
            json!(true),
            json!(false),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(10.0),
            json!(0.0),
            json!(0.0),
            json!(0.5),
            json!(1.0),
        ];
        assert!(runtime.add_ball(&replacement).is_err());
        assert_eq!(runtime.park.pending_removals.get(&1), Some(&due));
        assert_eq!(
            runtime
                .world()
                .get::<DestinyMass>(entity)
                .expect("Destiny mass")
                .0,
            10.0,
        );
        assert!(
            runtime
                .world()
                .get::<DestinyPendingRemoval>(entity)
                .is_some()
        );
    }

    #[test]
    fn failed_removal_scrub_does_not_partially_mutate_other_sensors() {
        let mut runtime = runtime(false);
        add_ball(&mut runtime, 1, true, true);
        add_ball(&mut runtime, 2, true, true);
        add_ball(&mut runtime, 3, true, true);
        runtime.metadata_mut(1).expect("metadata").sensors = vec![json!({
            "range": 100.0,
            "period": 2.0,
            "shuffle": 0,
            "only_interactives": false,
            "elapsed": 0.0,
            "members": [3],
            "cloak_sensor": false,
        })];
        let corrupt = runtime.entity(2).expect("corrupt entity");
        runtime
            .world_mut()
            .entity_mut(corrupt)
            .remove::<DestinyBallMetadata>();

        assert!(runtime.remove_ball(3).is_err());
        assert!(runtime.balls.contains_key(&3));
        assert_eq!(
            runtime.metadata(1).expect("metadata").sensors[0]["members"],
            json!([3]),
        );
    }

    #[test]
    fn bubble_membership_includes_noninteractive_balls_and_filters_inert_state() {
        let mut runtime = runtime(true);
        add_ball(&mut runtime, 1, true, true);
        add_ball(&mut runtime, 2, true, true);
        add_ball(&mut runtime, 3, true, true);
        for id in [1, 2, 3] {
            runtime.metadata_mut(id).expect("metadata").new_bubble_id = 0;
        }
        runtime.metadata_mut(2).expect("metadata").is_interactive = false;
        {
            let mut global = runtime.metadata_mut(3).expect("metadata");
            global.is_interactive = false;
            global.is_global = true;
        }
        assert_eq!(
            runtime.bubble_membership_value().expect("membership"),
            json!({
                "interactives": {"0": [1]},
                "members": {"0": [1, 2, 3]},
                "observers": {"1": [1, 2, 3]},
            }),
        );

        runtime
            .dispatch(request(
                "destiny.Ballpark.RemoveBall",
                "call",
                Some("park:0"),
                vec![json!(2), json!(2)],
            ))
            .expect("pending removal");
        runtime.metadata_mut(3).expect("metadata").is_cloaked = 2;
        assert_eq!(
            runtime.bubble_membership_value().expect("membership"),
            json!({
                "interactives": {"0": [1]},
                "members": {"0": [1]},
                "observers": {"1": [1]},
            }),
        );
    }

    #[test]
    fn first_tick_fast_body_cannot_cross_a_fixed_sphere_before_a_farther_target() {
        let mut runtime = runtime(false);
        runtime
            .dispatch(request(
                "dbc.compat.Runtime.Configure",
                "call",
                Some("park:0"),
                vec![json!(20), json!(true), json!(false)],
            ))
            .unwrap();
        for (id, mass, z, vz, max_speed) in [
            (1, 13_000_000.0, 4.0, 0.0, 10.0),
            (2, 1_000_000.0, 10.0, 0.0, 10.0),
            (3, 13_000_000.0, -4.0, 20.0, 100.0),
        ] {
            runtime
                .add_ball(&[
                    json!(id),
                    json!(mass),
                    json!(2.0),
                    json!(max_speed),
                    json!(true),
                    json!(false),
                    json!(true),
                    json!(true),
                    json!(false),
                    json!(0.0),
                    json!(0.0),
                    json!(z),
                    json!(0.0),
                    json!(0.0),
                    json!(vz),
                    json!(0.9),
                    json!(0.95),
                ])
                .unwrap();
        }
        runtime.set_free(1, false).unwrap();
        runtime.advance_one_tick(true).unwrap();
        assert_eq!(runtime.position(2).unwrap(), DVec3::new(0.0, 0.0, 10.0));
        assert!(runtime.position(3).unwrap().z < runtime.position(1).unwrap().z);
    }

    #[test]
    fn collision_hooks_isolate_bubbles_but_allow_same_bubble_contacts() {
        fn configured_pair(left_bubble: i64, right_bubble: i64) -> CompatRuntime {
            let mut runtime = runtime(false);
            add_ball(&mut runtime, 1, true, true);
            add_ball(&mut runtime, 2, true, true);
            runtime
                .set_position(1, DVec3::new(-0.9, 0.0, 0.0))
                .expect("left position");
            runtime
                .set_position(2, DVec3::new(0.9, 0.0, 0.0))
                .expect("right position");
            runtime.set_velocity(1, DVec3::X).expect("left velocity");
            runtime
                .set_velocity(2, DVec3::NEG_X)
                .expect("right velocity");
            runtime
                .metadata_mut(1)
                .expect("left metadata")
                .new_bubble_id = left_bubble;
            runtime
                .metadata_mut(2)
                .expect("right metadata")
                .new_bubble_id = right_bubble;
            runtime
                .dispatch(request(
                    "destiny.Ballpark.friction",
                    "set",
                    Some("park:0"),
                    vec![json!(0.0)],
                ))
                .expect("friction");
            runtime
        }

        let mut isolated = configured_pair(10, 20);
        isolated
            .dispatch(request(
                "destiny.Ballpark.Evolve",
                "call",
                Some("park:0"),
                vec![],
            ))
            .expect("isolated evolve");
        for id in [1, 2] {
            let entity = isolated.entity(id).expect("isolated entity");
            assert!(
                isolated
                    .world()
                    .get::<CollidingEntities>(entity)
                    .expect("collision tracker")
                    .is_empty()
            );
        }

        let mut interacting = configured_pair(10, 10);
        interacting
            .dispatch(request(
                "destiny.Ballpark.Evolve",
                "call",
                Some("park:0"),
                vec![],
            ))
            .expect("same-bubble evolve");
        let left = interacting.entity(1).expect("left entity");
        assert!(
            !interacting
                .world()
                .get::<CollidingEntities>(left)
                .expect("collision tracker")
                .is_empty()
        );
    }
}
