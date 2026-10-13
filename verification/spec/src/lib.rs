#![forbid(unsafe_code)]

//! Bevy-independent contracts extracted from the original Destiny tests.
//!
//! This crate is intentionally engine-agnostic.  Adapters from a concrete
//! implementation may depend on Bevy/Avian, but the contracts in this crate
//! must not.

mod ball;
mod bounded_list;
mod box_shape;
mod catalog;
mod client_history;
mod client_ticker;
mod collision;
mod formation;
mod geometry;
mod model;
mod original_motion;
mod original_rotation;
mod original_warp;
mod original_stream;
mod original_marshal;
mod network_interests;
mod original_bubbles;
mod original_collision;
mod original_iterative_collision;
mod original_lifetime;
mod original_park;
mod original_partition;
mod original_static_partition;
mod python_assertions;
mod server_actions;
mod server_park_actions;
mod server_ticker;
mod settings;
mod update_batcher;
mod visibility;

pub use ball::{
    MAX_ORIGINAL_FORMATION_SLOTS, ORIGINAL_TEN_BILLION, OriginalBallDefaults,
    child_count_after_capsule_attempt, child_count_after_successful_add, identity_rotated_vector,
    mini_capsule_radius_accepted, next_sequential_formation_slot, original_ball_defaults,
    proximity_sensor_accepted, slot_reused_after_free,
};
pub use bounded_list::BoundedList;
pub use client_ticker::{ClientSnapshot,ClientTickerError,OriginalClientTicker};
pub use server_ticker::{OriginalServerTicker,ServerTickerSignal};
pub use original_lifetime::{OriginalObjectClass,OriginalBallLease,OriginalLifetimeProjection};
pub use original_rotation::{original_rotated_vector,OriginalPythonVectorLists};
pub use original_stream::OriginalStream;
pub use original_marshal::{OriginalMarshalledAction,OriginalMarshalledEvent,marshal_original_event,unmarshal_original_event};
pub use server_park_actions::OriginalServerParkActions;
pub use network_interests::{OriginalNetworkInterests,InterestScopeError};
pub use box_shape::{BoxShape, Rectangle};
pub use catalog::{ORIGINAL_TEST_FILES, OriginalTestFile};
pub use client_history::{
    ClientHistory, ClientHistoryAction, ClientHistoryBatch, ClientHistoryEvent, ClientHistoryState,
    HistoryMergeError, merge_client_state_into_history,
};
pub use collision::{
    first_plane_contact, intersect_segment_plane, intersect_segment_sphere,
    intersect_sphere_triangle,
};
pub use formation::{NO_FORMATION, assign_formation, free_formation_slot, reserve_formation_slot};
pub use geometry::{Aabb, Plane, Triangle};
pub use model::{
    BallState, MotionMode, ParkState, SpecError, Vec3, add_ball_is_permitted, adjust_time,
    apply_non_negative_setter, apply_positive_setter, assign_flag, assign_vec3, center_distance,
    center_distance_squared, clamp_speed_fraction, follow_allowed, membership_count_after_add,
    membership_count_after_remove, missile_follow_range, non_negative_setter_accepts,
    orbit_allowed, positive_setter_accepts, proximity_eligible, running_after_pause,
    running_after_start, stopped_mode, surface_distance, surface_distance_from_center,
    uncloak_restores_massive, visibility_occluder,
};
pub use original_motion::{
    original_goto_thrust, original_integrate_duration, original_integrate_tick,
    original_stop_velocity,
    original_follow_thrust,
    original_old_orbit_thrust,
    original_new_orbit_thrust,
};
pub use original_bubbles::{OriginalBubbleIds,OriginalBubbleDelta,OriginalBubbleDeltaMap,
    OriginalBubbleEntry,OriginalBubbleUpdates,OriginalBubbleUpdater};
pub use original_park::{
    OriginalBallAttribute, OriginalBallInit, OriginalBoundaryCall, OriginalMiniBall, OriginalMiniBox,
    OriginalMiniCapsule, OriginalMode, OriginalParkBall, OriginalParkError, OriginalParkProjection,
    OriginalBoxObject,OriginalBoxObjectKey,
};
pub use python_assertions::{python_almost_equal, python_decimal_round_key, python_rounded_equal,
    python_almost_equal_delta,python_sum_three_floats};
pub use server_actions::{
    ActionName, ActionValue, NetworkAction, NetworkActionError, NetworkActionHistory,
    NetworkActionRecord, NetworkActions, NetworkBallProjection,
};
pub use settings::{OriginalGlobalSettings, OriginalSettingsConfiguration};
pub use update_batcher::{
    BatcherEvent, BatcherHistory, BatcherPayload, BatcherScopeError, BatcherSignal,
    OriginalUpdateBatcher,
    OriginalNetworkCast,OriginalNetworkOutput,CastDispatch,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationStatus {
    Specified,
    ImplementationMissing,
    ProofMissing,
    ProvedParity,
    KnownMismatch,
}

pub use visibility::{
    VisibilityState, cloak_transition, scan_cone_pi_over_2_x, uncloak_transition,
    visibility_candidate_blocks, visibility_result,
};
