//! Native Bevy Destiny port. Blue supplies reusable Python/CGP infrastructure.
#[path = "box.rs"]
pub mod spatial_box;
pub mod ball;
pub mod partition;
pub mod partitionable;
pub mod static_collidable;
pub mod quaternion;
pub mod settings;

pub use spatial_box::{Box, PartitionBox};
pub use partition::{
    BallMode, NearbyCandidates, NearbyCriteria, Partition, PartitionBall, PartitionPlugin,
    PartitionStatic, get_or_create_box, py_get_box_center, register_partition_python,
};
pub use partition::{Key, KeyHash};
pub use partitionable::{BoxKey, DestinyId, ID, SortedSetOfBoxes, Partitionable, PartitionableBehavior};
pub use quaternion::QuaternionD;
pub use settings::{
    DestinySettingsPlugin, GlobalSettings, SettingsConfiguration,
    register_destiny_settings_python, s_globalSettings,
    g_collisionMaxIterations, g_useIterativeCollision, g_useDynamicalOrientation,
    g_disableDynamicalOrientationForMissiles, g_useNewOrbit,
    g_collisionMaxIterationsDefault, g_useIterativeCollisionDefault,
    g_useDynamicalOrientationDefault,
    g_disableDynamicalOrientationForMissilesDefault, g_useNewOrbitDefault,
};

pub use static_collidable::{StaticCollidable, StaticCollidableSortComparer, StaticCollidableGeometry};

pub use ball::BallSortComparer;
pub use spatial_box::BoxSortComparer;
