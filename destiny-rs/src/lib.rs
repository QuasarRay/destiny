//! Native Bevy Destiny port. Blue supplies reusable Python/CGP infrastructure.
#[path = "box.rs"]
pub mod spatial_box;
pub mod partition;
pub mod partitionable;
pub mod quaternion;
pub mod settings;

pub use spatial_box::{BoxObject, PartitionBox};
pub use partition::{
    BallMode, NearbyCandidates, NearbyCriteria, Partition, PartitionBall, PartitionPlugin,
    PartitionStatic, get_or_create_box, py_get_box_center, register_partition_python,
};
pub use partitionable::{BoxKey, DestinyId, Partitionable, PartitionableBehavior};
pub use quaternion::QuaternionD;
pub use settings::{
    ApplyStagedSettings, CanApplyDestinySettings, CanReadDestinySettings,
    DestinySettingsApplierComponent, DestinySettingsPlugin,
    DestinySettingsReaderComponent, GlobalSettings, ReadStagedSettings,
    SettingsBridge, SettingsConfiguration, register_destiny_settings_python,
};
